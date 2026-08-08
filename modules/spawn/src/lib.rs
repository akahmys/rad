//! Exercises `proc-spawn` from inside a module.
//!
//! Two shapes, because they fail differently. `run` is a command read to
//! completion — the case where the child ends on its own. `pipe` writes to a
//! long-lived child and reads a reply back, which is what an MCP server is:
//! nothing closes, so a reader that only stops at end-of-stream never returns.
#![deny(clippy::pedantic)]

use rad_sdk::Error;

/// `read` reports "nothing yet" and "the pipe closed" identically — both are an
/// empty slice — so the only way to tell a slow child from a finished one is to
/// ask `wait`. That is why every loop here pairs the two.
const WAIT_PENDING: u32 = 504;

#[derive(serde::Deserialize)]
struct RunReq {
    argv: Vec<String>,
}

#[derive(serde::Serialize)]
struct RunRes {
    stdout: String,
    exit_code: i32,
}

#[derive(serde::Deserialize)]
struct PipeReq {
    argv: Vec<String>,
    /// Written to the child's stdin, and expected to come back changed.
    input: String,
}

#[derive(serde::Serialize)]
struct PipeRes {
    line: String,
}

fn spawn(argv: &[String]) -> Result<crate::types::Process, Error> {
    crate::syscall::proc_spawn(argv)
        .map_err(|e| Error::invalid(format!("proc-spawn failed ({}): {}", e.code, e.message)))
}

/// Runs to completion and returns everything the child wrote.
fn run(req: RunReq) -> Result<RunRes, Error> {
    let RunReq { argv } = req;
    let process = spawn(&argv)?;
    let stdout = process.stdout();

    let mut out = Vec::new();
    let exit_code = loop {
        let chunk = stdout
            .read(4096)
            .map_err(|e| Error::invalid(format!("read failed: {}", e.message)))?;
        if !chunk.is_empty() {
            out.extend_from_slice(&chunk);
            continue;
        }
        // Empty: either the child is slow or it is gone. `wait` decides.
        match process.wait() {
            Ok(code) => {
                // One more read after exit — the reader thread may still have
                // had bytes in flight when the process ended.
                if let Ok(last) = stdout.read(4096) {
                    out.extend_from_slice(&last);
                }
                break code;
            }
            // Still running: fall out of the match and read again.
            Err(e) if e.code == WAIT_PENDING => (),
            Err(e) => return Err(Error::invalid(format!("wait failed: {}", e.message))),
        }
    };

    Ok(RunRes {
        stdout: String::from_utf8_lossy(&out).into_owned(),
        exit_code,
    })
}

/// Writes, then reads one line back, leaving the child running.
///
/// The shape MCP needs. Nothing here waits for exit, because an MCP server does
/// not exit — a reader that stopped only at end-of-stream would block forever.
fn pipe(req: PipeReq) -> Result<PipeRes, Error> {
    let PipeReq { argv, input } = req;
    let process = spawn(&argv)?;
    let stdin = process.stdin();
    let stdout = process.stdout();

    let mut payload = input.into_bytes();
    payload.push(b'\n');
    stdin
        .write(&payload)
        .map_err(|e| Error::invalid(format!("write failed: {}", e.message)))?;

    let mut buffer = Vec::new();
    // Bounded rather than open-ended: `read` already waits 100ms per empty
    // call, so this is a ~10s ceiling on a child that never answers. Without
    // it a silent child would spin until the kernel's epoch deadline killed
    // the module, which reports as a runaway rather than as a timeout.
    for _ in 0..100 {
        let chunk = stdout
            .read(4096)
            .map_err(|e| Error::invalid(format!("read failed: {}", e.message)))?;
        buffer.extend_from_slice(&chunk);
        if let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            let line = String::from_utf8_lossy(&buffer[..pos]).trim().to_string();
            return Ok(PipeRes { line });
        }
    }
    Err(Error::invalid(
        "no line came back from the child".to_string(),
    ))
}

/// Spawns and returns immediately, without reading or waiting.
///
/// The case `pipe` cannot cover: `cat` exits by itself once its stdin closes,
/// so a test using it passes whether or not the kernel kills anything. A child
/// that ignores stdin has to be killed to go away.
fn leak(req: RunReq) -> Result<RunRes, Error> {
    let RunReq { argv } = req;
    let _process = spawn(&argv)?;
    Ok(RunRes {
        stdout: String::new(),
        exit_code: 0,
    })
}

rad_sdk::module! {
    wit: "../../wit/kernel/kernel.wit",
    name: "spawn",
    version: "0.1.0",
    methods: {
        "spawn.run" => run,
        "spawn.pipe" => pipe,
        "spawn.leak" => leak,
    }
}

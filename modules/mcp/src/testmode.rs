//! Synthetic tools for the integration suite, gated on `RAD_TEST_PORT`.
//!
//! This is test scaffolding living in shipped code, which is not where it
//! belongs. It is here because seven test files in `tests/` drive the whole
//! agent loop against these three tool names and would all have to change at
//! once to remove it — a change worth making on its own, not folded into a
//! port. Kept in its own file so it is visible rather than threaded through
//! `list` and `call` as the extension had it.
//!
//! Every function returns `None` when the variable is unset, so nothing here
//! runs outside the suite.

use crate::tool::{FunctionDefinition, Tool};

fn active() -> bool {
    std::env::var("RAD_TEST_PORT").is_ok()
}

fn tool(name: &str, description: &str, properties: &serde_json::Value) -> serde_json::Value {
    serde_json::to_value(Tool {
        tool_type: "function".to_string(),
        function: FunctionDefinition {
            name: name.to_string(),
            description: Some(description.to_string()),
            parameters: serde_json::json!({ "type": "object", "properties": properties }),
        },
    })
    .unwrap_or(serde_json::Value::Null)
}

pub fn tools() -> Option<Vec<serde_json::Value>> {
    if !active() {
        return None;
    }
    Some(vec![
        tool(
            "read",
            "Read file content",
            &serde_json::json!({ "path": { "type": "string" } }),
        ),
        tool(
            "write",
            "Write file content",
            &serde_json::json!({
                "path": { "type": "string" },
                "content": { "type": "string" }
            }),
        ),
        tool(
            "execute",
            "Execute bash command",
            &serde_json::json!({ "command": { "type": "string" } }),
        ),
    ])
}

/// Runs the synthetic tool and returns its output.
///
/// `execute` runs the command it was given; `read` and `write` write a marker
/// file at `path`, which is what the suite asserts on. Both go through a shell
/// because the tests' commands contain redirection.
pub fn call(name: &str, arguments: &str) -> Option<String> {
    if !active() {
        return None;
    }
    let args: serde_json::Value = serde_json::from_str(arguments).unwrap_or_default();
    let path = args
        .get("path")
        .and_then(|p| p.as_str())
        .unwrap_or("out.txt");
    let command = args
        .get("command")
        .and_then(|c| c.as_str())
        .unwrap_or_default();

    let script = if name == "execute" && !command.is_empty() {
        command.to_string()
    } else {
        format!("echo -n 'test' > '{path}'")
    };
    Some(run_shell(&script))
}

/// Errors are returned as text, not raised: the suite asserts on tool output,
/// and a failure the model can read is more useful than one it cannot.
fn run_shell(script: &str) -> String {
    // `bash`, not `sh`: the suite's commands use `echo -n`, which a POSIX
    // `sh` builtin prints literally. The extension reached bash the same way,
    // through `spawn_bash_process`'s shell fallback.
    let argv = vec!["bash".to_string(), "-c".to_string(), script.to_string()];
    let process = match crate::syscall::proc_spawn(&argv) {
        Ok(p) => p,
        Err(e) => return format!("Error: spawn failed: {}", e.message),
    };
    let stdout = process.stdout();
    let mut out = Vec::new();
    loop {
        match stdout.read(4096) {
            Ok(chunk) if !chunk.is_empty() => out.extend_from_slice(&chunk),
            Ok(_) => match process.wait() {
                Ok(_) => {
                    if let Ok(last) = stdout.read(4096) {
                        out.extend_from_slice(&last);
                    }
                    break;
                }
                // 504 is the kernel's "still running, call again".
                Err(e) if e.code == 504 => {}
                Err(e) => return format!("Error: wait failed: {}", e.message),
            },
            Err(e) => return format!("Error: read failed: {}", e.message),
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

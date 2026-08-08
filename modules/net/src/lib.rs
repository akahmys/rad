//! Exercises `net-open` from inside a module.
//!
//! The read loop here is the one `llm-transport-openai` will use in AWU 967,
//! and it differs from `modules/spawn`'s in exactly one way: an HTTP body has
//! no `wait` to ask, so "nothing has arrived yet" cannot be reported as an
//! empty read. It comes back as [`PENDING`] instead, and an empty read means
//! the response is over — full stop.
#![deny(clippy::pedantic)]

use rad_sdk::Error;

/// Nothing yet. Not a failure, and emphatically not the end of the body.
const PENDING: u32 = 504;

/// `read` waits 100ms per [`PENDING`], so this is a ~10s ceiling on a peer that
/// connects and then says nothing. Bounded rather than open-ended for the same
/// reason `modules/spawn` bounds its loop: without it the kernel's epoch
/// deadline eventually fires and reports a stall as a runaway module.
const MAX_PENDING: u32 = 100;

fn default_max() -> u32 {
    4096
}

#[derive(serde::Deserialize)]
struct FetchReq {
    url: String,
    #[serde(default)]
    headers: Vec<(String, String)>,
    #[serde(default)]
    body: String,
    /// The `max` passed to each `read`. Deliberately settable: a value smaller
    /// than the chunks the host delivers is what proves the kernel holds the
    /// remainder back instead of discarding it.
    #[serde(default = "default_max")]
    max: u32,
}

#[derive(serde::Serialize)]
struct FetchRes {
    body: String,
    /// How many reads returned bytes, and how many said "not yet". The second
    /// number is what makes the distinction observable from a test: a peer that
    /// pauses mid-response must produce `pending > 0` *and* a complete body.
    reads: u32,
    pending: u32,
}

/// Opens a request and reads the body to completion.
fn fetch(req: FetchReq) -> Result<FetchRes, Error> {
    let FetchReq {
        url,
        headers,
        body,
        max,
    } = req;

    let stream = crate::syscall::net_open(&url, &headers, body.as_bytes())
        .map_err(|e| Error::invalid(format!("net-open failed ({}): {}", e.code, e.message)))?;

    let mut out = Vec::new();
    let mut reads = 0;
    let mut pending = 0;
    loop {
        match stream.read(max) {
            // Empty means the body ended. It cannot also mean "nothing yet" —
            // that is what `PENDING` is for.
            Ok(chunk) if chunk.is_empty() => break,
            Ok(chunk) => {
                out.extend_from_slice(&chunk);
                reads += 1;
            }
            Err(e) if e.code == PENDING => {
                pending += 1;
                if pending > MAX_PENDING {
                    return Err(Error::invalid("peer sent nothing for 10s".to_string()));
                }
            }
            // A transport failure. Reported, not swallowed: returning what was
            // read so far would hand back a truncated body as a complete one.
            Err(e) => {
                return Err(Error::invalid(format!(
                    "read failed ({}): {}",
                    e.code, e.message
                )));
            }
        }
    }

    Ok(FetchRes {
        body: String::from_utf8_lossy(&out).into_owned(),
        reads,
        pending,
    })
}

rad_sdk::module! {
    wit: "../../wit/kernel/kernel.wit",
    name: "net",
    version: "0.1.0",
    methods: {
        "net.fetch" => fetch,
    }
}

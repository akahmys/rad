//! Returns what it was given. Exists so that AWU 955 can demonstrate a real
//! dispatch round trip through the kernel with nothing else in the way.
#![deny(clippy::pedantic)]

use rad_sdk::Error;

#[derive(serde::Deserialize)]
struct SayReq {
    text: String,
}

#[derive(serde::Serialize)]
struct SayRes {
    text: String,
}

fn say(req: SayReq) -> Result<SayRes, Error> {
    // Rejects empty input so the module exercises both arms of the dispatch
    // path — an always-`Ok` handler proves only half of it.
    if req.text.is_empty() {
        return Err(Error::invalid("text must not be empty"));
    }
    Ok(SayRes { text: req.text })
}

rad_sdk::module! {
    wit: "../../wit/kernel/kernel.wit",
    name: "echo",
    version: "0.1.0",
    methods: {
        "echo.say" => say,
    }
}

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Stdio};

pub struct McpProcess {
    pub name: String,
    pub stdin: ChildStdin,
    pub child: Child,
}

impl McpProcess {
    pub fn spawn(
        name: &str,
        cmd: &str,
        args: &[String],
        event_tx: std::sync::mpsc::Sender<crate::ipc::RasCoreEvent>,
    ) -> Result<Self, String> {
        let mut child = std::process::Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn MCP server '{name}': {e}"))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "Failed to open MCP stdin".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to open MCP stdout".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "Failed to open MCP stderr".to_string())?;

        let name_clone = name.to_string();
        let event_tx_clone = event_tx.clone();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line) {
                if n == 0 {
                    break;
                }
                let msg = line.trim().to_string();
                let call_id = serde_json::from_str::<serde_json::Value>(&msg)
                    .ok()
                    .and_then(|v| {
                        v.get("id")
                            .and_then(|id| id.as_str().map(|s| s.to_string()))
                    })
                    .unwrap_or_default();

                let _ = event_tx_clone.send(crate::ipc::RasCoreEvent::McpResponse {
                    call_id,
                    name: name_clone.clone(),
                    message: msg,
                });
                line.clear();
            }
        });

        let name_clone2 = name.to_string();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line) {
                if n == 0 {
                    break;
                }
                eprintln!("[MCP ERROR - {}] {}", name_clone2, line.trim());
                line.clear();
            }
        });

        Ok(Self {
            name: name.to_string(),
            stdin,
            child,
        })
    }

    pub fn send_message(&mut self, msg: &str) -> Result<(), String> {
        writeln!(self.stdin, "{}", msg)
            .map_err(|e| format!("Failed to write to MCP stdin: {e}"))?;
        self.stdin
            .flush()
            .map_err(|e| format!("Failed to flush MCP stdin: {e}"))?;
        Ok(())
    }
}

impl Drop for McpProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

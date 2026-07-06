use std::io::{BufRead, Write};

#[cfg(test)]
mod tests;

pub use rad_models::{Target, TimeoutPolicy, RasCoreEvent, RasRpcCommand, RasRpcRequest, RasRpcResponse};

pub struct IpcBridge<R, W> {
    reader: R,
    writer: W,
}

impl<R: BufRead, W: Write> IpcBridge<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self { reader, writer }
    }

    /// Read next RPC request from stream (1 JSON per line)
    ///
    /// # Errors
    ///
    /// Returns error if reading fails or JSON is invalid.
    pub fn read_request(&mut self) -> Result<Option<RasRpcRequest>, String> {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line).map_err(|e| format!("Failed to read line: {e}"))?;
        if bytes_read == 0 {
            return Ok(None);
        }
        let req = serde_json::from_str(&line).map_err(|e| format!("Invalid JSON: {e}"))?;
        Ok(Some(req))
    }

    /// Write RPC response to stream
    ///
    /// # Errors
    ///
    /// Returns error if writing or flushing fails.
    pub fn write_response(&mut self, resp: &RasRpcResponse) -> Result<(), String> {
        let mut json = serde_json::to_vec(resp).map_err(|e| format!("Serialization error: {e}"))?;
        json.push(b'\n');
        self.writer.write_all(&json).map_err(|e| format!("Write error: {e}"))?;
        self.writer.flush().map_err(|e| format!("Flush error: {e}"))?;
        Ok(())
    }

    /// Write Core Event to stream
    ///
    /// # Errors
    ///
    /// Returns error if writing or flushing fails.
    pub fn write_event(&mut self, event: &RasCoreEvent) -> Result<(), String> {
        let mut json = serde_json::to_vec(event).map_err(|e| format!("Serialization error: {e}"))?;
        json.push(b'\n');
        self.writer.write_all(&json).map_err(|e| format!("Write error: {e}"))?;
        self.writer.flush().map_err(|e| format!("Flush error: {e}"))?;
        Ok(())
    }
}

/// Route specific physical events directly to Stdout/Stderr in real-time
///
/// # Errors
///
/// Returns error if standard stream writing or flushing fails.
pub fn route_event_to_terminal(event: &RasCoreEvent) -> Result<(), String> {
    match event {
        RasCoreEvent::ProcessStdout { data, .. } => {
            std::io::stdout().write_all(data).map_err(|e| format!("Stdout write error: {e}"))?;
            std::io::stdout().flush().map_err(|e| format!("Stdout flush error: {e}"))?;
        }
        RasCoreEvent::ProcessStderr { data, .. } => {
            std::io::stderr().write_all(data).map_err(|e| format!("Stderr write error: {e}"))?;
            std::io::stderr().flush().map_err(|e| format!("Stderr flush error: {e}"))?;
        }
        _ => {}
    }
    Ok(())
}


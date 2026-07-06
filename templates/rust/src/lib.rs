// Generate guest bindings for the rad-extension world defined in wit/rad.wit
wit_bindgen::generate!({
    world: "rad-extension",
    path: "wit",
});

use radcomp::extension::types as wit;

struct ExtensionImpl;

impl Guest for ExtensionImpl {
    /// Entrypoint triggered when RAD Core dispatches events to this extension.
    fn on_event(event: wit::RasCoreEvent) -> Result<(), String> {
        match event {
            wit::RasCoreEvent::HumanInputReceived(prompt) => {
                println!("Template Extension received prompt: {prompt}");
                // You can call host capabilities via host_rpc:
                // let response = host_rpc(&wit::RasRpcCommand::WriteStdout("Hello!".to_string()));
            }
            wit::RasCoreEvent::TaskCompleted => {
                println!("Template Extension notified of task completion.");
            }
            _ => {}
        }
        Ok(())
    }

    /// Security hook triggered before executing sensitive RPC actions.
    /// Return `true` to approve, or `false` to deny.
    fn verify_rpc(command: wit::RasRpcCommand) -> bool {
        match command {
            // Examples of inspecting incoming commands
            wit::RasRpcCommand::SpawnBashProcess(cmd) => {
                println!("Verifying command execution: {cmd}");
                true
            }
            _ => true,
        }
    }
}

// Export the ExtensionImpl struct as the handler for the rad-extension world
export!(ExtensionImpl);

# RAD Extension Developer Guide (v0.0)

Welcome to the RAD (Rust Agent Dispatcher) Extension Developer Guide. RAD uses the **WebAssembly (Wasm) Component Model** for its extension system. This architecture allows developers to build pluggable agents, tool orchestrators, and security validators in any programming language that compiles to Wasm Components (such as Rust, Go, or TypeScript).

---

## 1. Architecture Overview

RAD extensions operate in a sandboxed WebAssembly environment. The interaction between the RAD Core (host) and an Extension (guest) is defined via a **WIT (Wasm Interface Types)** contract.

```mermaid
graph TD
    subgraph RAD Core (Host)
        subsystems[Core Subsystems: FS, Process, DAG, Network]
        wasm_runtime[WASM Runtime]
    end
    
    subgraph WASM Extension (Guest)
        on_event[on-event]
        verify_rpc[verify-rpc]
    end
    
    wasm_runtime -- "1. Dispatch Event (on-event)" --> WASM Extension
    WASM Extension -- "2. Call Host Function (host-rpc)" --> wasm_runtime
    wasm_runtime -- "3. Query Security Hook (verify-rpc)" --> WASM Extension
    wasm_runtime -- "4. Execute Action" --> subsystems
```

---

## 2. WIT Contract Definition

The contract is located at `wit/rad.wit`. It defines a single world `rad-extension` that extensions must implement.

### The `rad-extension` World

```wit
world rad-extension {
    use types.{ras-rpc-command, ras-core-event};
    
    // Host functions provided by RAD Core that the extension can call
    import host-rpc: func(command: ras-rpc-command) -> result<string, string>;

    // Guest functions that the extension must implement (export)
    export on-event: func(event: ras-core-event) -> result<_, string>;
    export verify-rpc: func(command: ras-rpc-command) -> bool;
}
```

* **`host-rpc`**: Allows extensions to execute physical operations (e.g., read files, run shell commands, access DAG).
* **`on-event`**: Dispatched by the core when an event occurs (e.g., terminal output received, file modified, HTTP stream chunk returned).
* **`verify-rpc`**: A security hook. When another extension or component triggers an RPC call, RAD Core queries other active extensions to approve or deny the action.

---

## 3. Main Data Structures

Extensions interact with RAD Core using two main enum-like variants defined in `types`:

### `ras-core-event` (Input to Extension)
Events dispatched from host to guest:
* `human-input-received(string)`: Fired when the user starts a task with an instruction.
* `process-stdout / process-stderr`: Raw output from spawned bash processes.
* `process-exited`: Spawned process termination signal and exit code.
* `file-changed`: Watcher notification of file modifications in the workspace.
* `http-chunk-received`: Stream chunk from http stream.
* `task-completed`: Termination signal for the current execution loop.

### `ras-rpc-command` (Requests to Host)
Actions extensions can request from RAD Core via `host-rpc`:
* `file-read / file-write / file-edit-patch`: Filesystem sandbox operations.
* `spawn-bash-process`: Executes commands in the workspace.
* `create-node / set-node-text / merge-nodes / get-dag`: Management of the context history DAG.
* `open-http-stream`: Initiates non-blocking HTTP streaming connections.
* `ask-human-approval`: Prompts the user for permission (Human-in-the-loop).
* `spawn-mcp-server / send-mcp-request`: Integrates with Model Context Protocol servers.

---

## 4. Configuration and Permissions (`rad.json`)

To load an extension, declare it in `rad.json` under the `extensions` array. Extensions must be granted explicit permissions to execute privileged host RPCs.

```json
{
  "extensions": [
    {
      "name": "my-agent-extension",
      "source": "./ext/my_extension.wasm",
      "enabled": true,
      "permissions": {
        "fs_read_allow": ["./src", "./Cargo.toml"],
        "fs_write_allow": ["./src"],
        "rpc_allow": [
          "FileRead",
          "FileWrite",
          "SpawnBashProcess",
          "AskHumanApproval"
        ]
      }
    }
  ]
}
```

### Permission Properties:
* **`fs_read_allow` / `fs_write_allow`**: Restricts which workspace paths the extension can access via `FileRead` / `FileWrite`.
* **`rpc_allow`**: A list of `RasRpcCommand` variants that the extension is authorized to execute. Any unlisted RPC command will be blocked automatically by RAD Core.

---

## 5. Development Lifecycle

### Step 1: Initialize Project
Create a project structure matching your preferred language (Rust or Go).

### Step 2: Bindings Generation
Use `wit-bindgen` to generate host-guest communication interfaces:
* **Rust**: Declare `wit-bindgen` dependency and use `wit_bindgen::generate!({ world: "rad-extension" });` in your `lib.rs`.
* **Go**: Run `wit-bindgen-go` command to generate Go stubs.

### Step 3: Implement Exports
Implement the `on-event` and `verify-rpc` functions. 

Example Rust skeleton:
```rust
struct MyExtension;

impl Guest for MyExtension {
    fn on_event(event: RasCoreEvent) -> Result<(), String> {
        match event {
            RasCoreEvent::HumanInputReceived(text) => {
                // Initialize LLM prompt and execute tools via host_rpc
                let response = host_rpc(RasRpcCommand::WriteStdout("Hello World!".into()));
            }
            _ => {}
        }
        Ok(())
    }

    fn verify_rpc(command: RasRpcCommand) -> bool {
        // Approve all RPC actions, or restrict unsafe operations
        true
    }
}
```

### Step 4: Compile to Wasm Component
Compile your binary to a Wasm Component:
* For Rust, build with `--target wasm32-wasip1` (or use `cargo-component`).
* For Go, use `tinygo` targeting wasi.

---

## 6. Debugging and Logs

Standard output (`stdout`) and error output (`stderr`) printed inside the Wasm extension are automatically inherited and redirected to the host process terminal. Use standard print macros (`println!`, `eprintln!`, or language equivalents) to log runtime debug information.

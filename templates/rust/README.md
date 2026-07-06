# RAD Rust Extension Template

This template provides a boilerplate for building WebAssembly (Wasm) Extensions for RAD using Rust.

## Prerequisites

1. Install target support for WebAssembly System Interface (WASI):
   ```bash
   rustup target add wasm32-wasip1
   ```

2. (Optional but recommended) Install `cargo-component` for building standardized WASM components:
   ```bash
   cargo install cargo-component --locked
   ```

## Compilation

Build the extension to a Wasm target:
```bash
cargo build --target wasm32-wasip1 --release
```

The compiled WASM file will be located at:
`target/wasm32-wasip1/release/rad_extension_template.wasm`

## Registration

To test your compiled extension, register it in your workspace's `rad.json` file:

```json
{
  "extensions": [
    {
      "name": "my-rust-extension",
      "source": "./target/wasm32-wasip1/release/rad_extension_template.wasm",
      "enabled": true,
      "permissions": {
        "fs_read_allow": ["."],
        "fs_write_allow": ["."],
        "rpc_allow": ["WriteStdout"]
      }
    }
  ]
}
```

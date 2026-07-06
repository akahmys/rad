# RAD Go Extension Template

This template provides a boilerplate for building WebAssembly (Wasm) Extensions for RAD using Go and TinyGo.

## Prerequisites

1. Install Go (version 1.21 or higher).
2. Install [TinyGo](https://tinygo.org/) (required for compiling Go code into compact WASI-compatible Wasm binaries).
3. Install `wit-bindgen-go` generator to translate the WIT contract into Go bindings:
   ```bash
   go install github.com/bytecodealliance/wit-bindgen-go/cmd/wit-bindgen-go@latest
   ```

## Generation & Compilation

1. Generate the Go bindings from the WIT file:
   ```bash
   wit-bindgen-go generate ./wit -o gen
   ```

2. Compile the Go extension into a Wasm Component using TinyGo:
   ```bash
   tinygo build -o extension.wasm -target=wasi -scheduler=none main.go
   ```

## Registration

To test your compiled extension, register it in your workspace's `rad.json` file:

```json
{
  "extensions": [
    {
      "name": "my-go-extension",
      "source": "./templates/go/extension.wasm",
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

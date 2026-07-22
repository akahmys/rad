# `rad` Configuration & Layout Guide

This document defines the rules for operational settings, Extensions, and related assets (Skills, Workflows, etc.) of `rad`, as well as their lookup specifications.

## 1. Configuration File (`rad.json`)

The overall operational parameters of the `rad` Core and the permission constraints of each Extension are managed centrally in `rad.json`.

### 1.1 Configuration File Discovery & Merge Precedence
When the `rad` Core starts, the system looks up and loads configuration files in the following order of precedence. If a local configuration file exists, its properties override or merge with the base configuration:

1. **Explicit Specification**: Path specified via the `--config <PATH>` command-line argument.
2. **Project Local (Recommended)**: `rad.json` in the root of the project.
3. **Project Local (Hidden Directory)**: `.rad/rad.json` inside the project.
4. **User Global (Recommended)**: `~/.rad/config.json` (or `~/.rad/rad.json`).
5. **User Global (XDG Fallback)**: `~/.config/rad/rad.json` (Windows: `%APPDATA%\rad\rad.json`).

**Local Override (Separation of Sensitive Information):**
After loading one of the configuration files above, if **`rad.local.json`** or **`config.local.json`** exists in the same directory (e.g., project root, `.rad/`, or `~/.rad/`), the Core automatically loads it and **merges (overrides)** the settings.
> [!IMPORTANT]
> `rad.local.json` is a local-only file designed to hold personal secrets like API keys. To prevent sharing credentials in repositories, it must always be excluded from Git version control (add it to `.gitignore`).

### 1.2 Configuration Schema Example (Full Parameters with Comments)
`rad.json` supports JSON with comments (JSONC). The following is a full configuration example containing all available parameters with explanatory comments:

```json
{
  // Operation directories for the rad Core
  "core": {
    // Root path of the workspace where the agent performs physical operations (defaults to ".")
    "workspace_dir": ".",
    // Directory where filesystem snapshot backups are saved (defaults to ".rad/snapshots")
    "snapshot_dir": ".rad/snapshots",
    // Directory where execution and session logs are output (defaults to ".rad/logs")
    "log_dir": ".rad/logs"
  },

  // Default timeout values for the Core (in milliseconds)
  "default_timeout": {
    // Maximum allowed idle time between tokens from the LLM (defaults to 15000 = 15s)
    "llm_stream_heartbeat_ms": 15000,
    // Maximum allowed time for a process to be idle before timing out (defaults to 60000 = 60s)
    "process_silent_timeout_ms": 60000
  },

  // List of Extensions (Policy Layer) to register and launch
  "extensions": [
    {
      // Unique identifier for the Extension
      "name": "standard-orchestrator",
      // Source location ("builtin://" schema, or absolute/relative path to local Wasm)
      "source": "builtin://standard-orchestrator",
      // Whether this Extension should be enabled and launched
      "enabled": true,
      // Capability Mask for physical primitives granted to this Extension
      // * Builtin extensions default to full privileges.
      "permissions": {
        // List of directories allowed for read access ("*" allows everything)
        "fs_read_allow": ["*"],
        // List of directories allowed for write access ("*" allows everything)
        "fs_write_allow": ["*"],
        // Bash command execution constraints
        "execution": {
          // Whether executing bash commands (spawn_bash_process) is allowed
          "allow_bash": true,
          // Whitelist of allowed commands/programs (empty means all allowed)
          "allow_commands": ["cargo check", "cargo clippy", "cargo test", "git"],
          // Blacklist of forbidden commands/programs
          "block_commands": ["curl", "wget", "rm -rf /"]
        },
        // Network access constraints
        "network": {
          // Whether external network access is allowed
          "allow_network": true,
          // Whitelist of domains allowed for communication (empty means unrestricted)
          "allow_domains": ["api.openai.com", "api.anthropic.com", "github.com"]
        }
      },
      // Settings passed transparently to the Extension
      // * The Core does not interpret these values, it only forwards them.
      "config": {
        // LLM model name to use
        "model": "claude-3-5-sonnet-20241022",
        // Base API endpoint URL (defaults to Extension's default if omitted)
        "api_base": "https://api.anthropic.com",
        // LLM generation temperature
        "temperature": 0.2
      }
    }
  ]
}
```

### 1.3 Handling Sensitive Information
To handle API keys and other secrets securely, pass them to the Extension using the following methods:

1. **Environment Variables (Recommended)**:
   The Extension retrieves credentials directly from environment variables (e.g., `export ANTHROPIC_API_KEY=sk-...`) in the launching shell. Do not write credentials in configuration files.
2. **Local Configuration File (`rad.local.json`)**:
   In local development environments where you do not want to set environment variables, specify them in `rad.local.json` to be merged:

**Example `rad.local.json`:**
```json
{
  "extensions": [
    {
      "name": "standard-orchestrator",
      "config": {
        "api_key": "sk-ant-..."
      }
    }
  ]
}
```

### 1.4 Optional Parameters & Defaults (Convention)
If settings are omitted from `rad.json`, the Core automatically applies the following default parameters:

| Setting | Default Value | Description |
| :--- | :--- | :--- |
| `core.workspace_dir` | `.` (Current directory) | The physical root path where the agent operates. |
| `core.snapshot_dir` | `.rad/snapshots` | The directory where filesystem snapshots are saved/restored. |
| `core.log_dir` | `.rad/logs` | The directory where operational and session logs are saved. |
| `default_timeout.llm_stream_heartbeat_ms` | `15000` (15s) | The maximum allowed interval between received tokens during LLM streaming. |
| `default_timeout.process_silent_timeout_ms` | `60000` (60s) | The maximum idle duration for a process spawned via `spawn_bash_process` before timing out. |
| `extensions[].permissions` | Builtin: Full / External: Deny All | If omitted, external Extensions default to denying all actions for security. |
| `extensions[].config` | Empty object (`{}`) | Extension-specific configuration passed directly to the Extension. |

---

## 2. Directory Layout Rules

Assets and runtime data related to `rad` are organized in the `.rad/` directory (project local) and the `~/.rad/` directory (user global).

### 2.1 Directory Map

#### 2.1.1 Project Local (Project-specific configuration and data)
```text
<Project Root>/
├── rad.json                   # [Recommended] Project-local configuration file
└── .rad/                      # Project-local data storage
    ├── rad.json               # [Alternative] Hidden project-local configuration file
    ├── ext/                   # Project-specific Extensions (.wasm, etc.)
    │   └── analyzer.wasm
    ├── skills/                # Project-specific custom Skills (scripts)
    │   └── local_helper.sh
    ├── workflows/             # Project-specific Workflows
    │   └── local_flow.json
    ├── snapshots/             # Backup snapshots created by Core
    │   └── <node_id>/
    └── logs/                  # Log output directory
```

#### 2.1.2 User Global (Shared assets across projects)
```text
~/.rad/                        # User-global data storage
│                              # (C:\Users\<User>\.rad\ on Windows)
├── ext/                       # Shared Extensions available to all projects
│   └── global_analyzer.wasm
├── skills/                    # Shared Skills available to all projects
│   └── git_helper.sh
└── workflows/                 # Shared Workflows available to all projects
    └── standard_coding_flow.json
```

### 2.2 Subdirectory Descriptions

#### 2.2.1 Extensions (`.rad/ext/` and `~/.rad/ext/`)
Holds Wasm files and scripts referenced by `extensions.source` in `rad.json`.
- **Project Local**: Under `./.rad/ext/`
- **User Global**: Under `~/.rad/ext/`
- **Package Layout (Directory format)**:
  Extensions can be packaged as directories containing metadata and binaries:
  ```text
  .rad/ext/custom-analyzer/
  ├── extension.json           # Extension metadata and default permissions
  └── main.wasm                # Execution binary
  ```

#### 2.2.2 Skills (`.rad/skills/` and `~/.rad/skills/`)
Holds tools and scripts used by the LLM (or Policy Layer) for specific tasks.
- **Project Local**: Under `./.rad/skills/`
- **User Global**: Under `~/.rad/skills/`
- Upon Core startup, the Extension collects script lists and documentation from both directories and registers them as tools to the LLM.
- The LLM runs these scripts by issuing a command like `.rad/skills/xxx.sh` through `spawn_bash_process`.

#### 2.2.3 Workflows (`.rad/workflows/` and `~/.rad/workflows/`)
Contains development process definitions or initial DAG (Directed Acyclic Graph) templates in JSON/YAML.
- **Project Local**: Under `./.rad/workflows/`
- **User Global**: Under `~/.rad/workflows/`

#### 2.2.4 Snapshots (`.rad/snapshots/`)
Stores the filesystem state backed up by the `take_snapshot` RPC, partitioned by DAG `node_id` (Project local only).

#### 2.2.5 Logs (`.rad/logs/`)
Stores operational logs for the Core/Extension and standard I/O logs for the PTY session.
* Global logs for `rad` run outside any project are saved under `~/.rad/logs/`.

---

## 3. Update Rules for Core & Extensions

`rad` requires no system-level administrative privileges and can be updated and executed entirely within user space.

* **Core Update**: Run `rad --update` to fetch the latest stable binary from the official release and replace the local binary in-place.
* **Extension Update**: Simply overwrite the `.wasm` file or scripts to apply updates dynamically.

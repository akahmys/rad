# `rad` Configuration & Layout Guide

This document defines the rules for operational settings and Extensions of `rad`, as well as their lookup specifications.

## 1. Configuration File

The overall operational parameters of the `rad` Core and the permission constraints of each Extension are managed centrally in a JSON config file — `~/.rad/config.json` (user-global) and/or `rad.json`/`.rad/config.json` (project-local).

### 1.1 Configuration Discovery & Unified Cascade Precedence
When the `rad` Core starts, the system resolves operational parameters and credentials using a strict 5-tier Unified Precedence Cascade (each tier's values override the one below it):

1. **CLI Arguments (Highest Priority)**: Options specified via `--base-url`, `--api-key`, `--model`, `--workspace`, `--config`.
2. **Environment Variables**: `LLM_BASE_URL` / `RAD_BASE_URL`, `LLM_API_KEY` / `RAD_API_KEY`, `LLM_MODEL` / `RAD_MODEL`, `RAD_WORKSPACE`.
3. **Local Directory Override**: `rad.local.json` (or `config.local.json`) next to whichever project config file was found in tier 4.
4. **Project Local Config**: `rad.json` at the project root, or `.rad/config.json` if that isn't present. An explicit `--config <path>` takes the place of this discovery entirely.
5. **User Global Config (Base)**: `~/.rad/config.json`, always loaded first as the base that the other tiers merge on top of.

> [!IMPORTANT]
> `rad.local.json` is a local-only file designed to hold personal secrets like API keys. To prevent sharing credentials in repositories, it must always be excluded from Git version control (add it to `.gitignore`).

### 1.2 Configuration Schema Example (Full Parameters with Comments)
The config file supports JSON with comments (JSONC). The following example registers the 2 extensions and 4 kernel modules `rad` ships, plus two locally-installed MCP servers — without at least one MCP server registered under the `mcp` module, `rad` has no built-in file/shell tools at all and can't act on anything (see [README.md](README.md) §3.2 for more on this):

```json
{
  // Operation directories for the rad Core
  "core": {
    // Root path of the workspace where the agent performs physical operations (defaults to ".")
    "workspace_dir": ".",
    // Directory where filesystem snapshot backups are saved (defaults to ".rad/snapshots")
    "snapshot_dir": ".rad/snapshots",
    // Directory where execution and session logs are output (defaults to ".rad/logs")
    "log_dir": ".rad/logs",
    // Beyond this count, the oldest .rad/sessions/*.json files are pruned at startup
    // (defaults to 50). The currently active session is always kept regardless.
    "max_sessions": 50
  },

  // Default timeout values for the Core (in milliseconds)
  "default_timeout": {
    // Maximum allowed idle time between tokens from the LLM (defaults to 15000 = 15s)
    "llm_stream_heartbeat_ms": 15000,
    // Maximum allowed time for a process to be idle before timing out (defaults to 60000 = 60s)
    "process_silent_timeout_ms": 60000
  },

  // LLM endpoint profiles, managed interactively via the /llm slash command
  // (list, switch, test, add, model, delete, context)
  "llm": {
    // Name of the currently active profile (key into "endpoints" below)
    "active": "local",
    "endpoints": {
      "local": {
        "base_url": "http://localhost:11434",
        "model": "qwen2.5-coder",
        // Detected automatically via /props or /api/show when the endpoint is
        // reachable; set manually with /llm context <n> if auto-detection fails
        "context_length": 32768
      },
      // "dialect" selects the provider wire format — URL path, auth header, and
      // where the interesting fields sit in the SSE payload. Omit it (or leave
      // it null) for the OpenAI-compatible default, which is what every profile
      // written before this field existed gets; those keep working untouched.
      //   "openai" (default) — {base_url}/v1/chat/completions, Authorization: Bearer <key>
      //   "gemini"           — /v1beta/openai/chat/completions
      //   "azure"            — /openai/deployments/{model}/chat/completions, api-key: <key>
      // An unrecognized name logs a warning and falls back to "openai" rather
      // than failing, so a typo cannot take the agent offline.
      "gemini-pro": {
        "base_url": "https://generativelanguage.googleapis.com",
        "model": "gemini-3-pro",
        "api_key": "env:GEMINI_API_KEY",
        "dialect": "gemini"
      }
    }
  },

  // List of Extensions (Policy Layer) to register and launch. Extensions are
  // plain .wasm files at a path — there is no "builtin" scheme or directory
  // auto-discovery; `./scripts/build_all.sh` builds all 6 into `~/.rad/wasm/`,
  // which is a convenient install location to point `source` at (works from
  // any project directory, not just a source checkout).
  "extensions": [
    {
      "name": "rad-orchestrator",
      "source": "~/.rad/wasm/rad_orchestrator.wasm",
      "enabled": true,
      "role": "orchestrator",
      "permissions": {
        "fs_read_allow": ["*"],
        "fs_write_allow": ["*"],
        "execution": {
          "allow_bash": true,
          "allow_commands": [],
          "block_commands": []
        },
        "network": { "allow_network": true, "allow_domains": [] }
      }
    },
    {
      "name": "security-guard",
      "source": "~/.rad/wasm/security_guard.wasm",
      "enabled": true,
      "role": "security",
      "permissions": { "fs_read_allow": ["*"], "fs_write_allow": ["*"] },
      // Extension-specific settings, passed through opaquely by the Core (it
      // does not interpret them — only the Extension itself does). Empty or
      // omitted means the blocklist is opt-in and blocks nothing.
      "config": {
        "block_path_patterns": ["secrets.env"],
        "block_command_patterns": ["rm -rf /"]
      }
    }
  ],
  // Kernel modules. Unlike extensions these declare no role and no
  // permissions: a module reaches whatever the host preopens for it
  // (the working directory and $HOME), and what it *provides* comes from
  // the manifest it exports rather than from this file.
  "modules": [
    {
      "name": "context-tools",
      "source": "~/.rad/wasm/context_module.wasm",
      "enabled": true
    },
    {
      "name": "skills",
      "source": "~/.rad/wasm/skills_module.wasm",
      "enabled": true,
      // Opaque to the kernel; the module reads it via `kernel.config`.
      "config": {}
    },
    {
      // The LLM transport. Provider differences — URL path, auth header, where
      // the interesting fields sit in the SSE payload — are a compiled-in table
      // (`dialect` on an `llm.endpoints` profile selects a row), not settings
      // here. Was the `llm-connector` extension until v0.76.0.
      "name": "llm-openai",
      "source": "~/.rad/wasm/llm_openai_module.wasm",
      "enabled": true
    },
    {
      "name": "mcp",
      "source": "~/.rad/wasm/mcp_module.wasm",
      "enabled": true,
      // The only source of general-purpose tools rad has — any stdio MCP
      // server works here. `command` and `args` stay separate all the way to
      // the process: the extension used to join them into one string that the
      // host then split back apart on whitespace, mangling any argument
      // containing a space.
      "config": {
        "mcp_servers": {
          "core-utilities": { "command": "~/.cargo/bin/core-utilities-mcp", "args": [] },
          "web-access": { "command": "~/.cargo/bin/web-access-mcp", "args": [] }
        }
      }
    }
  ]
}
```

All four were extensions until recently. `context-tools` moved in AWU 957,
`skill-tool-provider` became the `skills` module in AWU 959/960,
`mcp-tool-provider` became `mcp` in AWU 964/965 — the latter also shed its
`allow_bash` requirement, which existed only because the old WIT made it return
results through `open_process("echo ...")` — and `llm-connector` became
`llm-openai` in AWU 967–969. A module returns a string, so a Markdown reader no
longer asks for shell execution.

### 1.3 Handling Sensitive Information
To handle API keys and other secrets securely, pass them to `rad` using the following methods:

1. **Environment Variables (Recommended)**:
   Set `RAD_API_KEY` (or `LLM_API_KEY`) in the launching shell; it's applied to the active `llm.endpoints` profile on every startup. Do not write credentials in configuration files.
2. **Local Configuration File (`rad.local.json`)**:
   In local development environments where you do not want to set environment variables, specify the key directly on the endpoint profile in `rad.local.json` to be merged on top of the project/global config:

**Example `rad.local.json`:**
```json
{
  "llm": {
    "endpoints": {
      "local": { "api_key": "sk-..." }
    }
  }
}
```

### 1.4 Optional Parameters & Defaults (Convention)
If settings are omitted from the config file, the Core automatically applies the following default parameters:

| Setting | Default Value | Description |
| :--- | :--- | :--- |
| `core.workspace_dir` | `.` (Current directory) | The physical root path where the agent operates. |
| `core.snapshot_dir` | `.rad/snapshots` | The directory where filesystem snapshots are saved/restored. |
| `core.log_dir` | `.rad/logs` | The directory reserved for operational and session logs. |
| `core.max_sessions` | `50` | Session files beyond this count (oldest first) are pruned from `.rad/sessions/` at startup; the active session is always kept. |
| `default_timeout.llm_stream_heartbeat_ms` | `15000` (15s) | The maximum allowed interval between received tokens during LLM streaming. |
| `default_timeout.process_silent_timeout_ms` | `60000` (60s) | The maximum idle duration for a process spawned via `spawn_bash_process` before timing out. |
| `extensions[].permissions` | Deny all if omitted | Extensions default to denying all filesystem/execution/network actions unless explicitly granted. |
| `extensions[].config` | Empty object (`{}`) | Extension-specific configuration passed straight through to the Extension, uninterpreted by the Core. |

---

## 2. Directory Layout

Data `rad` reads or writes lives under `.rad/` (project-local) and `~/.rad/` (user-global).

### 2.1 Project Local (`.rad/`)
```text
<Project Root>/
├── rad.json                   # [Recommended] Project-local configuration file
├── rad.local.json             # Local-only overrides/secrets (gitignore this)
├── .agents/
│   ├── AGENTS.md              # Project-specific rules appended to the system prompt
│   ├── commands/               # Custom slash commands (see §2.3)
│   └── skills/                 # Skills, one directory per skill (see §2.5)
│       └── <name>/SKILL.md
└── .rad/                      # Project-local data storage
    ├── config.json            # [Alternative] Hidden project-local configuration file
    ├── snapshots/              # Filesystem snapshots, partitioned by DAG node_id
    │   └── <node_id>/
    ├── sessions/               # Saved session DAGs (<session_id>.json), pruned by core.max_sessions
    └── logs/                  # Reserved for operational/session log output
```

### 2.2 User Global (`~/.rad/`)
```text
~/.rad/                        # User-global data storage
├── config.json                 # User-global configuration (base of the precedence cascade)
├── wasm/                       # Convenient install location for built extensions
│   └── rad_orchestrator.wasm   # (populated by ./scripts/build_all.sh)
├── commands/                   # Custom slash commands shared across all projects (see §2.3)
└── skills/                     # Skills shared across all projects (see §2.5)
    └── <name>/SKILL.md
```

### 2.3 Custom Slash Commands
Markdown files under `.agents/commands/` (project-local, checked first) or `~/.rad/commands/` (user-global) become slash commands automatically — `.agents/commands/review.md` becomes `/review`. The file's content is sent as the task prompt; a literal `$ARGUMENTS` placeholder is substituted with whatever followed the command name, or the arguments are appended on their own line if no placeholder is present. No code or registration step is required — just drop a `.md` file in either directory.

### 2.4 Project Rules (`AGENTS.md`)
`.agents/AGENTS.md` or `AGENTS.md` at the project root, if present, is appended to the system prompt on every turn — use it for project-specific conventions, build commands, or constraints the agent should always know about.

### 2.5 Skills
A skill is a directory containing a `SKILL.md` under `.agents/skills/<name>/` (project-local, checked first) or `~/.rad/skills/<name>/` (user-global) — same precedence direction as custom slash commands. Unlike commands, skills aren't invoked by typing `/name`: the `skills` kernel module offers the model a **single `skill(name, args?)` tool** whose description lists every skill it found, one line each, with that skill's own `description`. The model picks one by name, the same way it decides to call any other tool.

This used to be one tool *per* skill. A tool schema costs roughly 468 characters, so ten skills spent about 4,700 characters of every prompt on entries differing only in a name and a line of prose; the index does it in under 1,000. What is *not* traded away is progressive disclosure — a skill's body is still read only when it runs.

`SKILL.md` starts with a `---`-delimited frontmatter block, then the body sent as the tool result when the skill is invoked:
```
---
description: Runs the team's PR review checklist against currently staged changes.
---

Check that the diff includes tests and an updated changelog entry.
```
- `description` (required): shown to the model in the tool list. A skill missing this is skipped.
- `mode`: **removed.** It only ever accepted `inline`, with `subagent` reserved for a nested-task mode that returned "not yet implemented". Subagents were dropped as a goal, so the field went with them. An existing `SKILL.md` that still carries a `mode:` line is not rejected — the line is ignored and the skill runs.
- `allowed_tools` (optional): reserved for a future access-scoping mechanism, currently parsed but not enforced. Enforcement is planned as the `policy` module's job rather than this one's, since what needs constraining is the model's choice, not the module's behaviour.

Callers can pass an optional `args` string when invoking a skill (`skill(name: "review-pr", args: "the staged diff")`); if the body contains a literal `$ARGUMENTS` placeholder it's substituted, otherwise `args` is appended on its own line (same substitution rule as custom slash commands' `$ARGUMENTS`).

---

## 3. Updating Core & Extensions

`rad` requires no system-level administrative privileges and runs entirely within user space.

* **Core Update**: Pull the latest source and re-run `./scripts/build_all.sh` — it rebuilds every extension and kernel module in the workspace (the list is derived from `cargo metadata`, not maintained by hand), runs the test/Clippy gates, and reinstalls the `rad` binary to `~/.cargo/bin/rad` via `cargo install --path .`.
* **Extension Update**: Overwrite the extension's `.wasm` file at its configured `source` path, then run `/reload` inside a running session (or just restart `rad`) — `/reload` re-reads the config and clears the cached WASM runtimes so the next task picks up the new binary.

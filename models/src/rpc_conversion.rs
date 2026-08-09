//! Shared `macro_rules!` definitions that generate the WIT ↔ `RasRpcCommand`
//! (and `Target`/`TimeoutPolicy`) conversion boilerplate, so each crate that
//! previously hand-duplicated this match invokes one shared definition instead
//! of hand-copying ~25 match arms per direction. Four crates did when this was
//! written; the migration has taken it down to two — `rad`'s host bindings and
//! `rad-orchestrator` — as `security-guard` and `mcp-tool-provider` became
//! modules, which cross no WIT boundary and so need no conversion at all.
//!
//! Each crate's `wit_bindgen::generate!` produces its own local `RasRpcCommand`
//! type (structurally identical, since all worlds import the same `rad.wit`
//! `types` interface, but nominally distinct per crate) — hence these are
//! declarative macros taking the invoking crate's `wit` module path as a
//! parameter, rather than a single `impl` written once here.
//!
//! `core-to-wit` is split from `wit-to-core`: `RasRpcCommand::OpenFile`/
//! `OpenProcess` exist only on the core side (normalized into `FileRead`/
//! `FileWrite`/`SpawnBashProcess` before ever reaching a guest), so no
//! guest-side `wit` variant exists for them. The host converts them for
//! real; guests never construct them and are expected to handle that
//! residual themselves (typically `unreachable!()`/`panic!()`), so the
//! core-to-wit macro returns `Option` and leaves that residual to the caller.
//!
//! The two large command-level macros live in companion files
//! (`wit_to_core.rs`, `core_to_wit.rs`) to stay under the 300-line file
//! limit — `#[macro_export]` always exports at the crate root regardless
//! of which module a macro is textually defined in, so callers are
//! unaffected by the split.
mod core_to_wit;
mod wit_to_core;

/// Generates `From<$wit::Target> for $crate::Target`.
#[macro_export]
macro_rules! impl_rpc_target_wit_to_core {
    ($wit:ident) => {
        impl From<$wit::Target> for $crate::Target {
            fn from(t: $wit::Target) -> Self {
                match t {
                    $wit::Target::Llm => $crate::Target::Llm,
                    $wit::Target::Process(p) => $crate::Target::Process(p.to_string()),
                }
            }
        }
    };
}

/// Generates `From<$crate::Target> for $wit::Target`.
#[macro_export]
macro_rules! impl_rpc_target_core_to_wit {
    ($wit:ident) => {
        impl From<$crate::Target> for $wit::Target {
            fn from(t: $crate::Target) -> Self {
                match t {
                    $crate::Target::Llm => $wit::Target::Llm,
                    $crate::Target::Process(p) => $wit::Target::Process(p.parse().unwrap_or(0)),
                }
            }
        }
    };
}

/// Generates `From<$wit::TimeoutPolicy> for $crate::TimeoutPolicy`.
#[macro_export]
macro_rules! impl_rpc_timeout_policy_wit_to_core {
    ($wit:ident) => {
        impl From<$wit::TimeoutPolicy> for $crate::TimeoutPolicy {
            fn from(tp: $wit::TimeoutPolicy) -> Self {
                match tp {
                    $wit::TimeoutPolicy::Dynamic(p) => $crate::TimeoutPolicy::Dynamic {
                        heartbeat_timeout_ms: p.heartbeat_timeout_ms,
                        max_silent_wait_ms: p.max_silent_wait_ms,
                    },
                    $wit::TimeoutPolicy::Infinite => $crate::TimeoutPolicy::Infinite,
                }
            }
        }
    };
}

/// Generates `From<$crate::TimeoutPolicy> for $wit::TimeoutPolicy`.
#[macro_export]
macro_rules! impl_rpc_timeout_policy_core_to_wit {
    ($wit:ident) => {
        impl From<$crate::TimeoutPolicy> for $wit::TimeoutPolicy {
            fn from(tp: $crate::TimeoutPolicy) -> Self {
                match tp {
                    $crate::TimeoutPolicy::Dynamic {
                        heartbeat_timeout_ms,
                        max_silent_wait_ms,
                    } => $wit::TimeoutPolicy::Dynamic($wit::DynamicPolicy {
                        heartbeat_timeout_ms,
                        max_silent_wait_ms,
                    }),
                    $crate::TimeoutPolicy::Infinite => $wit::TimeoutPolicy::Infinite,
                }
            }
        }
    };
}

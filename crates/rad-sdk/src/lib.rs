//! Guest-side SDK. Everything a module author writes goes through here.
//!
//! ```ignore
//! rad_sdk::module! {
//!     wit:     "../../wit/kernel/kernel.wit",
//!     name:    "context",
//!     version: "0.3.1",
//!     methods: {
//!         "context.optimize" => optimize,
//!         "context.digest"   => digest,
//!     }
//! }
//!
//! fn optimize(req: OptimizeReq) -> Result<OptimizeRes, rad_sdk::Error> { .. }
//! ```
//!
//! The macro generates `manifest()` and `handle()` and wires the `export!`.
//! `provides` is derived from the method map, so a manifest cannot fall out of
//! step with what the module actually answers — the one drift that breaks
//! routing silently at startup.
//!
//! Declarative rather than a `#[rad::module]` attribute on purpose: an
//! attribute reads better but costs a proc-macro crate and a `syn` dependency
//! in every module's build, and this form already does the job.

pub mod error;

pub use error::Error;
pub use rad_abi::{ABI_VERSION, Manifest};

// Re-exported so generated code can name them without the module author having
// to add either as a direct dependency.
#[doc(hidden)]
pub mod __private {
    pub use rad_abi::{ABI_VERSION, Manifest};
    pub use serde_json;
}

/// Generates `manifest()` and `handle()` for a module's method map.
///
/// Split out of [`module!`] so the routing logic can be tested natively —
/// `module!` also emits wasm component bindings, which cannot link off-target.
/// Module authors call `module!`; this is what it builds on.
///
/// Method handlers take any `serde::Deserialize` request and return
/// `Result<T, Error>` where `T` is `serde::Serialize`.
#[macro_export]
macro_rules! routes {
    (
        name: $name:expr,
        version: $version:expr,
        methods: { $($method:expr => $handler:expr),+ $(,)? } $(,)?
    ) => {
        #[doc(hidden)]
        pub struct __RadModule;

        impl __RadModule {
            /// The manifest, with `provides` derived from the method map above
            /// so the two cannot disagree.
            pub fn manifest() -> String {
                let manifest = $crate::__private::Manifest {
                    name: ($name).to_string(),
                    version: ($version).to_string(),
                    abi: $crate::__private::ABI_VERSION.to_string(),
                    provides: vec![$(($method).to_string()),+],
                };
                // A manifest of literals cannot fail to serialize; if it somehow
                // does, an empty string makes the kernel reject the module with
                // a parse error rather than the guest trapping mid-startup.
                $crate::__private::serde_json::to_string(&manifest).unwrap_or_default()
            }

            /// Routes one inbound message. Unknown methods are reported rather
            /// than trapping: the kernel is expected to keep going and tell the
            /// caller the method is unsupported.
            pub fn handle(method: String, payload: String) -> Result<String, String> {
                match method.as_str() {
                    $(
                        $method => $crate::__dispatch_to(&payload, $handler),
                    )+
                    other => Err(format!(
                        concat!("module '", $name, "' does not provide method '{}'"),
                        other
                    )),
                }
            }
        }

    };
}

/// Defines a module: [`routes!`] plus the WIT bindings and component export.
///
/// `$wit` is a path to the single `wit/kernel/kernel.wit`, relative to the
/// module's `Cargo.toml` — never a copy of it. `templates/` drifted three times
/// precisely because it held copies.
///
/// The bindings are generated in the *module's* crate rather than the SDK's:
/// wit-bindgen emits `#[export_name]` shims that edition 2024 requires to be
/// spelled `#[unsafe(export_name)]`, and when `export!` crosses a crate
/// boundary those shims land where nothing can accept them — a hard error, not
/// a lint. Every existing extension generates in place for the same reason.
#[macro_export]
macro_rules! module {
    (
        wit: $wit:expr,
        name: $name:expr,
        version: $version:expr,
        methods: { $($method:expr => $handler:expr),+ $(,)? } $(,)?
    ) => {
        $crate::routes! {
            name: $name,
            version: $version,
            methods: { $($method => $handler),+ }
        }

        #[doc(hidden)]
        #[allow(
            unsafe_op_in_unsafe_fn,
            clippy::same_length_and_capacity,
            clippy::pedantic
        )]
        mod __rad_bindings {
            wit_bindgen::generate!({
                path: $wit,
                world: "module",
            });

            impl Guest for super::__RadModule {
                fn manifest() -> String {
                    super::__RadModule::manifest()
                }
                fn handle(method: String, payload: String) -> Result<String, String> {
                    super::__RadModule::handle(method, payload)
                }
            }

            use super::__RadModule;
            export!(__RadModule);
        }

        /// The kernel's imports, re-exported at the module's crate root so a
        /// module author writes `crate::syscall::proc_spawn(..)` rather than
        /// reaching into generated-binding paths.
        pub use __rad_bindings::rad::kernel::{dispatch, syscall, types};
    };
}

/// Adapts a handler that cannot fail.
///
/// Every handler returns `Result` because the dispatch boundary needs one, but
/// plenty of them have no failure case — listing what is on disk, formatting a
/// value. Writing `Ok(..)` anyway makes clippy's `unnecessary_wraps` fire in
/// the module author's own crate, and the alternatives are both bad: allow the
/// lint (CODING.md §1 permits that only for generated code) or invent an error
/// that cannot happen.
///
/// ```ignore
/// methods: {
///     "skills.tools.list" => rad_sdk::infallible(list),
/// }
///
/// fn list(req: ListReq) -> ListRes { .. }
/// ```
pub fn infallible<Req, Res, F>(handler: F) -> impl FnOnce(Req) -> Result<Res, Error>
where
    F: FnOnce(Req) -> Res,
{
    move |req| Ok(handler(req))
}

/// Deserializes a request, runs a handler, serializes the response.
///
/// Extracted from the macro so the serde work is type-checked once here rather
/// than re-expanded, and so error messages point at this function instead of
/// inside a macro expansion.
///
/// # Errors
///
/// Returns a message if the payload does not deserialize, the handler fails, or
/// the response does not serialize.
pub fn __dispatch_to<Req, Res, F>(payload: &str, handler: F) -> Result<String, String>
where
    Req: serde::de::DeserializeOwned,
    Res: serde::Serialize,
    F: FnOnce(Req) -> Result<Res, Error>,
{
    let request: Req = serde_json::from_str(payload)
        .map_err(|e| format!("request payload did not match the expected shape: {e}"))?;
    let response = handler(request).map_err(|e| e.to_string())?;
    serde_json::to_string(&response).map_err(|e| format!("response failed to serialize: {e}"))
}

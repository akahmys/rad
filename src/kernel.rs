//! The kernel surface, live alongside the existing extension host.
//!
//! Deliberately a separate module tree from `src/wasm/`: during the migration
//! both surfaces run in the same process, and keeping them apart is what makes
//! the final step — deleting the old one — a deletion rather than a
//! disentangling (`ARCHITECTURE-NEXT.md` §9.1).

mod host;
mod loader;
mod registry;

#[cfg(test)]
mod tests;

pub use host::KernelState;
pub use loader::ModuleRuntime;
pub use registry::{Registry, RegistryError};

//! The kernel surface, live alongside the existing extension host.
//!
//! Deliberately a separate module tree from `src/wasm/`: during the migration
//! both surfaces run in the same process, and keeping them apart is what makes
//! the final step — deleting the old one — a deletion rather than a
//! disentangling (`ARCHITECTURE-NEXT.md` §9.1).

mod bootstrap;
mod host;
mod loader;
mod net;
mod posts;
pub(crate) mod proc;
mod registry;
mod shared;
pub(crate) mod stream;
pub mod tools;

#[cfg(test)]
mod tests;

pub use bootstrap::boot;
pub use host::KernelState;
pub use loader::ModuleRuntime;
pub use posts::Posted;
pub use registry::{Registry, RegistryError};
pub use shared::{KERNEL_TARGET, KernelShared};

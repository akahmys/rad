//! The routing table: which module answers which method.
//!
//! Built from each module's `manifest().provides` at load time, before anything
//! runs. Deliberately has no knowledge of what a method *does* — dispatch is
//! opaque, so routing is a string lookup and nothing more.

use rad_abi::Manifest;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// Two modules claim the same method. A startup error rather than an
    /// implicit first-wins: which one serves a call must not depend on the
    /// order entries happen to appear in a config file
    /// (`ARCHITECTURE-NEXT.md` §3.6.8).
    DuplicateMethod {
        method: String,
        existing: String,
        incoming: String,
    },
    /// Two modules share a name, so `dispatch` could not address them apart.
    DuplicateModule(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateMethod {
                method,
                existing,
                incoming,
            } => write!(
                f,
                "method '{method}' is provided by both '{existing}' and '{incoming}'; \
                 remove one of them or rename the method"
            ),
            Self::DuplicateModule(name) => {
                write!(f, "two modules are both named '{name}'")
            }
        }
    }
}

impl std::error::Error for RegistryError {}

/// Maps method names to the module that answers them.
#[derive(Debug, Default)]
pub struct Registry {
    /// method -> module name
    routes: HashMap<String, String>,
    /// module name -> its manifest, kept for diagnostics and `/tools`-style
    /// introspection.
    modules: HashMap<String, Manifest>,
}

impl Registry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a module's routes.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] if the module name is already taken or any of
    /// its methods is already provided elsewhere. On error the registry is left
    /// unchanged, so a rejected module cannot half-register.
    pub fn register(&mut self, manifest: Manifest) -> Result<(), RegistryError> {
        if let Some(existing) = self.modules.get(&manifest.name) {
            return Err(RegistryError::DuplicateModule(existing.name.clone()));
        }
        // Check every method before inserting any, so a conflict halfway
        // through does not leave the earlier ones routed to a module that was
        // then rejected.
        for method in &manifest.provides {
            if let Some(existing) = self.routes.get(method) {
                return Err(RegistryError::DuplicateMethod {
                    method: method.clone(),
                    existing: existing.clone(),
                    incoming: manifest.name.clone(),
                });
            }
        }
        for method in &manifest.provides {
            self.routes.insert(method.clone(), manifest.name.clone());
        }
        self.modules.insert(manifest.name.clone(), manifest);
        Ok(())
    }

    /// Which module answers `method`, if any.
    #[must_use]
    pub fn route(&self, method: &str) -> Option<&str> {
        self.routes.get(method).map(String::as_str)
    }

    #[must_use]
    pub fn manifest(&self, module: &str) -> Option<&Manifest> {
        self.modules.get(module)
    }

    #[must_use]
    pub fn module_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.modules.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }
}

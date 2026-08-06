//! The single contract shared between the kernel and every module.
//!
//! Dispatch is opaque — `call(target, method, payload)` moves strings, and the
//! kernel never looks inside a payload. So the only thing both sides must agree
//! on is the manifest: a module emits it, and the kernel reads `provides` to
//! build its routing table. If those two drift, routing breaks silently at
//! startup, which is exactly the class of failure worth a shared type.
//!
//! Payload types for specific methods deliberately do *not* live here. They are
//! agreements between two modules, not between a module and the kernel, and
//! adding them before a second consumer exists would be inventing a shared
//! contract for one party (see `CODING.md` §3).

#[cfg(test)]
mod tests;

/// The ABI version a module was built against. The kernel refuses to load a
/// module declaring anything else.
///
/// This buys a clear error message, not safety: opaque dispatch means the WIT
/// itself does not change, so protocol evolution cannot produce a link failure.
/// What it catches is a module requiring a syscall an older kernel lacks —
/// without it, that surfaces as a linker error naming an internal symbol.
pub const ABI_VERSION: &str = "1.0";

/// What a module tells the kernel about itself, returned as JSON from the
/// `manifest` export.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    /// Must equal [`ABI_VERSION`]; see [`Manifest::check_abi`].
    pub abi: String,
    /// Fully qualified method names this module answers, e.g.
    /// `"context.optimize"`. The kernel routes on these, and two modules
    /// claiming the same name is a startup error rather than an implicit
    /// first-wins — which of them served a call must not depend on load order.
    #[serde(default)]
    pub provides: Vec<String>,
}

/// Why a manifest was rejected. A plain enum rather than a `thiserror` type:
/// this crate has three failure modes and no callers that need to match on
/// their internals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// The `manifest` export returned something that is not a `Manifest`.
    Malformed(String),
    /// Built against a different ABI than this kernel implements.
    AbiMismatch { expected: String, found: String },
    /// A required field was present but empty, which would break routing or
    /// diagnostics in ways that are hard to trace back here.
    Empty(&'static str),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed(e) => write!(f, "manifest is not valid JSON for a Manifest: {e}"),
            Self::AbiMismatch { expected, found } => write!(
                f,
                "module was built against ABI {found}, this kernel implements {expected}"
            ),
            Self::Empty(field) => write!(f, "manifest field '{field}' must not be empty"),
        }
    }
}

impl std::error::Error for ManifestError {}

impl Manifest {
    /// Parses and validates a manifest as returned by a module.
    ///
    /// # Errors
    ///
    /// Returns [`ManifestError`] if the JSON does not describe a `Manifest`, if
    /// the ABI does not match, or if `name`/`version` are empty.
    pub fn parse(json: &str) -> Result<Self, ManifestError> {
        let manifest: Self =
            serde_json::from_str(json).map_err(|e| ManifestError::Malformed(e.to_string()))?;
        manifest.check_abi()?;
        if manifest.name.trim().is_empty() {
            return Err(ManifestError::Empty("name"));
        }
        if manifest.version.trim().is_empty() {
            return Err(ManifestError::Empty("version"));
        }
        Ok(manifest)
    }

    /// # Errors
    ///
    /// Returns [`ManifestError::AbiMismatch`] if `abi` is not [`ABI_VERSION`].
    pub fn check_abi(&self) -> Result<(), ManifestError> {
        if self.abi == ABI_VERSION {
            Ok(())
        } else {
            Err(ManifestError::AbiMismatch {
                expected: ABI_VERSION.to_string(),
                found: self.abi.clone(),
            })
        }
    }

    /// Serializes for the `manifest` export.
    ///
    /// # Errors
    ///
    /// Returns an error only if serialization fails, which for this shape means
    /// an allocation failure.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }
}

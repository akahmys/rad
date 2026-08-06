//! The error a module handler returns.
//!
//! Dispatch carries `result<string, string>` — the wire has no structure to
//! preserve, so this type exists to give module authors something better than
//! `String` to construct and match on, not to survive the boundary intact.

/// What went wrong inside a module handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The request was structurally fine but semantically wrong — an unknown
    /// id, an out-of-range value, a path that does not exist.
    Invalid(String),
    /// A syscall or a dispatch to another module failed.
    Io(String),
    /// The module cannot do this at all: an unimplemented mode, a missing
    /// dependency, a capability it does not have.
    Unsupported(String),
}

impl Error {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid(message.into())
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::Io(message.into())
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::Unsupported(message.into())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The variant name is kept in the text: once this crosses dispatch it is
        // an opaque string, and "Invalid: ..." is the only thing telling the
        // caller which kind of failure it was.
        match self {
            Self::Invalid(m) => write!(f, "Invalid: {m}"),
            Self::Io(m) => write!(f, "Io: {m}"),
            Self::Unsupported(m) => write!(f, "Unsupported: {m}"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests;

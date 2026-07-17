use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorLevel {
    #[serde(rename = "L1")]
    L1,
    #[serde(rename = "L2")]
    L2,
    #[serde(rename = "L3")]
    L3,
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[serde(tag = "level", content = "payload")]
pub enum UnifiedError {
    #[error("L1 (Adaptation) Error: {message} ({category})")]
    L1 { message: String, category: String },

    #[error("L2 (Rollback) Error: {message} (rollback to: {rollback_target})")]
    L2 {
        message: String,
        rollback_target: String,
    },

    #[error("L3 (Reset) Error: {message} (tokens: {prompt_tokens:?}/{limit:?})")]
    L3 {
        message: String,
        prompt_tokens: Option<u32>,
        limit: Option<u32>,
    },
}

impl UnifiedError {
    pub fn error_level(&self) -> ErrorLevel {
        match self {
            UnifiedError::L1 { .. } => ErrorLevel::L1,
            UnifiedError::L2 { .. } => ErrorLevel::L2,
            UnifiedError::L3 { .. } => ErrorLevel::L3,
        }
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|e| format!("{{\"level\":\"L1\",\"payload\":{{\"message\":\"Serialization failed: {e}\",\"category\":\"Internal\"}}}}"))
    }

    pub fn from_json_string(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }

    pub fn l1(message: impl Into<String>, category: impl Into<String>) -> Self {
        UnifiedError::L1 {
            message: message.into(),
            category: category.into(),
        }
    }

    pub fn l2(message: impl Into<String>, rollback_target: impl Into<String>) -> Self {
        UnifiedError::L2 {
            message: message.into(),
            rollback_target: rollback_target.into(),
        }
    }

    pub fn l3(message: impl Into<String>, prompt_tokens: Option<u32>, limit: Option<u32>) -> Self {
        UnifiedError::L3 {
            message: message.into(),
            prompt_tokens,
            limit,
        }
    }
}

impl From<UnifiedError> for String {
    fn from(err: UnifiedError) -> Self {
        err.to_json_string()
    }
}

#[cfg(test)]
mod tests;

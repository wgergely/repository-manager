//! PresetProvider trait and related types

use crate::Result;
use crate::context::Context;
use async_trait::async_trait;

/// Status of a preset after checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetStatus {
    Healthy,
    Missing,
    Drifted,
    Broken,
}

/// Remedial action needed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    None,
    Install,
    Repair,
    Update,
}

/// Report from checking a preset
#[derive(Debug, Clone)]
pub struct PresetCheckReport {
    pub status: PresetStatus,
    pub details: Vec<String>,
    pub action: ActionType,
}

impl PresetCheckReport {
    pub fn healthy() -> Self {
        Self {
            status: PresetStatus::Healthy,
            details: vec![],
            action: ActionType::None,
        }
    }

    pub fn missing(detail: impl Into<String>) -> Self {
        Self {
            status: PresetStatus::Missing,
            details: vec![detail.into()],
            action: ActionType::Install,
        }
    }

    pub fn drifted(detail: impl Into<String>) -> Self {
        Self {
            status: PresetStatus::Drifted,
            details: vec![detail.into()],
            action: ActionType::Repair,
        }
    }

    pub fn broken(detail: impl Into<String>) -> Self {
        Self {
            status: PresetStatus::Broken,
            details: vec![detail.into()],
            action: ActionType::Install,
        }
    }
}

/// Status of a preset apply operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyStatus {
    /// The provider performed real setup actions successfully.
    Success,
    /// The provider checked the environment but does not perform setup.
    /// Callers should not treat this as a real apply success.
    DetectionOnly,
    /// The provider attempted setup but it failed.
    Failed,
}

/// Report from applying a preset
#[derive(Debug, Clone)]
pub struct ApplyReport {
    pub status: ApplyStatus,
    pub actions_taken: Vec<String>,
    pub errors: Vec<String>,
}

impl ApplyReport {
    pub fn success(actions: Vec<String>) -> Self {
        Self {
            status: ApplyStatus::Success,
            actions_taken: actions,
            errors: vec![],
        }
    }

    pub fn detection_only(messages: Vec<String>) -> Self {
        Self {
            status: ApplyStatus::DetectionOnly,
            actions_taken: messages,
            errors: vec![],
        }
    }

    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            status: ApplyStatus::Failed,
            actions_taken: vec![],
            errors,
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self.status, ApplyStatus::Success)
    }

    pub fn is_detection_only(&self) -> bool {
        matches!(self.status, ApplyStatus::DetectionOnly)
    }

    pub fn is_failure(&self) -> bool {
        matches!(self.status, ApplyStatus::Failed)
    }
}

/// Core trait for preset providers
#[async_trait]
pub trait PresetProvider: Send + Sync {
    fn id(&self) -> &str;
    async fn check(&self, context: &Context) -> Result<PresetCheckReport>;
    async fn apply(&self, context: &Context) -> Result<ApplyReport>;
}

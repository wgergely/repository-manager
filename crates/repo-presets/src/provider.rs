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
pub struct CheckReport {
    pub status: PresetStatus,
    pub details: Vec<String>,
    pub action: ActionType,
}

impl CheckReport {
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
}

/// Report from applying a preset
#[derive(Debug, Clone)]
pub struct ApplyReport {
    pub success: bool,
    pub actions_taken: Vec<String>,
    pub errors: Vec<String>,
}

impl ApplyReport {
    pub fn success(actions: Vec<String>) -> Self {
        Self {
            success: true,
            actions_taken: actions,
            errors: vec![],
        }
    }

    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            success: false,
            actions_taken: vec![],
            errors,
        }
    }
}

/// Core trait for preset providers
#[async_trait]
pub trait PresetProvider: Send + Sync {
    fn id(&self) -> &str;
    async fn check(&self, context: &Context) -> Result<CheckReport>;
    async fn apply(&self, context: &Context) -> Result<ApplyReport>;
}

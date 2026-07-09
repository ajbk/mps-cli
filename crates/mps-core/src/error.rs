use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum MpsError {
    #[error("Unsupported movement experience: {0}")]
    UnsupportedMovementExperience(String),

    #[error("Unsupported level: {0}")]
    UnsupportedLevel(String),

    #[error("Unsupported duration: {requested}. MVP supports 45, 60, and 75 minutes.")]
    UnsupportedDuration { requested: u32 },

    #[error("Unsupported equipment: {0}")]
    UnsupportedEquipment(String),

    #[error("Unsupported risk policy: {0}")]
    UnsupportedRiskPolicy(String),

    #[error("No candidates available for phase {phase} and role {role}")]
    NoCandidates { phase: String, role: String },

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Generation failed: {0}")]
    Generation(String),
}

pub type MpsResult<T> = Result<T, MpsError>;

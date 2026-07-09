pub mod class_plan;
pub mod class_request;
pub mod domain;
pub mod error;
pub mod markdown;
pub mod phase_allocation;
pub mod repository;
pub mod scoring;
pub mod strategy;

pub use class_plan::*;
pub use class_request::{ClassRequest, GroupSafety};
pub use domain::*;
pub use error::{MpsError, MpsResult};
pub use markdown::render_markdown;
pub use phase_allocation::{phase_allocations_for_duration, total_minutes, PhaseAllocation};
pub use repository::*;
pub use scoring::*;
pub use strategy::build_strategy;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub mod class_request;
pub mod domain;
pub mod error;

pub use class_request::{ClassRequest, GroupSafety};
pub use domain::*;
pub use error::{MpsError, MpsResult};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

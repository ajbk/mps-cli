pub mod domain;
pub mod error;

pub use domain::*;
pub use error::{MpsError, MpsResult};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

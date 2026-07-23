pub mod flashcard_repository;
pub mod sqlite_repository;

pub use flashcard_repository::*;
pub use sqlite_repository::SqliteRepository;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

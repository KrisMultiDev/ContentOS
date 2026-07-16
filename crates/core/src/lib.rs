pub mod activity;
pub mod backup;
pub mod batches;
pub mod components;
pub mod davinci;
pub mod db;
pub mod library;
pub mod media;
pub mod posts;
pub mod ideas;
pub mod ids;
pub mod jobs;
pub mod pillars;
pub mod reels;
pub mod roots;
pub mod search;
pub mod settings;

pub use db::Db;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Invalid(String),
    #[error("not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;

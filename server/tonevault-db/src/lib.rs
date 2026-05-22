pub mod repository;

#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(feature = "sqlite")]
pub use sqlite::SqliteRepository;

#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "postgres")]
pub use postgres::PostgresRepository;

pub use repository::Repository;
pub use sqlx;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Not found")]
    NotFound,
    #[error("{0}")]
    Other(#[from] sqlx::Error),
}

pub type Result<T> = std::result::Result<T, DbError>;

#[derive(Clone)]
pub struct Database {
    pool: sqlx::SqlitePool,
}

impl Database {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &sqlx::SqlitePool {
        &self.pool
    }
}

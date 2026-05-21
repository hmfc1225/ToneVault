use crate::{Database, Result};
use tonevault_core::models::book::Book;
use tonevault_core::models::library::{CreateLibrary, Library, UpdateLibrary};

impl Database {
    pub async fn create_library_with_scan(
        &self,
        input: &CreateLibrary,
    ) -> Result<Library> {
        let library = self.create_library(input).await?;
        Ok(library)
    }

    pub async fn search_books(
        &self,
        library_id: Option<i64>,
        query: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Book>> {
        let pattern = format!("%{}%", query);
        if let Some(lid) = library_id {
            sqlx::query_as::<_, Book>(
                "SELECT * FROM books WHERE library_id = ? AND (title LIKE ? OR author LIKE ?) ORDER BY title LIMIT ? OFFSET ?",
            )
            .bind(lid)
            .bind(&pattern)
            .bind(&pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
        } else {
            sqlx::query_as::<_, Book>(
                "SELECT * FROM books WHERE title LIKE ? OR author LIKE ? ORDER BY title LIMIT ? OFFSET ?",
            )
            .bind(&pattern)
            .bind(&pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
        }
    }
}

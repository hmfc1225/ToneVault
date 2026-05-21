use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: Uuid,
    pub library_id: Uuid,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub cover_path: Option<String>,
    pub duration_secs: f64,
    pub track_count: i32,
    pub file_size: i64,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub publish_year: Option<i32>,
    pub isbn: Option<String>,
    pub asin: Option<String>,
    pub source_path: String,
    pub metadata_source: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: Uuid,
    pub book_id: Uuid,
    pub title: String,
    pub disc_number: Option<i32>,
    pub track_number: i32,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub duration_secs: f64,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub channels: Option<i32>,
    pub codec: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookAuthor {
    pub book_id: Uuid,
    pub author_id: Uuid,
    pub role: AuthorRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthorRole {
    Author,
    Narrator,
    Translator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookSeries {
    pub book_id: Uuid,
    pub series_id: Uuid,
    pub position: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBook {
    pub library_id: Uuid,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub cover_path: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub publish_year: Option<i32>,
    pub isbn: Option<String>,
    pub asin: Option<String>,
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateBook {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub cover_path: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub publish_year: Option<i32>,
    pub isbn: Option<String>,
    pub asin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookFilter {
    pub library_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub series_id: Option<Uuid>,
    pub query: Option<String>,
    pub sort: BookSort,
    pub order: SortOrder,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BookSort {
    #[default]
    Title,
    Added,
    Duration,
    Year,
    Author,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

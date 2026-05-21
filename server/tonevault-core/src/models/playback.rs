use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackPosition {
    pub id: Uuid,
    pub user_id: Uuid,
    pub book_id: Uuid,
    pub track_id: Uuid,
    pub position_secs: f64,
    pub duration_secs: f64,
    pub percentage: f64,
    pub is_finished: bool,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertPlaybackPosition {
    pub book_id: Uuid,
    pub track_id: Uuid,
    pub position_secs: f64,
    pub duration_secs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: Uuid,
    pub user_id: Uuid,
    pub book_id: Uuid,
    pub track_id: Uuid,
    pub title: String,
    pub note: Option<String>,
    pub position_secs: f64,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBookmark {
    pub book_id: Uuid,
    pub track_id: Uuid,
    pub title: String,
    pub note: Option<String>,
    pub position_secs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBookmark {
    pub title: Option<String>,
    pub note: Option<String>,
}

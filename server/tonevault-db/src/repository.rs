use async_trait::async_trait;
use tonevault_core::models::*;
use uuid::Uuid;

use crate::{Database, Result};

#[async_trait]
pub trait Repository: Send + Sync {
    // Library
    async fn list_libraries(&self) -> Result<Vec<Library>>;
    async fn get_library(&self, id: i64) -> Result<Option<Library>>;
    async fn create_library(&self, input: &CreateLibrary) -> Result<Library>;
    async fn update_library(&self, id: i64, input: &UpdateLibrary) -> Result<Library>;
    async fn delete_library(&self, id: i64) -> Result<()>;
    async fn update_last_scan(&self, id: i64) -> Result<()>;

    // Book
    async fn list_books(&self, filter: &BookFilter) -> Result<PaginatedResult<Book>>;
    async fn get_book(&self, id: Uuid) -> Result<Option<Book>>;
    async fn update_book(&self, id: Uuid, input: &UpdateBook) -> Result<Book>;
    async fn delete_book(&self, id: Uuid) -> Result<()>;
    async fn search_books(&self, query: &str, limit: i64) -> Result<Vec<Book>>;
    async fn get_book_authors(&self, book_id: Uuid) -> Result<Vec<(Author, AuthorRole)>>;
    async fn get_book_series(&self, book_id: Uuid) -> Result<Vec<(Series, Option<f64>)>>;

    // Track
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>>;
    async fn get_tracks_by_book(&self, book_id: Uuid) -> Result<Vec<Track>>;

    // Author
    async fn list_authors(&self) -> Result<Vec<Author>>;
    async fn get_books_by_author(&self, author_id: Uuid) -> Result<Vec<Book>>;

    // Series
    async fn list_series(&self) -> Result<Vec<Series>>;
    async fn get_books_by_series(&self, series_id: Uuid) -> Result<Vec<Book>>;

    // Playback position
    async fn upsert_playback_position(&self, user_id: Uuid, pos: &UpsertPlaybackPosition) -> Result<PlaybackPosition>;
    async fn get_playback_position(&self, user_id: Uuid, book_id: Uuid) -> Result<Option<PlaybackPosition>>;
    async fn get_user_positions(&self, user_id: Uuid) -> Result<Vec<PlaybackPosition>>;

    // Bookmark
    async fn create_bookmark(&self, user_id: Uuid, bookmark: &CreateBookmark) -> Result<Bookmark>;
    async fn list_bookmarks(&self, user_id: Uuid, book_id: Uuid) -> Result<Vec<Bookmark>>;
    async fn list_all_user_bookmarks(&self, user_id: Uuid) -> Result<Vec<Bookmark>>;
    async fn delete_bookmark(&self, id: Uuid) -> Result<()>;

    // Collection
    async fn create_collection(&self, user_id: Uuid, collection: &CreateCollection) -> Result<Collection>;
    async fn list_collections(&self, user_id: Uuid) -> Result<Vec<Collection>>;
    async fn delete_collection(&self, id: Uuid) -> Result<()>;
    async fn add_book_to_collection(&self, collection_id: Uuid, book_id: Uuid) -> Result<()>;
    async fn remove_book_from_collection(&self, collection_id: Uuid, book_id: Uuid) -> Result<()>;
    async fn get_collection_books(&self, collection_id: Uuid) -> Result<Vec<Book>>;

    // User
    async fn list_users(&self) -> Result<Vec<User>>;
    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>>;
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn create_user(&self, input: &CreateUser) -> Result<User>;
}

// Delegate Repository impl to Database
#[async_trait]
impl Repository for Database {
    async fn list_libraries(&self) -> Result<Vec<Library>> {
        crate::sqlite::list_libraries(&self.pool).await
    }
    async fn get_library(&self, id: i64) -> Result<Option<Library>> {
        crate::sqlite::get_library(&self.pool, id).await
    }
    async fn create_library(&self, input: &CreateLibrary) -> Result<Library> {
        crate::sqlite::create_library(&self.pool, input).await
    }
    async fn update_library(&self, id: i64, input: &UpdateLibrary) -> Result<Library> {
        crate::sqlite::update_library(&self.pool, id, input).await
    }
    async fn delete_library(&self, id: i64) -> Result<()> {
        crate::sqlite::delete_library(&self.pool, id).await
    }
    async fn update_last_scan(&self, id: i64) -> Result<()> {
        crate::sqlite::update_last_scan(&self.pool, id).await
    }
    async fn list_books(&self, filter: &BookFilter) -> Result<PaginatedResult<Book>> {
        crate::sqlite::list_books(&self.pool, filter).await
    }
    async fn get_book(&self, id: Uuid) -> Result<Option<Book>> {
        crate::sqlite::get_book(&self.pool, id).await
    }
    async fn update_book(&self, id: Uuid, input: &UpdateBook) -> Result<Book> {
        crate::sqlite::update_book(&self.pool, id, input).await
    }
    async fn delete_book(&self, id: Uuid) -> Result<()> {
        crate::sqlite::delete_book(&self.pool, id).await
    }
    async fn search_books(&self, query: &str, limit: i64) -> Result<Vec<Book>> {
        crate::sqlite::search_books(&self.pool, query, limit).await
    }
    async fn get_book_authors(&self, book_id: Uuid) -> Result<Vec<(Author, AuthorRole)>> {
        crate::sqlite::get_book_authors(&self.pool, book_id).await
    }
    async fn get_book_series(&self, book_id: Uuid) -> Result<Vec<(Series, Option<f64>)>> {
        crate::sqlite::get_book_series(&self.pool, book_id).await
    }
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>> {
        crate::sqlite::get_track(&self.pool, id).await
    }
    async fn get_tracks_by_book(&self, book_id: Uuid) -> Result<Vec<Track>> {
        crate::sqlite::get_tracks_by_book(&self.pool, book_id).await
    }
    async fn list_authors(&self) -> Result<Vec<Author>> {
        crate::sqlite::list_authors(&self.pool).await
    }
    async fn get_books_by_author(&self, author_id: Uuid) -> Result<Vec<Book>> {
        crate::sqlite::get_books_by_author(&self.pool, author_id).await
    }
    async fn list_series(&self) -> Result<Vec<Series>> {
        crate::sqlite::list_series(&self.pool).await
    }
    async fn get_books_by_series(&self, series_id: Uuid) -> Result<Vec<Book>> {
        crate::sqlite::get_books_by_series(&self.pool, series_id).await
    }
    async fn upsert_playback_position(&self, user_id: Uuid, pos: &UpsertPlaybackPosition) -> Result<PlaybackPosition> {
        crate::sqlite::upsert_playback_position(&self.pool, user_id, pos).await
    }
    async fn get_playback_position(&self, user_id: Uuid, book_id: Uuid) -> Result<Option<PlaybackPosition>> {
        crate::sqlite::get_playback_position(&self.pool, user_id, book_id).await
    }
    async fn get_user_positions(&self, user_id: Uuid) -> Result<Vec<PlaybackPosition>> {
        crate::sqlite::get_user_positions(&self.pool, user_id).await
    }
    async fn create_bookmark(&self, user_id: Uuid, bookmark: &CreateBookmark) -> Result<Bookmark> {
        crate::sqlite::create_bookmark(&self.pool, user_id, bookmark).await
    }
    async fn list_bookmarks(&self, user_id: Uuid, book_id: Uuid) -> Result<Vec<Bookmark>> {
        crate::sqlite::list_bookmarks(&self.pool, user_id, book_id).await
    }
    async fn list_all_user_bookmarks(&self, user_id: Uuid) -> Result<Vec<Bookmark>> {
        crate::sqlite::list_all_user_bookmarks(&self.pool, user_id).await
    }
    async fn delete_bookmark(&self, id: Uuid) -> Result<()> {
        crate::sqlite::delete_bookmark(&self.pool, id).await
    }
    async fn create_collection(&self, user_id: Uuid, collection: &CreateCollection) -> Result<Collection> {
        crate::sqlite::create_collection(&self.pool, user_id, collection).await
    }
    async fn list_collections(&self, user_id: Uuid) -> Result<Vec<Collection>> {
        crate::sqlite::list_collections(&self.pool, user_id).await
    }
    async fn delete_collection(&self, id: Uuid) -> Result<()> {
        crate::sqlite::delete_collection(&self.pool, id).await
    }
    async fn add_book_to_collection(&self, collection_id: Uuid, book_id: Uuid) -> Result<()> {
        crate::sqlite::add_book_to_collection(&self.pool, collection_id, book_id).await
    }
    async fn remove_book_from_collection(&self, collection_id: Uuid, book_id: Uuid) -> Result<()> {
        crate::sqlite::remove_book_from_collection(&self.pool, collection_id, book_id).await
    }
    async fn get_collection_books(&self, collection_id: Uuid) -> Result<Vec<Book>> {
        crate::sqlite::get_collection_books(&self.pool, collection_id).await
    }
    async fn list_users(&self) -> Result<Vec<User>> {
        crate::sqlite::list_users(&self.pool).await
    }
    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>> {
        crate::sqlite::get_user_by_id(&self.pool, id).await
    }
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        crate::sqlite::get_user_by_username(&self.pool, username).await
    }
    async fn create_user(&self, input: &CreateUser) -> Result<User> {
        crate::sqlite::create_user(&self.pool, input).await
    }
}

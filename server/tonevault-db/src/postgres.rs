// PostgreSQL support is not yet implemented with the new Repository trait.
// This module will be updated when PostgreSQL support is needed.

use crate::repository::Repository;

pub struct PostgresRepository;

#[async_trait::async_trait]
impl Repository for PostgresRepository {
    async fn list_libraries(&self) -> crate::Result<Vec<tonevault_core::models::Library>> {
        unimplemented!("PostgreSQL support not yet implemented")
    }
    async fn get_library(&self, _id: i64) -> crate::Result<Option<tonevault_core::models::Library>> {
        unimplemented!()
    }
    async fn create_library(&self, _input: &tonevault_core::models::CreateLibrary) -> crate::Result<tonevault_core::models::Library> {
        unimplemented!()
    }
    async fn update_library(&self, _id: i64, _input: &tonevault_core::models::UpdateLibrary) -> crate::Result<tonevault_core::models::Library> {
        unimplemented!()
    }
    async fn delete_library(&self, _id: i64) -> crate::Result<()> {
        unimplemented!()
    }
    async fn update_last_scan(&self, _id: i64) -> crate::Result<()> {
        unimplemented!()
    }
    async fn list_books(&self, _filter: &tonevault_core::models::BookFilter) -> crate::Result<tonevault_core::models::PaginatedResult<tonevault_core::models::Book>> {
        unimplemented!()
    }
    async fn get_book(&self, _id: uuid::Uuid) -> crate::Result<Option<tonevault_core::models::Book>> {
        unimplemented!()
    }
    async fn update_book(&self, _id: uuid::Uuid, _input: &tonevault_core::models::UpdateBook) -> crate::Result<tonevault_core::models::Book> {
        unimplemented!()
    }
    async fn delete_book(&self, _id: uuid::Uuid) -> crate::Result<()> {
        unimplemented!()
    }
    async fn search_books(&self, _query: &str, _limit: i64) -> crate::Result<Vec<tonevault_core::models::Book>> {
        unimplemented!()
    }
    async fn get_book_authors(&self, _book_id: uuid::Uuid) -> crate::Result<Vec<(tonevault_core::models::Author, tonevault_core::models::AuthorRole)>> {
        unimplemented!()
    }
    async fn get_book_series(&self, _book_id: uuid::Uuid) -> crate::Result<Vec<(tonevault_core::models::Series, Option<f64>)>> {
        unimplemented!()
    }
    async fn get_track(&self, _id: uuid::Uuid) -> crate::Result<Option<tonevault_core::models::Track>> {
        unimplemented!()
    }
    async fn get_tracks_by_book(&self, _book_id: uuid::Uuid) -> crate::Result<Vec<tonevault_core::models::Track>> {
        unimplemented!()
    }
    async fn list_authors(&self) -> crate::Result<Vec<tonevault_core::models::Author>> {
        unimplemented!()
    }
    async fn get_books_by_author(&self, _author_id: uuid::Uuid) -> crate::Result<Vec<tonevault_core::models::Book>> {
        unimplemented!()
    }
    async fn list_series(&self) -> crate::Result<Vec<tonevault_core::models::Series>> {
        unimplemented!()
    }
    async fn get_books_by_series(&self, _series_id: uuid::Uuid) -> crate::Result<Vec<tonevault_core::models::Book>> {
        unimplemented!()
    }
    async fn upsert_playback_position(&self, _user_id: uuid::Uuid, _pos: &tonevault_core::models::UpsertPlaybackPosition) -> crate::Result<tonevault_core::models::PlaybackPosition> {
        unimplemented!()
    }
    async fn get_playback_position(&self, _user_id: uuid::Uuid, _book_id: uuid::Uuid) -> crate::Result<Option<tonevault_core::models::PlaybackPosition>> {
        unimplemented!()
    }
    async fn get_user_positions(&self, _user_id: uuid::Uuid) -> crate::Result<Vec<tonevault_core::models::PlaybackPosition>> {
        unimplemented!()
    }
    async fn create_bookmark(&self, _user_id: uuid::Uuid, _bookmark: &tonevault_core::models::CreateBookmark) -> crate::Result<tonevault_core::models::Bookmark> {
        unimplemented!()
    }
    async fn list_bookmarks(&self, _user_id: uuid::Uuid, _book_id: uuid::Uuid) -> crate::Result<Vec<tonevault_core::models::Bookmark>> {
        unimplemented!()
    }
    async fn list_all_user_bookmarks(&self, _user_id: uuid::Uuid) -> crate::Result<Vec<tonevault_core::models::Bookmark>> {
        unimplemented!()
    }
    async fn delete_bookmark(&self, _id: uuid::Uuid) -> crate::Result<()> {
        unimplemented!()
    }
    async fn create_collection(&self, _user_id: uuid::Uuid, _collection: &tonevault_core::models::CreateCollection) -> crate::Result<tonevault_core::models::Collection> {
        unimplemented!()
    }
    async fn list_collections(&self, _user_id: uuid::Uuid) -> crate::Result<Vec<tonevault_core::models::Collection>> {
        unimplemented!()
    }
    async fn delete_collection(&self, _id: uuid::Uuid) -> crate::Result<()> {
        unimplemented!()
    }
    async fn add_book_to_collection(&self, _collection_id: uuid::Uuid, _book_id: uuid::Uuid) -> crate::Result<()> {
        unimplemented!()
    }
    async fn remove_book_from_collection(&self, _collection_id: uuid::Uuid, _book_id: uuid::Uuid) -> crate::Result<()> {
        unimplemented!()
    }
    async fn get_collection_books(&self, _collection_id: uuid::Uuid) -> crate::Result<Vec<tonevault_core::models::Book>> {
        unimplemented!()
    }
    async fn list_users(&self) -> crate::Result<Vec<tonevault_core::models::User>> {
        unimplemented!()
    }
    async fn get_user_by_id(&self, _id: uuid::Uuid) -> crate::Result<Option<tonevault_core::models::User>> {
        unimplemented!()
    }
    async fn get_user_by_username(&self, _username: &str) -> crate::Result<Option<tonevault_core::models::User>> {
        unimplemented!()
    }
    async fn create_user(&self, _input: &tonevault_core::models::CreateUser) -> crate::Result<tonevault_core::models::User> {
        unimplemented!()
    }
}
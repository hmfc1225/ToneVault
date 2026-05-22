use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use tonevault_core::models::*;
use tonevault_db::Repository;

use crate::AppState;

macro_rules! uuid_path {
    ($id:expr) => {
        match Uuid::parse_str($id.as_ref()) {
            Ok(u) => u,
            Err(_) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid id"}))).into_response(),
        }
    };
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/books", axum::routing::get(list_books))
        .route("/api/v1/books/{id}", axum::routing::get(get_book).put(update_book).delete(delete_book))
        .route("/api/v1/books/{id}/authors", axum::routing::get(get_book_authors))
        .route("/api/v1/books/{id}/series", axum::routing::get(get_book_series))
        .route("/api/v1/books/{id}/tracks", axum::routing::get(get_book_tracks))
        .route("/api/v1/authors", axum::routing::get(list_authors))
        .route("/api/v1/authors/{id}/books", axum::routing::get(get_author_books))
        .route("/api/v1/series", axum::routing::get(list_series))
        .route("/api/v1/series/{id}/books", axum::routing::get(get_series_books))
        .route("/api/v1/search", axum::routing::get(search_books))
        .route("/api/v1/positions/{user_id}", axum::routing::put(upsert_position).get(get_user_positions))
        .route("/api/v1/positions/{user_id}/{book_id}", axum::routing::get(get_position))
        .route("/api/v1/bookmarks/{user_id}", axum::routing::post(create_bookmark).get(list_all_user_bookmarks))
        .route("/api/v1/bookmarks/{user_id}/{book_id}", axum::routing::get(list_bookmarks))
        .route("/api/v1/bookmarks/item/{id}", axum::routing::delete(delete_bookmark))
        .route("/api/v1/collections/{user_id}", axum::routing::post(create_collection).get(list_collections))
        .route("/api/v1/collections/item/{id}", axum::routing::delete(delete_collection))
        .route("/api/v1/collections/{collection_id}/books/{book_id}", axum::routing::post(add_book_to_collection).delete(remove_book_from_collection))
        .route("/api/v1/collections/{id}/books", axum::routing::get(get_collection_books))
}

#[derive(Deserialize)]
pub struct ListBooksQuery {
    pub library_id: Option<String>,
    pub author_id: Option<String>,
    pub series_id: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn list_books(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListBooksQuery>,
) -> impl IntoResponse {
    let library_id = query.library_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let author_id = query.author_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let series_id = query.series_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());

    let sort = match query.sort.as_deref() {
        Some("added") => BookSort::Added,
        Some("duration") => BookSort::Duration,
        Some("year") => BookSort::Year,
        Some("author") => BookSort::Author,
        _ => BookSort::Title,
    };

    let order = match query.order.as_deref() {
        Some("desc") => SortOrder::Desc,
        _ => SortOrder::Asc,
    };

    let filter = BookFilter {
        library_id,
        author_id,
        series_id,
        query: query.q,
        sort,
        order,
        page: query.page.unwrap_or(1),
        per_page: query.per_page.unwrap_or(20),
    };

    match state.repo.list_books(&filter).await {
        Ok(result) => Json(ListResponse {
            items: result.items.iter().map(book_to_response).collect(),
            total: result.total,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
        }).into_response(),
        Err(e) => err500(&e),
    }
}

pub async fn get_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = uuid_path!(id);
    match state.repo.get_book(uuid).await {
        Ok(Some(book)) => Json(book_to_response(&book)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Book not found"}))).into_response(),
        Err(e) => err500(&e),
    }
}

#[derive(Deserialize)]
pub struct UpdateBookRequest {
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

pub async fn update_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateBookRequest>,
) -> impl IntoResponse {
    let uuid = uuid_path!(id);
    let update = UpdateBook {
        title: req.title,
        subtitle: req.subtitle,
        description: req.description,
        cover_path: req.cover_path,
        language: req.language,
        publisher: req.publisher,
        publish_year: req.publish_year,
        isbn: req.isbn,
        asin: req.asin,
    };
    match state.repo.update_book(uuid, &update).await {
        Ok(book) => Json(book_to_response(&book)).into_response(),
        Err(e) => err500(&e),
    }
}

pub async fn delete_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = uuid_path!(id);
    match state.repo.delete_book(uuid).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err500(&e),
    }
}

pub async fn get_book_authors(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = uuid_path!(id);
    match state.repo.get_book_authors(uuid).await {
        Ok(authors) => {
            #[derive(Serialize)]
            struct AuthorWithRole { id: String, name: String, role: String }
            let items: Vec<AuthorWithRole> = authors.iter().map(|(a, r)| AuthorWithRole {
                id: a.id.to_string(),
                name: a.name.clone(),
                role: match r {
                    AuthorRole::Author => "author",
                    AuthorRole::Narrator => "narrator",
                    AuthorRole::Translator => "translator",
                }.to_string(),
            }).collect();
            Json(items).into_response()
        }
        Err(e) => err500(&e),
    }
}

pub async fn get_book_series(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = uuid_path!(id);
    match state.repo.get_book_series(uuid).await {
        Ok(series) => {
            #[derive(Serialize)]
            struct SeriesWithPosition { id: String, name: String, position: Option<f64> }
            let items: Vec<SeriesWithPosition> = series.iter().map(|(s, p)| SeriesWithPosition {
                id: s.id.to_string(), name: s.name.clone(), position: *p,
            }).collect();
            Json(items).into_response()
        }
        Err(e) => err500(&e),
    }
}

pub async fn get_book_tracks(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = uuid_path!(id);
    match state.repo.get_tracks_by_book(uuid).await {
        Ok(tracks) => {
            #[derive(Serialize)]
            struct TrackResponse { id: String, title: String, track_number: i32, disc_number: Option<i32>, file_path: String, duration_secs: f64, mime_type: String }
            let items: Vec<TrackResponse> = tracks.iter().map(|t| TrackResponse {
                id: t.id.to_string(), title: t.title.clone(), track_number: t.track_number,
                disc_number: t.disc_number, file_path: t.file_path.clone(),
                duration_secs: t.duration_secs, mime_type: t.mime_type.clone(),
            }).collect();
            Json(items).into_response()
        }
        Err(e) => err500(&e),
    }
}

pub async fn list_authors(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.repo.list_authors().await {
        Ok(authors) => {
            #[derive(Serialize)]
            struct AuthorResponse { id: String, name: String, book_count: i64 }
            let items: Vec<AuthorResponse> = authors.iter().map(|a| AuthorResponse {
                id: a.id.to_string(), name: a.name.clone(), book_count: a.book_count,
            }).collect();
            Json(items).into_response()
        }
        Err(e) => err500(&e),
    }
}

pub async fn get_author_books(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = uuid_path!(id);
    match state.repo.get_books_by_author(uuid).await {
        Ok(books) => Json(books.iter().map(book_to_response).collect::<Vec<_>>()).into_response(),
        Err(e) => err500(&e),
    }
}

pub async fn list_series(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.repo.list_series().await {
        Ok(series) => {
            #[derive(Serialize)]
            struct SeriesResponse { id: String, name: String, book_count: i64 }
            let items: Vec<SeriesResponse> = series.iter().map(|s| SeriesResponse {
                id: s.id.to_string(), name: s.name.clone(), book_count: s.book_count,
            }).collect();
            Json(items).into_response()
        }
        Err(e) => err500(&e),
    }
}

pub async fn get_series_books(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = uuid_path!(id);
    match state.repo.get_books_by_series(uuid).await {
        Ok(books) => Json(books.iter().map(book_to_response).collect::<Vec<_>>()).into_response(),
        Err(e) => err500(&e),
    }
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i64>,
}

pub async fn search_books(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(20);
    match state.repo.search_books(&query.q, limit).await {
        Ok(books) => Json(books.iter().map(book_to_response).collect::<Vec<_>>()).into_response(),
        Err(e) => err500(&e),
    }
}

#[derive(Deserialize)]
pub struct UpsertPositionRequest {
    pub book_id: String,
    pub track_id: String,
    pub position_secs: f64,
    pub duration_secs: f64,
}

pub async fn upsert_position(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(req): Json<UpsertPositionRequest>,
) -> impl IntoResponse {
    let uid = uuid_path!(user_id);
    let book_id = uuid_path!(req.book_id);
    let track_id = uuid_path!(req.track_id);
    let pos = UpsertPlaybackPosition { book_id, track_id, position_secs: req.position_secs, duration_secs: req.duration_secs };
    match state.repo.upsert_playback_position(uid, &pos).await {
        Ok(p) => {
            #[derive(Serialize)]
            struct PosResponse { book_id: String, track_id: String, position_secs: f64, percentage: f64, is_finished: bool }
            Json(PosResponse {
                book_id: p.book_id.to_string(), track_id: p.track_id.to_string(),
                position_secs: p.position_secs, percentage: p.percentage, is_finished: p.is_finished,
            }).into_response()
        }
        Err(e) => err500(&e),
    }
}

pub async fn get_position(
    State(state): State<Arc<AppState>>,
    Path((user_id, book_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let uid = uuid_path!(user_id);
    let bid = uuid_path!(book_id);
    match state.repo.get_playback_position(uid, bid).await {
        Ok(Some(p)) => {
            #[derive(Serialize)]
            struct PosResponse { book_id: String, track_id: String, position_secs: f64, percentage: f64, is_finished: bool }
            Json(PosResponse {
                book_id: p.book_id.to_string(), track_id: p.track_id.to_string(),
                position_secs: p.position_secs, percentage: p.percentage, is_finished: p.is_finished,
            }).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "No position found"}))).into_response(),
        Err(e) => err500(&e),
    }
}

pub async fn get_user_positions(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = uuid_path!(id);
    match state.repo.get_user_positions(uuid).await {
        Ok(positions) => {
            #[derive(Serialize)]
            struct PosResponse { book_id: String, track_id: String, position_secs: f64, percentage: f64, is_finished: bool }
            let items: Vec<PosResponse> = positions.iter().map(|p| PosResponse {
                book_id: p.book_id.to_string(), track_id: p.track_id.to_string(),
                position_secs: p.position_secs, percentage: p.percentage, is_finished: p.is_finished,
            }).collect();
            Json(items).into_response()
        }
        Err(e) => err500(&e),
    }
}

#[derive(Deserialize)]
pub struct CreateBookmarkRequest {
    pub book_id: String,
    pub track_id: String,
    pub title: String,
    pub note: Option<String>,
    pub position_secs: f64,
}

pub async fn create_bookmark(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(req): Json<CreateBookmarkRequest>,
) -> impl IntoResponse {
    let uid = uuid_path!(user_id);
    let bookmark = CreateBookmark {
        book_id: uuid_path!(req.book_id),
        track_id: uuid_path!(req.track_id),
        title: req.title,
        note: req.note,
        position_secs: req.position_secs,
    };
    match state.repo.create_bookmark(uid, &bookmark).await {
        Ok(b) => {
            #[derive(Serialize)]
            struct BmResponse { id: String, title: String, position_secs: f64 }
            Json(BmResponse { id: b.id.to_string(), title: b.title.clone(), position_secs: b.position_secs }).into_response()
        }
        Err(e) => err500(&e),
    }
}

pub async fn list_bookmarks(
    State(state): State<Arc<AppState>>,
    Path((user_id, book_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let uid = uuid_path!(user_id);
    let bid = uuid_path!(book_id);
    match state.repo.list_bookmarks(uid, bid).await {
        Ok(bookmarks) => {
            #[derive(Serialize)]
            struct BmResponse { id: String, book_id: String, title: String, position_secs: f64, note: Option<String> }
            let items: Vec<BmResponse> = bookmarks.iter().map(|b| BmResponse {
                id: b.id.to_string(), book_id: b.book_id.to_string(), title: b.title.clone(), position_secs: b.position_secs, note: b.note.clone(),
            }).collect();
            Json(items).into_response()
        }
        Err(e) => err500(&e),
    }
}

pub async fn list_all_user_bookmarks(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let uid = uuid_path!(user_id);
    match state.repo.list_all_user_bookmarks(uid).await {
        Ok(bookmarks) => {
            #[derive(Serialize)]
            struct BmResponse { id: String, book_id: String, title: String, position_secs: f64, note: Option<String> }
            let items: Vec<BmResponse> = bookmarks.iter().map(|b| BmResponse {
                id: b.id.to_string(), book_id: b.book_id.to_string(), title: b.title.clone(), position_secs: b.position_secs, note: b.note.clone(),
            }).collect();
            Json(items).into_response()
        }
        Err(e) => err500(&e),
    }
}

pub async fn delete_bookmark(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = uuid_path!(id);
    match state.repo.delete_bookmark(uuid).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err500(&e),
    }
}

#[derive(Deserialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    pub description: Option<String>,
}

pub async fn create_collection(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(req): Json<CreateCollectionRequest>,
) -> impl IntoResponse {
    let uid = uuid_path!(user_id);
    let collection = CreateCollection { name: req.name, description: req.description };
    match state.repo.create_collection(uid, &collection).await {
        Ok(c) => {
            #[derive(Serialize)]
            struct CollResponse { id: String, name: String, book_count: i64 }
            Json(CollResponse { id: c.id.to_string(), name: c.name.clone(), book_count: c.book_count }).into_response()
        }
        Err(e) => err500(&e),
    }
}

pub async fn list_collections(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = uuid_path!(id);
    match state.repo.list_collections(uuid).await {
        Ok(collections) => {
            #[derive(Serialize)]
            struct CollResponse { id: String, name: String, book_count: i64 }
            let items: Vec<CollResponse> = collections.iter().map(|c| CollResponse {
                id: c.id.to_string(), name: c.name.clone(), book_count: c.book_count,
            }).collect();
            Json(items).into_response()
        }
        Err(e) => err500(&e),
    }
}

pub async fn delete_collection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = uuid_path!(id);
    match state.repo.delete_collection(uuid).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err500(&e),
    }
}

pub async fn add_book_to_collection(
    State(state): State<Arc<AppState>>,
    Path((collection_id, book_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let cid = uuid_path!(collection_id);
    let bid = uuid_path!(book_id);
    match state.repo.add_book_to_collection(cid, bid).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err500(&e),
    }
}

pub async fn remove_book_from_collection(
    State(state): State<Arc<AppState>>,
    Path((collection_id, book_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let cid = uuid_path!(collection_id);
    let bid = uuid_path!(book_id);
    match state.repo.remove_book_from_collection(cid, bid).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => err500(&e),
    }
}

pub async fn get_collection_books(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = uuid_path!(id);
    match state.repo.get_collection_books(uuid).await {
        Ok(books) => Json(books.iter().map(book_to_response).collect::<Vec<_>>()).into_response(),
        Err(e) => err500(&e),
    }
}

#[derive(Serialize)]
struct BookResponse {
    id: String,
    library_id: String,
    title: String,
    subtitle: Option<String>,
    description: Option<String>,
    duration_secs: f64,
    track_count: i32,
    file_size: i64,
    cover_path: Option<String>,
    language: Option<String>,
    publisher: Option<String>,
    publish_year: Option<i32>,
    isbn: Option<String>,
    asin: Option<String>,
    source_path: String,
    metadata_source: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize)]
struct ListResponse {
    items: Vec<BookResponse>,
    total: i64,
    page: i64,
    per_page: i64,
    total_pages: i64,
}

fn book_to_response(book: &Book) -> BookResponse {
    BookResponse {
        id: book.id.to_string(),
        library_id: book.library_id.to_string(),
        title: book.title.clone(),
        subtitle: book.subtitle.clone(),
        description: book.description.clone(),
        duration_secs: book.duration_secs,
        track_count: book.track_count,
        file_size: book.file_size,
        cover_path: book.cover_path.clone(),
        language: book.language.clone(),
        publisher: book.publisher.clone(),
        publish_year: book.publish_year,
        isbn: book.isbn.clone(),
        asin: book.asin.clone(),
        source_path: book.source_path.clone(),
        metadata_source: book.metadata_source.clone(),
        created_at: book.created_at.to_string(),
        updated_at: book.updated_at.to_string(),
    }
}

fn err500(e: &tonevault_db::DbError) -> axum::response::Response {
    tracing::error!("Database error: {}", e);
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Database error"}))).into_response()
}
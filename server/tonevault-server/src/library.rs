use axum::Router;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use tonevault_core::models::library::{CreateLibrary, Library, SourceType, UpdateLibrary};
use tonevault_db::Repository;

use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/libraries", axum::routing::get(list_libraries).post(create_library))
        .route("/api/v1/libraries/{id}", axum::routing::get(get_library).put(update_library).delete(delete_library))
        .route("/api/v1/libraries/scan", axum::routing::post(trigger_scan))
}

#[derive(Serialize)]
pub struct LibraryResponse {
    #[serde(flatten)]
    library: Library,
    book_count: i64,
}

pub async fn list_libraries(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<LibraryResponse>>, StatusCode> {
    let libraries = state.repo.list_libraries().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let responses: Vec<LibraryResponse> = libraries
        .into_iter()
        .map(|library| LibraryResponse {
            library,
            book_count: 0,
        })
        .collect();
    Ok(Json(responses))
}

pub async fn get_library(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<LibraryResponse>, StatusCode> {
    let library = state.repo.get_library(id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let library = library.ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(LibraryResponse {
        library,
        book_count: 0,
    }))
}

pub async fn create_library(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateLibrary>,
) -> Result<Json<LibraryResponse>, (StatusCode, String)> {
    let library = state.repo.create_library(&input).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    Ok(Json(LibraryResponse {
        library,
        book_count: 0,
    }))
}

pub async fn update_library(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateLibrary>,
) -> Result<Json<LibraryResponse>, (StatusCode, String)> {
    let library = state.repo.update_library(id, &input).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    Ok(Json(LibraryResponse {
        library,
        book_count: 0,
    }))
}

pub async fn delete_library(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    state.repo.delete_library(id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct ScanRequest {
    pub library_id: i64,
}

pub async fn trigger_scan(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ScanRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let scan_mgr = crate::scan::ScanManager::new(state.repo.clone());
    scan_mgr.start_scan(req.library_id).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    Ok(StatusCode::ACCEPTED)
}

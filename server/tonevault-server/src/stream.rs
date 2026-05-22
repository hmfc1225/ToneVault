use axum::Router;
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
use uuid::Uuid;

use tonevault_db::Repository;

use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/tracks/{id}/stream", axum::routing::get(stream_track))
        .route("/api/v1/books/{id}/cover", axum::routing::get(get_book_cover))
}

pub async fn stream_track(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    req: axum::extract::Request,
) -> impl IntoResponse {
    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid track id").into_response(),
    };

    let track = match state.repo.get_track(uuid).await {
        Ok(Some(t)) => t,
        Ok(None) => return (StatusCode::NOT_FOUND, "Track not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to get track: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let path = std::path::Path::new(&track.file_path);
    if !path.exists() {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    let file_size = match tokio::fs::metadata(&track.file_path).await {
        Ok(m) => m.len(),
        Err(_) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
    };

    let range = req.headers()
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| parse_range(v, file_size));

    let mut file = match File::open(&track.file_path).await {
        Ok(f) => f,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to open file").into_response(),
    };

    if let Some((start, end)) = range {
        let content_length = end - start + 1;
        let _ = file.seek(SeekFrom::Start(start)).await;

        let mut buffer = vec![0u8; content_length as usize];
        if let Err(e) = file.read_exact(&mut buffer).await {
            tracing::error!("Failed to read file: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Read error").into_response();
        }

        Response::builder()
            .status(StatusCode::PARTIAL_CONTENT)
            .header(header::CONTENT_TYPE, &track.mime_type)
            .header(header::CONTENT_LENGTH, content_length)
            .header(header::CONTENT_RANGE, format!("bytes {}-{}/{}", start, end, file_size))
            .header(header::ACCEPT_RANGES, "bytes")
            .body(axum::body::Body::from(buffer))
            .unwrap()
            .into_response()
    } else {
        let mut buffer = Vec::with_capacity(file_size as usize);
        if let Err(e) = file.read_to_end(&mut buffer).await {
            tracing::error!("Failed to read file: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Read error").into_response();
        }

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, &track.mime_type)
            .header(header::CONTENT_LENGTH, file_size)
            .header(header::ACCEPT_RANGES, "bytes")
            .body(axum::body::Body::from(buffer))
            .unwrap()
            .into_response()
    }
}

pub async fn get_book_cover(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid book id").into_response(),
    };

    let book = match state.repo.get_book(uuid).await {
        Ok(Some(b)) => b,
        Ok(None) => return (StatusCode::NOT_FOUND, "Book not found").into_response(),
        Err(e) => {
            tracing::error!("Failed to get book: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let cover_path = match &book.cover_path {
        Some(p) => p.clone(),
        None => return (StatusCode::NOT_FOUND, "No cover image").into_response(),
    };

    let path = std::path::Path::new(&cover_path);
    if !path.exists() {
        return (StatusCode::NOT_FOUND, "Cover file not found").into_response();
    }

    let mime_type = match path.extension().and_then(|e| e.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    };

    match tokio::fs::read(&cover_path).await {
        Ok(data) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime_type)
            .header(header::CACHE_CONTROL, "public, max-age=86400")
            .body(axum::body::Body::from(data))
            .unwrap()
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Cover file not found").into_response(),
    }
}

fn parse_range(header: &str, file_size: u64) -> Option<(u64, u64)> {
    let range = header.strip_prefix("bytes=")?;
    let parts: Vec<&str> = range.split('-').collect();
    if parts.len() != 2 {
        return None;
    }
    let start: u64 = parts[0].parse().ok()?;
    let end = if parts[1].is_empty() {
        file_size - 1
    } else {
        let e: u64 = parts[1].parse().ok()?;
        e.min(file_size - 1)
    };
    if start > end {
        return None;
    }
    Some((start, end))
}
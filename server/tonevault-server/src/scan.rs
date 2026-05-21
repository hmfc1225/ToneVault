use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use std::path::Path as StdPath;
use std::sync::Arc;
use uuid::Uuid;

use tonevault_core::metadata::MetadataParser;
use tonevault_core::models::*;
use tonevault_core::scanner::Scanner;
use tonevault_db::repository::Repository;

#[derive(Clone)]
pub struct ScanState {
    pub repo: Arc<dyn Repository>,
}

#[derive(serde::Serialize)]
pub struct ScanResponse {
    pub status: String,
    pub message: String,
}

pub async fn trigger_scan(
    State(state): State<ScanState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid library id"}))).into_response(),
    };

    let library = match state.repo.get_library(uuid).await {
        Ok(Some(l)) => l,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Library not found"}))).into_response(),
        Err(e) => {
            tracing::error!("Failed to get library: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Database error"}))).into_response();
        }
    };

    if library.scan_status == ScanStatus::Scanning {
        return (StatusCode::CONFLICT, Json(serde_json::json!({"error": "Library is already being scanned"}))).into_response();
    }

    if let Err(e) = state.repo.update_library_scan_status(uuid, ScanStatus::Scanning).await {
        tracing::error!("Failed to update scan status: {}", e);
    }

    let repo = state.repo.clone();
    tokio::spawn(async move {
        let scanner = Scanner::new();
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        let lib = library.clone();
        let scan_handle = tokio::spawn(async move {
            scanner.scan_library(&lib, tx).await
        });

        while let Some(event) = rx.recv().await {
            match event {
                tonevault_core::scanner::ScannerEvent::BookParsed { library_id, dir, metadata } => {
                    let title = metadata.title.clone().unwrap_or_else(|| "Unknown".to_string());
                    let source_path = dir.clone();

                    // Deduplication: check if book already exists with this source_path
                    let existing_book = repo.find_book_by_source_path(library_id, &source_path).await.ok().flatten();

                    let book = if let Some(existing) = existing_book {
                        // Re-scan: delete old tracks and re-create
                        let _ = repo.delete_tracks_by_book(existing.id).await;
                        // Remove old author links so they get re-linked
                        if let Ok(old_authors) = repo.get_book_authors(existing.id).await {
                            for (author, _role) in old_authors {
                                let _ = repo.remove_book_author(existing.id, author.id).await;
                            }
                        }
                        tracing::info!("Updating existing book: {} from {}", title, dir);
                        existing
                    } else {
                        let create_book = CreateBook {
                            library_id,
                            title: title.clone(),
                            subtitle: metadata.subtitle.clone(),
                            description: metadata.description.clone(),
                            cover_path: None,
                            language: metadata.language.clone(),
                            publisher: metadata.publisher.clone(),
                            publish_year: metadata.publish_year,
                            isbn: None,
                            asin: None,
                            source_path: source_path.clone(),
                        };

                        match repo.create_book(&create_book).await {
                            Ok(book) => {
                                tracing::info!("Created book: {} from {}", title, dir);
                                book
                            }
                            Err(e) => {
                                tracing::error!("Failed to create book from {}: {}", dir, e);
                                continue;
                            }
                        }
                    };

                    // Link author
                    if let Some(author_name) = &metadata.author {
                        if let Ok(Some(author)) = repo.find_author_by_name(author_name).await {
                            let _ = repo.add_book_author(book.id, author.id, AuthorRole::Author).await;
                        } else if let Ok(author) = repo.create_author(&CreateAuthor {
                            name: author_name.clone(),
                            description: None,
                        }).await {
                            let _ = repo.add_book_author(book.id, author.id, AuthorRole::Author).await;
                        }
                    }

                    // Link narrator
                    if let Some(narrator_name) = &metadata.narrator {
                        if let Ok(Some(narrator)) = repo.find_author_by_name(narrator_name).await {
                            let _ = repo.add_book_author(book.id, narrator.id, AuthorRole::Narrator).await;
                        } else if let Ok(narrator) = repo.create_author(&CreateAuthor {
                            name: narrator_name.clone(),
                            description: None,
                        }).await {
                            let _ = repo.add_book_author(book.id, narrator.id, AuthorRole::Narrator).await;
                        }
                    }

                    // Link series
                    if let Some(series_name) = &metadata.series {
                        if let Ok(Some(series)) = repo.find_series_by_name(series_name).await {
                            let _ = repo.add_book_series(book.id, series.id, metadata.series_position).await;
                        } else if let Ok(series) = repo.create_series(&CreateSeries {
                            name: series_name.clone(),
                            description: None,
                        }).await {
                            let _ = repo.add_book_series(book.id, series.id, metadata.series_position).await;
                        }
                    }

                    // Create tracks and compute stats
                    let mut total_duration = 0.0f64;
                    let mut total_file_size = 0i64;
                    let track_count = metadata.tracks.len() as i32;

                    for track in &metadata.tracks {
                        let file_size = std::fs::metadata(&track.file_path)
                            .map(|m| m.len() as i64)
                            .unwrap_or(0);
                        total_file_size += file_size;
                        total_duration += track.duration_secs;

                        let new_track = Track {
                            id: Uuid::new_v4(),
                            book_id: book.id,
                            title: track.title.clone(),
                            disc_number: track.disc_number,
                            track_number: track.track_number,
                            file_path: track.file_path.clone(),
                            file_size,
                            mime_type: track.mime_type.clone(),
                            duration_secs: track.duration_secs,
                            bitrate: track.bitrate,
                            sample_rate: track.sample_rate,
                            channels: track.channels,
                            codec: track.codec.clone(),
                            created_at: chrono::Utc::now().naive_utc(),
                        };
                        let _ = repo.create_track(&new_track).await;
                    }

                    // Update book stats
                    if track_count > 0 {
                        if let Err(e) = repo.update_book_stats(book.id, total_duration, track_count, total_file_size).await {
                            tracing::error!("Failed to update book stats: {}", e);
                        }
                    }

                    // Save cover path
                    let book_dir = StdPath::new(&dir);
                    if let Some(cover_path) = MetadataParser::find_cover(book_dir) {
                        let cover_str = cover_path.to_string_lossy().to_string();
                        let update = UpdateBook {
                            cover_path: Some(cover_str),
                            ..Default::default()
                        };
                        if let Err(e) = repo.update_book(book.id, &update).await {
                            tracing::error!("Failed to update cover path: {}", e);
                        }
                    }
                }
                tonevault_core::scanner::ScannerEvent::Completed { library_id, new_books, .. } => {
                    tracing::info!("Scan completed: {} new books", new_books);
                    if let Err(e) = repo.update_library_scan_time(library_id).await {
                        tracing::error!("Failed to update scan time: {}", e);
                    }
                    if let Err(e) = repo.update_library_book_count(library_id).await {
                        tracing::error!("Failed to update library book count: {}", e);
                    }
                    if let Err(e) = repo.update_library_scan_status(library_id, ScanStatus::Idle).await {
                        tracing::error!("Failed to update scan status: {}", e);
                    }
                }
                tonevault_core::scanner::ScannerEvent::Error { library_id, message } => {
                    tracing::error!("Scan error: {}", message);
                    if let Err(e) = repo.update_library_scan_status(library_id, ScanStatus::Error).await {
                        tracing::error!("Failed to update scan status: {}", e);
                    }
                }
                _ => {}
            }
        }

        if let Err(e) = scan_handle.await {
            tracing::error!("Scan task failed: {}", e);
        }
    });

    Json(ScanResponse {
        status: "scanning".to_string(),
        message: format!("Scan started for library {}", id),
    }).into_response()
}

pub async fn scan_status(
    State(state): State<ScanState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = match Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid library id"}))).into_response(),
    };

    match state.repo.get_library(uuid).await {
        Ok(Some(lib)) => {
            #[derive(serde::Serialize)]
            struct StatusResponse {
                scan_status: String,
                last_scan_at: Option<String>,
                book_count: i64,
            }
            Json(StatusResponse {
                scan_status: match lib.scan_status {
                    ScanStatus::Idle => "idle".to_string(),
                    ScanStatus::Scanning => "scanning".to_string(),
                    ScanStatus::Error => "error".to_string(),
                },
                last_scan_at: lib.last_scan_at.map(|t| t.to_string()),
                book_count: lib.book_count,
            }).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Library not found"}))).into_response(),
        Err(e) => {
            tracing::error!("Failed to get library: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Database error"}))).into_response()
        }
    }
}

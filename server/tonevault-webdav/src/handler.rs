use axum::body::Body;
use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::extract::Request;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};

use tonevault_core::models::*;
use tonevault_db::repository::Repository;

use crate::xml::{DavProp, DavResponse, MultiStatus, PropStat, ResourceType};#[derive(Clone)]
pub struct WebDavState {
    pub repo: Arc<dyn Repository>,
    pub base_path: String,
}

pub async fn webdav_handler(State(state): State<WebDavState>, req: Request) -> impl IntoResponse {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let dav_path = path.strip_prefix(&state.base_path).unwrap_or(&path);
    let dav_path = dav_path.strip_prefix('/').unwrap_or(dav_path);

    match method.as_str() {
        "OPTIONS" => options().await,
        "PROPFIND" => propfind(&state, dav_path, req).await,
        "GET" => get_file(&state, dav_path, req).await,
        "HEAD" => head_file(&state, dav_path).await,
        "PUT" => put_file(&state, dav_path, req).await,
        "MKCOL" => mkcol(&state, dav_path).await,
        "DELETE" => delete_file(&state, dav_path).await,
        _ => (StatusCode::METHOD_NOT_ALLOWED, "Method not supported").into_response(),
    }
}

async fn options() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("DAV", "1, 2")
        .header("Allow", "OPTIONS, PROPFIND, GET, HEAD, PUT, MKCOL, DELETE")
        .header(header::CONTENT_LENGTH, 0)
        .body(Body::empty())
        .unwrap()
        .into_response()
}

async fn propfind(state: &WebDavState, dav_path: &str, req: Request) -> Response {
    let depth = req
        .headers()
        .get("Depth")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("1");

    let href_base = &state.base_path;

    // Root: list libraries
    if dav_path.is_empty() || dav_path == "/" {
        let libraries = match state.repo.list_libraries().await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("WebDAV PROPFIND error: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
            }
        };

        let mut responses = vec![DavResponse {
            href: href_base.to_string(),
            propstat: vec![PropStat {
                prop: DavProp {
                    resource_type: Some(ResourceType::Collection),
                    content_type: None,
                    content_length: None,
                    display_name: Some("ToneVault".into()),
                    last_modified: None,
                    creation_date: None,
                    etag: None,
                },
                status: "HTTP/1.1 200 OK".into(),
            }],
        }];

        if depth != "0" {
            for lib in &libraries {
                responses.push(DavResponse {
                    href: format!("{}/{}", href_base, url_encode(&lib.name)),
                    propstat: vec![PropStat {
                        prop: DavProp {
                            resource_type: Some(ResourceType::Collection),
                            content_type: None,
                            content_length: None,
                            display_name: Some(lib.name.clone()),
                            last_modified: Some(lib.updated_at.to_string()),
                            creation_date: Some(lib.created_at.to_string()),
                            etag: None,
                        },
                        status: "HTTP/1.1 200 OK".into(),
                    }],
                });
            }
        }

        return multistatus_response(&MultiStatus { responses });
    }

    // Parse path segments: {library_name} or {library_name}/{author_name}/{book_title}
    let segments: Vec<&str> = dav_path.split('/').filter(|s| !s.is_empty()).collect();

    match segments.len() {
        1 => {
            // Library level: list authors
            let lib_name = url_decode(segments[0]);
            let libraries = match state.repo.list_libraries().await {
                Ok(l) => l,
                Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
            };
            let library = match libraries.iter().find(|l| l.name == lib_name) {
                Some(l) => l,
                None => return (StatusCode::NOT_FOUND, "Library not found").into_response(),
            };

            let authors = match state.repo.list_authors().await {
                Ok(a) => a,
                Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
            };

            let mut responses = vec![DavResponse {
                href: format!("{}/{}", href_base, url_encode(&lib_name)),
                propstat: vec![PropStat {
                    prop: DavProp {
                        resource_type: Some(ResourceType::Collection),
                        content_type: None,
                        content_length: None,
                        display_name: Some(lib_name.clone()),
                        last_modified: Some(library.updated_at.to_string()),
                        creation_date: Some(library.created_at.to_string()),
                        etag: None,
                    },
                    status: "HTTP/1.1 200 OK".into(),
                }],
            }];

            if depth != "0" {
                for author in &authors {
                    let author_books = match state.repo.get_books_by_author(author.id).await {
                        Ok(b) => b,
                        Err(_) => continue,
                    };
                    let lib_id_str = library.id.to_string();
                    let author_books: Vec<&Book> = author_books.iter().filter(|b| b.library_id.to_string() == lib_id_str).collect();
                    if author_books.is_empty() {
                        continue;
                    }
                    responses.push(DavResponse {
                        href: format!("{}/{}/{}", href_base, url_encode(&lib_name), url_encode(&author.name)),
                        propstat: vec![PropStat {
                            prop: DavProp {
                                resource_type: Some(ResourceType::Collection),
                                content_type: None,
                                content_length: None,
                                display_name: Some(author.name.clone()),
                                last_modified: None,
                                creation_date: None,
                                etag: None,
                            },
                            status: "HTTP/1.1 200 OK".into(),
                        }],
                    });
                }
            }

            multistatus_response(&MultiStatus { responses })
        }
        2 => {
            // Author level: list books
            let lib_name = url_decode(segments[0]);
            let author_name = url_decode(segments[1]);

            let authors = match state.repo.list_authors().await {
                Ok(a) => a,
                Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
            };
            let author = match authors.iter().find(|a| a.name == author_name) {
                Some(a) => a,
                None => return (StatusCode::NOT_FOUND, "Author not found").into_response(),
            };

            let books = match state.repo.get_books_by_author(author.id).await {
                Ok(b) => b,
                Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
            };

            let mut responses = vec![DavResponse {
                href: format!("{}/{}/{}", href_base, url_encode(&lib_name), url_encode(&author_name)),
                propstat: vec![PropStat {
                    prop: DavProp {
                        resource_type: Some(ResourceType::Collection),
                        content_type: None,
                        content_length: None,
                        display_name: Some(author_name.clone()),
                        last_modified: None,
                        creation_date: None,
                        etag: None,
                    },
                    status: "HTTP/1.1 200 OK".into(),
                }],
            }];

            if depth != "0" {
                for book in &books {
                    let tracks = match state.repo.get_tracks_by_book(book.id).await {
                        Ok(t) => t,
                        Err(_) => continue,
                    };
                    responses.push(DavResponse {
                        href: format!("{}/{}/{}/{}", href_base, url_encode(&lib_name), url_encode(&author_name), url_encode(&book.title)),
                        propstat: vec![PropStat {
                            prop: DavProp {
                                resource_type: Some(ResourceType::Collection),
                                content_type: None,
                                content_length: None,
                                display_name: Some(book.title.clone()),
                                last_modified: Some(book.updated_at.to_string()),
                                creation_date: Some(book.created_at.to_string()),
                                etag: None,
                            },
                            status: "HTTP/1.1 200 OK".into(),
                        }],
                    });

                    // Also list tracks inside book directory
                    for track in &tracks {
                        let file_size = match tokio::fs::metadata(&track.file_path).await {
                            Ok(m) => m.len(),
                            Err(_) => 0,
                        };
                        responses.push(DavResponse {
                            href: format!("{}/{}/{}/{}/{}", href_base, url_encode(&lib_name), url_encode(&author_name), url_encode(&book.title), url_encode(&track.title)),
                            propstat: vec![PropStat {
                                prop: DavProp {
                                    resource_type: None,
                                    content_type: Some(track.mime_type.clone()),
                                    content_length: Some(file_size),
                                    display_name: Some(track.title.clone()),
                                    last_modified: Some(track.created_at.to_string()),
                                    creation_date: Some(track.created_at.to_string()),
                                    etag: Some(format!("\"{}\"", track.id)),
                                },
                                status: "HTTP/1.1 200 OK".into(),
                            }],
                        });
                    }
                }
            }

            multistatus_response(&MultiStatus { responses })
        }
        3 => {
            // Book level: list tracks
            let lib_name = url_decode(segments[0]);
            let author_name = url_decode(segments[1]);
            let book_title = url_decode(segments[2]);

            let books = match state.repo.search_books(&book_title, 100).await {
                Ok(b) => b,
                Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
            };
            let book = match books.iter().find(|b| b.title == book_title) {
                Some(b) => b,
                None => return (StatusCode::NOT_FOUND, "Book not found").into_response(),
            };

            let tracks = match state.repo.get_tracks_by_book(book.id).await {
                Ok(t) => t,
                Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
            };

            let mut responses = vec![DavResponse {
                href: format!("{}/{}/{}/{}", href_base, url_encode(&lib_name), url_encode(&author_name), url_encode(&book_title)),
                propstat: vec![PropStat {
                    prop: DavProp {
                        resource_type: Some(ResourceType::Collection),
                        content_type: None,
                        content_length: None,
                        display_name: Some(book_title.clone()),
                        last_modified: Some(book.updated_at.to_string()),
                        creation_date: Some(book.created_at.to_string()),
                        etag: None,
                    },
                    status: "HTTP/1.1 200 OK".into(),
                }],
            }];

            if depth != "0" {
                for track in &tracks {
                    let file_size = match tokio::fs::metadata(&track.file_path).await {
                        Ok(m) => m.len(),
                        Err(_) => 0,
                    };
                    responses.push(DavResponse {
                        href: format!("{}/{}/{}/{}/{}", href_base, url_encode(&lib_name), url_encode(&author_name), url_encode(&book_title), url_encode(&track.title)),
                        propstat: vec![PropStat {
                            prop: DavProp {
                                resource_type: None,
                                content_type: Some(track.mime_type.clone()),
                                content_length: Some(file_size),
                                display_name: Some(track.title.clone()),
                                last_modified: Some(track.created_at.to_string()),
                                creation_date: Some(track.created_at.to_string()),
                                etag: Some(format!("\"{}\"", track.id)),
                            },
                            status: "HTTP/1.1 200 OK".into(),
                        }],
                    });
                }
            }

            multistatus_response(&MultiStatus { responses })
        }
        _ => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}

async fn get_file(state: &WebDavState, dav_path: &str, req: Request) -> Response {
    let track = match resolve_track(state, dav_path).await {
        Ok(Some(t)) => t,
        Ok(None) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
        Err(e) => {
            tracing::error!("WebDAV GET error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    let path = std::path::Path::new(&track.file_path);
    if !path.exists() {
        return (StatusCode::NOT_FOUND, "File not found on disk").into_response();
    }

    let file_size = match tokio::fs::metadata(&track.file_path).await {
        Ok(m) => m.len(),
        Err(_) => return (StatusCode::NOT_FOUND, "Cannot read file").into_response(),
    };

    let range = req
        .headers()
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
        if file.read_exact(&mut buffer).await.is_err() {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Read error").into_response();
        }
        Response::builder()
            .status(StatusCode::PARTIAL_CONTENT)
            .header(header::CONTENT_TYPE, &track.mime_type)
            .header(header::CONTENT_LENGTH, content_length)
            .header(header::CONTENT_RANGE, format!("bytes {}-{}/{}", start, end, file_size))
            .header(header::ACCEPT_RANGES, "bytes")
            .body(Body::from(buffer))
            .unwrap()
            .into_response()
    } else {
        let mut buffer = Vec::with_capacity(file_size as usize);
        if file.read_to_end(&mut buffer).await.is_err() {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Read error").into_response();
        }
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, &track.mime_type)
            .header(header::CONTENT_LENGTH, file_size)
            .header(header::ACCEPT_RANGES, "bytes")
            .body(Body::from(buffer))
            .unwrap()
            .into_response()
    }
}

async fn head_file(state: &WebDavState, dav_path: &str) -> Response {
    let track = match resolve_track(state, dav_path).await {
        Ok(Some(t)) => t,
        Ok(None) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
    };

    let file_size = match tokio::fs::metadata(&track.file_path).await {
        Ok(m) => m.len(),
        Err(_) => return (StatusCode::NOT_FOUND, "File not found on disk").into_response(),
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &track.mime_type)
        .header(header::CONTENT_LENGTH, file_size)
        .header(header::ACCEPT_RANGES, "bytes")
        .body(Body::empty())
        .unwrap()
        .into_response()
}

async fn put_file(_state: &WebDavState, _dav_path: &str, _req: Request) -> Response {
    // Read-only WebDAV for now
    (StatusCode::FORBIDDEN, "Read-only WebDAV").into_response()
}

async fn mkcol(_state: &WebDavState, _dav_path: &str) -> Response {
    (StatusCode::FORBIDDEN, "Read-only WebDAV").into_response()
}

async fn delete_file(_state: &WebDavState, _dav_path: &str) -> Response {
    (StatusCode::FORBIDDEN, "Read-only WebDAV").into_response()
}

async fn resolve_track(state: &WebDavState, dav_path: &str) -> anyhow::Result<Option<Track>> {
    let segments: Vec<&str> = dav_path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.len() < 4 {
        return Ok(None);
    }

    let book_title = url_decode(segments[2]);
    let track_name = url_decode(segments[3]);

    let books = state.repo.search_books(&book_title, 100).await?;
    let book = match books.iter().find(|b| b.title == book_title) {
        Some(b) => b,
        None => return Ok(None),
    };

    let tracks = state.repo.get_tracks_by_book(book.id).await?;
    Ok(tracks.into_iter().find(|t| t.title == track_name))
}

fn multistatus_response(ms: &MultiStatus) -> Response {
    let xml = ms.to_xml();
    Response::builder()
        .status(StatusCode::MULTI_STATUS)
        .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
        .header(header::CONTENT_LENGTH, xml.len())
        .body(Body::from(xml))
        .unwrap()
        .into_response()
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

fn url_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '%' => result.push_str("%25"),
            ' ' => result.push_str("%20"),
            '#' => result.push_str("%23"),
            '&' => result.push_str("%26"),
            '+' => result.push_str("%2B"),
            '?' => result.push_str("%3F"),
            _ => result.push(c),
        }
    }
    result
}

fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else {
            result.push(c);
        }
    }
    result
}

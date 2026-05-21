use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tonevault_core::models::library::{CreateLibrary, Library, SourceType, UpdateLibrary};
use tonevault_db::Database;

#[derive(Debug, Deserialize)]
pub struct CreateLibraryRequest {
    pub name: String,
    pub root_path: String,
    #[serde(default)]
    pub source_type: SourceType,
    pub base_url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub scan_interval: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateLibraryRequest {
    pub name: Option<String>,
    pub root_path: Option<String>,
    pub source_type: Option<SourceType>,
    pub base_url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub scan_interval: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct LibraryResponse {
    pub id: i64,
    pub name: String,
    pub root_path: String,
    pub source_type: SourceType,
    pub base_url: Option<String>,
    pub username: Option<String>,
    pub has_password: bool,
    pub scan_interval: Option<i64>,
    pub last_scan: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Library> for LibraryResponse {
    fn from(lib: Library) -> Self {
        Self {
            id: lib.id,
            name: lib.name,
            root_path: lib.root_path,
            source_type: lib.source_type,
            base_url: lib.base_url,
            username: lib.username,
            has_password: lib.password.is_some(),
            last_scan: lib.last_scan.map(|dt| dt.to_string()),
            created_at: lib.created_at.to_string(),
            updated_at: lib.updated_at.to_string(),
            scan_interval: lib.scan_interval,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct WebDavConnectRequest {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct WebDavEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

pub async fn list_libraries(State(db): State<Database>) -> impl IntoResponse {
    match db.list_libraries().await {
        Ok(libraries) => Json(
            libraries
                .into_iter()
                .map(LibraryResponse::from)
                .collect::<Vec<_>>(),
        )
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn get_library(
    State(db): State<Database>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match db.get_library(id).await {
        Ok(Some(library)) => Json(LibraryResponse::from(library)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Library not found" }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn create_library(
    State(db): State<Database>,
    Json(req): Json<CreateLibraryRequest>,
) -> impl IntoResponse {
    let input = CreateLibrary {
        name: req.name,
        root_path: req.root_path,
        source_type: req.source_type,
        base_url: req.base_url,
        username: req.username,
        password: req.password,
        scan_interval: req.scan_interval,
    };
    match db.create_library(&input).await {
        Ok(library) => (StatusCode::CREATED, Json(LibraryResponse::from(library))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn update_library(
    State(db): State<Database>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateLibraryRequest>,
) -> impl IntoResponse {
    let input = UpdateLibrary {
        name: req.name,
        root_path: req.root_path,
        source_type: req.source_type,
        base_url: req.base_url,
        username: req.username,
        password: req.password,
        scan_interval: req.scan_interval,
    };
    match db.update_library(id, &input).await {
        Ok(library) => Json(LibraryResponse::from(library)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn delete_library(
    State(db): State<Database>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match db.delete_library(id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn scan_library(
    State(db): State<Database>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match db.get_library(id).await {
        Ok(Some(_library)) => {
            // Trigger scan in background
            let db_clone = db.clone();
            tokio::spawn(async move {
                let _ = db_clone.update_last_scan(id).await;
            });
            Json(serde_json::json!({ "status": "scanning" })).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Library not found" }))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn webdav_connect(
    State(_db): State<Database>,
    Json(req): Json<WebDavConnectRequest>,
) -> impl IntoResponse {
    // Validate WebDAV connection by trying to list the root
    let client = reqwest::Client::new();
    let url = req.url.trim_end_matches('/').to_string();

    let response = client
        .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
        .header("Depth", "1")
        .basic_auth(&req.username, Some(&req.password))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 207 => {
            // Parse WebDAV response to list entries
            let body = resp.text().await.unwrap_or_default();
            let entries = parse_webdav_response(&body, &url);
            Json(serde_json::json!({ "status": "connected", "entries": entries })).into_response()
        }
        Ok(resp) => {
            let status = resp.status().as_u16();
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("WebDAV server returned status {}", status) })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": format!("Failed to connect: {}", e) })),
        )
            .into_response(),
    }
}

pub async fn webdav_list(
    State(_db): State<Database>,
    Json(req): Json<WebDavConnectRequest>,
) -> impl IntoResponse {
    let client = reqwest::Client::new();
    let url = req.url.trim_end_matches('/').to_string();

    let response = client
        .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
        .header("Depth", "1")
        .basic_auth(&req.username, Some(&req.password))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 207 => {
            let body = resp.text().await.unwrap_or_default();
            let entries = parse_webdav_response(&body, &url);
            Json(entries).into_response()
        }
        Ok(resp) => {
            let status = resp.status().as_u16();
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({ "error": format!("WebDAV server returned status {}", status) })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": format!("Failed to connect: {}", e) })),
        )
            .into_response(),
    }
}

fn parse_webdav_response(body: &str, base_url: &str) -> Vec<WebDavEntry> {
    let mut entries = Vec::new();

    // Simple XML parsing for WebDAV PROPFIND response
    let base_path = url::Url::parse(base_url)
        .map(|u| u.path().to_string())
        .unwrap_or_default();

    for response_block in body.split("<d:response>").skip(1) {
        let href = extract_xml_tag(response_block, "d:href")
            .or_else(|| extract_xml_tag(response_block, "D:href"))
            .unwrap_or_default();

        if href.is_empty() || href == base_path || href == format!("{}/", base_path) {
            continue;
        }

        let is_dir = response_block.contains("<d:collection/>")
            || response_block.contains("<D:collection/>")
            || href.ends_with('/');

        let name = href
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("unknown")
            .to_string();

        // URL decode the name
        let name = percent_encoding::percent_decode_str(&name)
            .decode_utf8_lossy()
            .to_string();

        entries.push(WebDavEntry {
            name,
            path: href,
            is_dir,
        });
    }

    // Sort: directories first, then files
    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name))
    });

    entries
}

fn extract_xml_tag(content: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = content.find(&open)?;
    let inner = &content[start + open.len()..];
    let end = inner.find(&close)?;
    Some(inner[..end].to_string())
}

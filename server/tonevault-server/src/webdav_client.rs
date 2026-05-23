use axum::Router;
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/webdav/connect", axum::routing::post(webdav_connect))
        .route("/api/v1/webdav/list", axum::routing::post(webdav_list))
}

#[derive(Deserialize)]
pub struct WebDavRequest {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct WebDavConnectResponse {
    pub status: String,
    pub entries: Vec<WebDavEntry>,
}

#[derive(Serialize)]
pub struct WebDavEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<i64>,
}

pub async fn webdav_connect(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<WebDavRequest>,
) -> Result<Json<WebDavConnectResponse>, (StatusCode, String)> {
    let entries = propfind(&req.url, &req.username, &req.password).await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(Json(WebDavConnectResponse {
        status: "connected".to_string(),
        entries,
    }))
}

pub async fn webdav_list(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<WebDavRequest>,
) -> Result<Json<Vec<WebDavEntry>>, (StatusCode, String)> {
    let entries = propfind(&req.url, &req.username, &req.password).await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(Json(entries))
}

async fn propfind(url: &str, username: &str, password: &str) -> Result<Vec<WebDavEntry>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let body = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\
        <D:propfind xmlns:D=\"DAV:\">\
        <D:prop>\
        <D:resourcetype/>\
        <D:displayname/>\
        <D:getcontentlength/>\
        <D:getlastmodified/>\
        </D:prop>\
        </D:propfind>";

    let response = client
        .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), url)
        .header("Depth", "1")
        .header("Content-Type", "application/xml; charset=utf-8")
        .basic_auth(username, Some(password))
        .body(body)
        .send()
        .await?;

    if !response.status().is_success() && response.status().as_u16() != 207 {
        return Err(format!("WebDAV server returned status {}", response.status()).into());
    }

    let xml_body = response.text().await?;
    parse_webdav_response(&xml_body, url)
}

fn parse_webdav_response(xml: &str, base_url: &str) -> Result<Vec<WebDavEntry>, Box<dyn std::error::Error>> {
    let base_url_normalized = base_url.trim_end_matches('/');

    let mut result = Vec::new();

    // Split by <D:response> tags
    let responses: Vec<&str> = xml.split("<D:response>").collect();

    for resp in &responses[1..] {
        let href = extract_xml_value(resp, "D:href")
            .or_else(|| extract_xml_value(resp, "href"))
            .unwrap_or_default();

        let display_name = extract_xml_value(resp, "D:displayname")
            .or_else(|| extract_xml_value(resp, "displayname"));

        let is_collection = resp.contains("<D:collection/>") || resp.contains("<D:collection />");

        let content_length = extract_xml_value(resp, "D:getcontentlength")
            .or_else(|| extract_xml_value(resp, "getcontentlength"))
            .and_then(|s| s.parse::<i64>().ok());

        // Skip the parent directory (href matches base_url)
        let href_normalized = href.trim_end_matches('/');
        if href_normalized == base_url_normalized {
            continue;
        }

        // Derive name from displayname or href path
        let name = display_name.unwrap_or_else(|| {
            href_normalized
                .rsplit('/')
                .next()
                .unwrap_or("")
                .to_string()
        });

        // Derive path from href
        let path = if href_normalized.starts_with("http") {
            href_normalized.to_string()
        } else {
            // Relative href - construct full URL
            format!("{}/{}", base_url_normalized, href_normalized.trim_start_matches('/'))
        };

        result.push(WebDavEntry {
            name,
            path,
            is_dir: is_collection,
            size: content_length,
        });
    }

    Ok(result)
}

fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
    // Handle both <D:tag>value</D:tag> and <tag>value</tag>
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);

    if let Some(start) = xml.find(&open) {
        let content_start = start + open.len();
        if let Some(end) = xml[content_start..].find(&close) {
            return Some(xml[content_start..content_start + end].trim().to_string());
        }
    }
    None
}
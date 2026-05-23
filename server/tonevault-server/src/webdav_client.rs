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
    // Parse base URL to get the origin (scheme + host + port)
    let base_url_normalized = base_url.trim_end_matches('/');
    let origin = extract_origin(base_url_normalized);
    let base_path = extract_path(base_url_normalized);

    let mut result = Vec::new();

    // Split by <D:response> tags
    let responses: Vec<&str> = xml.split("<D:response>").collect();

    for resp in &responses[1..] {
        let href = extract_xml_value(resp, "D:href")
            .or_else(|| extract_xml_value(resp, "href"))
            .unwrap_or_default();

        let is_collection = resp.contains("<D:collection></D:collection>")
            || resp.contains("<D:collection />")
            || resp.contains("<D:collection/>");

        let content_length = extract_xml_value(resp, "D:getcontentlength")
            .or_else(|| extract_xml_value(resp, "getcontentlength"))
            .and_then(|s| s.parse::<i64>().ok());

        // Normalize href
        let href_normalized = href.trim_end_matches('/');

        // Skip the parent directory (href matches base path)
        if href_normalized == base_path || href_normalized.is_empty() {
            continue;
        }

        // Extract name from the last path segment of href
        let name = href_normalized
            .rsplit('/')
            .next()
            .unwrap_or("")
            .to_string();
        let name = percent_decode(&name);

        // Build full path from href
        let path = if href.starts_with('/') {
            // Relative href
            format!("{}{}", origin, href.trim_end_matches('/'))
        } else {
            href_normalized.to_string()
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

fn extract_origin(url: &str) -> String {
    // Extract scheme://host:port from URL
    if let Some(idx) = url.find("://") {
        let after_scheme = &url[idx + 3..];
        if let Some(slash_idx) = after_scheme.find('/') {
            return url[..idx + 3 + slash_idx].to_string();
        }
    }
    url.to_string()
}

fn extract_path(url: &str) -> String {
    if let Some(idx) = url.find("://") {
        let after_scheme = &url[idx + 3..];
        if let Some(slash_idx) = after_scheme.find('/') {
            return after_scheme[slash_idx..].to_string();
        }
    }
    "/".to_string()
}

fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);

    if let Some(start) = xml.find(&open) {
        let content_start = start + open.len();
        if let Some(end) = xml[content_start..].find(&close) {
            let value = xml[content_start..content_start + end].trim().to_string();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

fn percent_decode(s: &str) -> String {
    let mut result = String::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = &s[i + 1..i + 3];
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                result.push(byte as char);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}
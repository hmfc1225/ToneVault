use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Local,
    #[serde(rename = "webdav")]
    WebDav,
    Rss,
}

impl Default for SourceType {
    fn default() -> Self {
        Self::Local
    }
}

impl SourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::WebDav => "webdav",
            Self::Rss => "rss",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "local" => Some(Self::Local),
            "webdav" => Some(Self::WebDav),
            "rss" => Some(Self::Rss),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub id: i64,
    pub name: String,
    pub root_path: String,
    pub source_type: SourceType,
    pub base_url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub scan_interval: Option<i64>,
    pub last_scan: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLibrary {
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
pub struct UpdateLibrary {
    pub name: Option<String>,
    pub root_path: Option<String>,
    pub source_type: Option<SourceType>,
    pub base_url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub scan_interval: Option<i64>,
}
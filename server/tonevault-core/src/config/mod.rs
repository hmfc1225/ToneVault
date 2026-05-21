use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub scanner: ScannerConfig,
    pub webdav: WebDavConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub engine: DatabaseEngine,
    #[serde(default = "default_sqlite_path")]
    pub sqlite_path: String,
    #[serde(default)]
    pub postgres_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseEngine {
    Sqlite,
    Postgres,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiry_hours")]
    pub jwt_expiry_hours: u64,
    #[serde(default = "default_refresh_expiry_days")]
    pub refresh_expiry_days: u64,
    #[serde(default = "default_api_key_prefix")]
    pub api_key_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    #[serde(default = "default_true")]
    pub scan_on_startup: bool,
    #[serde(default = "default_true")]
    pub watch_for_changes: bool,
    #[serde(default = "default_scan_interval")]
    pub scan_interval_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebDavConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_webdav_mount")]
    pub mount_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".into(),
                port: 8080,
                base_url: None,
            },
            database: DatabaseConfig {
                engine: DatabaseEngine::Sqlite,
                sqlite_path: default_sqlite_path(),
                postgres_url: None,
            },
            auth: AuthConfig {
                jwt_secret: default_jwt_secret(),
                jwt_expiry_hours: default_jwt_expiry_hours(),
                refresh_expiry_days: default_refresh_expiry_days(),
                api_key_prefix: default_api_key_prefix(),
            },
            scanner: ScannerConfig {
                scan_on_startup: true,
                watch_for_changes: true,
                scan_interval_minutes: default_scan_interval(),
            },
            webdav: WebDavConfig {
                enabled: true,
                mount_path: default_webdav_mount(),
            },
            logging: LoggingConfig {
                level: default_log_level(),
            },
        }
    }
}

impl AppConfig {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_or_default(path: Option<&std::path::Path>) -> anyhow::Result<Self> {
        match path {
            Some(p) if p.exists() => Self::load(p),
            _ => {
                tracing::info!("No config file found, using defaults");
                Ok(Self::default())
            }
        }
    }
}

fn default_sqlite_path() -> String {
    "data/tonevault.db".into()
}

fn default_jwt_secret() -> String {
    "change-me-in-production".into()
}

fn default_jwt_expiry_hours() -> u64 {
    1
}

fn default_refresh_expiry_days() -> u64 {
    7
}

fn default_api_key_prefix() -> String {
    "tv_".into()
}

fn default_scan_interval() -> u64 {
    60
}

fn default_webdav_mount() -> String {
    "/dav".into()
}

fn default_log_level() -> String {
    "info".into()
}

fn default_true() -> bool {
    true
}

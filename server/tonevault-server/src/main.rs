mod books;
mod library;
mod scan;
mod stream;
mod webdav_auth;

use anyhow::Result;
use axum::extract::{FromRef, State};
use axum::http::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::{any, delete, get, post, put};
use axum::Router;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use tonevault_auth::{AuthState, JwtManager};
use tonevault_core::config::AppConfig;
use tonevault_db::repository::Repository;

#[cfg(feature = "sqlite")]
use tonevault_db::SqliteRepository;
#[cfg(feature = "postgres")]
use tonevault_db::PostgresRepository;

#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn Repository>,
    pub jwt: Arc<JwtManager>,
    pub config: AppConfig,
}

impl FromRef<AppState> for AuthState {
    fn from_ref(state: &AppState) -> Self {
        AuthState { repo: state.repo.clone(), jwt: state.jwt.clone() }
    }
}

impl FromRef<AppState> for library::LibraryState {
    fn from_ref(state: &AppState) -> Self {
        library::LibraryState { repo: state.repo.clone() }
    }
}

impl FromRef<AppState> for scan::ScanState {
    fn from_ref(state: &AppState) -> Self {
        scan::ScanState { repo: state.repo.clone() }
    }
}

impl FromRef<AppState> for books::BookState {
    fn from_ref(state: &AppState) -> Self {
        books::BookState { repo: state.repo.clone() }
    }
}

impl FromRef<AppState> for stream::StreamState {
    fn from_ref(state: &AppState) -> Self {
        stream::StreamState { repo: state.repo.clone() }
    }
}

impl FromRef<AppState> for tonevault_webdav::WebDavState {
    fn from_ref(state: &AppState) -> Self {
        tonevault_webdav::WebDavState { repo: state.repo.clone(), base_path: "/dav".to_string() }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load(std::path::Path::new("config/tonevault.toml")).unwrap_or_default();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("ToneVault starting on {}:{}", config.server.host, config.server.port);

    let repo = init_database(&config).await?;
    tracing::info!("Database initialized and migrations applied");
    let jwt = Arc::new(JwtManager::new(
        config.auth.jwt_secret.clone(),
        config.auth.jwt_expiry_hours as i64,
        config.auth.refresh_expiry_days as i64,
    ));

    let state = AppState { repo, jwt, config: config.clone() };

    let auth_routes = Router::new()
        .route("/setup/status", get(tonevault_auth::setup_status))
        .route("/setup", post(tonevault_auth::setup))
        .route("/login", post(tonevault_auth::login))
        .route("/refresh", post(tonevault_auth::refresh))
        .route("/me", get(tonevault_auth::me_full));

    let library_routes = Router::new()
        .route("/", post(library::create_library).get(library::list_libraries))
        .route("/{id}", get(library::get_library).put(library::update_library).delete(library::delete_library))
        .route("/{id}/scan", post(scan::trigger_scan))
        .route("/{id}/scan/status", get(scan::scan_status));

    let book_routes = Router::new()
        .route("/", get(books::list_books))
        .route("/{id}", get(books::get_book).put(books::update_book).delete(books::delete_book))
        .route("/{id}/authors", get(books::get_book_authors))
        .route("/{id}/series", get(books::get_book_series))
        .route("/{id}/tracks", get(books::get_book_tracks));

    let author_routes = Router::new()
        .route("/", get(books::list_authors))
        .route("/{id}/books", get(books::get_author_books));

    let series_routes = Router::new()
        .route("/", get(books::list_series))
        .route("/{id}/books", get(books::get_series_books));

    let position_routes = Router::new()
        .route("/{user_id}", put(books::upsert_position).get(books::get_user_positions))
        .route("/{user_id}/{book_id}", get(books::get_position));

    let bookmark_routes = Router::new()
        .route("/{user_id}", post(books::create_bookmark).get(books::list_all_user_bookmarks))
        .route("/{user_id}/{book_id}", get(books::list_bookmarks))
        .route("/item/{id}", delete(books::delete_bookmark));

    let collection_routes = Router::new()
        .route("/{user_id}", post(books::create_collection).get(books::list_collections))
        .route("/item/{id}", delete(books::delete_collection))
        .route("/{collection_id}/books/{book_id}", post(books::add_book_to_collection).delete(books::remove_book_from_collection))
        .route("/{id}/books", get(books::get_collection_books));

    let api_routes = Router::new()
        .route("/health", get(health))
        .nest("/api/v1/auth", auth_routes)
        .nest("/api/v1/libraries", library_routes)
        .nest("/api/v1/books", book_routes)
        .nest("/api/v1/authors", author_routes)
        .nest("/api/v1/series", series_routes)
        .nest("/api/v1/positions", position_routes)
        .nest("/api/v1/bookmarks", bookmark_routes)
        .nest("/api/v1/collections", collection_routes)
        .route("/api/v1/search", get(books::search_books))
        .route("/api/v1/tracks/{id}/stream", get(stream::stream_track))
        .route("/api/v1/books/{id}/cover", get(stream::get_book_cover))
        .route("/api/v1/system/info", get(system_info))
        .layer(CorsLayer::permissive());

    let dav_routes = Router::new()
        .route("/dav", any(tonevault_webdav::handler::webdav_handler))
        .route("/dav/{*path}", any(tonevault_webdav::handler::webdav_handler))
        .layer(axum::middleware::from_fn_with_state(state.clone(), webdav_auth::basic_auth_middleware));

    let app = Router::new()
        .merge(api_routes)
        .merge(dav_routes)
        .layer(axum::middleware::from_fn_with_state(state.clone(), inject_jwt))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Serve frontend static files in production (if dist/ exists)
    let dist_path = PathBuf::from("web/dist");
    if dist_path.exists() {
        let index_path = dist_path.join("index.html");
        let app = app
            .nest_service("/assets", ServeDir::new(dist_path.join("assets")))
            .fallback_service(ServeFile::new(index_path));
        tracing::info!("Serving frontend from {}", dist_path.display());

        let addr = format!("{}:{}", config.server.host, config.server.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        tracing::info!("ToneVault listening on {}", addr);
        axum::serve(listener, app).await?;
    } else {
        let addr = format!("{}:{}", config.server.host, config.server.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        tracing::info!("ToneVault listening on {} (no frontend dist/ found)", addr);
        axum::serve(listener, app).await?;
    }

    Ok(())
}

async fn inject_jwt(
    State(state): State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> impl IntoResponse {
    req.extensions_mut().insert((*state.jwt).clone());
    next.run(req).await
}

async fn init_database(config: &AppConfig) -> Result<Arc<dyn Repository>> {
    match config.database.engine {
        tonevault_core::config::DatabaseEngine::Sqlite => {
            #[cfg(feature = "sqlite")]
            {
                let db_url = if config.database.sqlite_path.contains("://") {
                    config.database.sqlite_path.clone()
                } else {
                    let path = PathBuf::from(&config.database.sqlite_path);
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    format!("sqlite:{}?mode=rwc", config.database.sqlite_path)
                };
                let db = SqliteRepository::new(&db_url).await?;
                db.run_migrations().await?;
                Ok(Arc::new(db))
            }
            #[cfg(not(feature = "sqlite"))]
            {
                Err(anyhow::anyhow!("SQLite support not compiled in. Build with --features sqlite"))
            }
        }
        tonevault_core::config::DatabaseEngine::Postgres => {
            #[cfg(feature = "postgres")]
            {
                let db_url = config.database.postgres_url.as_ref()
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("PostgreSQL URL not configured"))?;
                let db = PostgresRepository::new(&db_url).await?;
                db.run_migrations().await?;
                Ok(Arc::new(db))
            }
            #[cfg(not(feature = "postgres"))]
            {
                Err(anyhow::anyhow!("PostgreSQL support not compiled in. Build with --features postgres"))
            }
        }
    }
}

async fn health() -> impl IntoResponse {
    #[derive(Serialize)]
    struct Health { status: String, version: String }
    axum::Json(Health { status: "ok".into(), version: env!("CARGO_PKG_VERSION").into() })
}

async fn system_info(State(state): State<AppState>) -> impl IntoResponse {
    let library_count = state.repo.list_libraries().await.map(|l| l.len()).unwrap_or(0);
    #[derive(Serialize)]
    struct Info { version: String, database_engine: String, library_count: usize, webdav_enabled: bool }
    axum::Json(Info {
        version: env!("CARGO_PKG_VERSION").into(),
        database_engine: format!("{:?}", state.config.database.engine),
        library_count,
        webdav_enabled: state.config.webdav.enabled,
    })
}

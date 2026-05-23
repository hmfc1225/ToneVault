use std::sync::Arc;

use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use tonevault_auth::{AuthState, JwtManager, setup_status, setup, login, refresh, me_full};
use tonevault_db::{Database, Repository, SqliteRepository};

mod books;
mod library;
mod scan;
mod stream;

pub struct AppState {
    repo: Arc<dyn Repository>,
    jwt: Arc<JwtManager>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./data/tonevault.db?mode=rwc".to_string());

    let sqlite_repo = SqliteRepository::new(&database_url).await?;
    sqlite_repo.run_migrations().await?;

    let db = sqlite_repo.database().clone();
    let repo: Arc<dyn Repository> = Arc::new(db);

    let jwt_secret = std::env::var("TONEVAULT_AUTH_SECRET")
        .unwrap_or_else(|_| "change-me-in-production".to_string());
    let jwt = Arc::new(JwtManager::new(jwt_secret, 1, 7));

    let auth_state = AuthState {
        repo: repo.clone(),
        jwt: jwt.clone(),
    };

    let state = Arc::new(AppState {
        repo: repo.clone(),
        jwt: jwt.clone(),
    });

    let web_dir = std::env::var("TONEVAULT_WEB_DIR")
        .unwrap_or_else(|_| "./web/dist".to_string());
    let index_path = std::path::PathBuf::from(format!("{}/index.html", web_dir));

    let auth_routes = Router::new()
        .route("/api/v1/auth/setup/status", axum::routing::get(setup_status))
        .route("/api/v1/auth/setup", axum::routing::post(setup))
        .route("/api/v1/auth/login", axum::routing::post(login))
        .route("/api/v1/auth/refresh", axum::routing::post(refresh))
        .route("/api/v1/auth/me", axum::routing::get(me_full))
        .with_state(auth_state);

    let api_routes = Router::new()
        .merge(books::router())
        .merge(library::router())
        .merge(stream::router());

    let app = Router::new()
        .merge(auth_routes)
        .merge(api_routes)
        .fallback_service(
            ServeDir::new(&web_dir).not_found_service(ServeFile::new(index_path))
        )
        .layer(axum::Extension((*jwt).clone()))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

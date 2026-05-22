use std::sync::Arc;

use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use tonevault_db::{Database, Repository, SqliteRepository};

mod books;
mod library;
mod scan;
mod stream;
mod webdav_auth;

pub struct AppState {
    repo: Arc<dyn Repository>,
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

    let state = Arc::new(AppState { repo });

    let app = Router::new()
        .merge(books::router())
        .merge(library::router())
        .merge(stream::router())
        .merge(webdav_auth::router())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}

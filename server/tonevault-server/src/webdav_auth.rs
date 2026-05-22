use axum::Router;
use axum::extract::{State, Json};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use tonevault_db::Repository;
use tonevault_core::models::User;

use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/auth/login", axum::routing::post(login))
        .route("/api/v1/auth/register", axum::routing::post(register))
        .route("/api/v1/auth/me", axum::routing::get(get_me))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub role: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let user = match state.repo.get_user_by_username(&req.username).await {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid credentials"}))).into_response(),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Database error"}))).into_response();
        }
    };

    // Simple password check (in production, use proper hashing)
    if user.password_hash != req.password {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid credentials"}))).into_response();
    }

    let token = format!("token-{}", user.id);
    (StatusCode::OK, Json(AuthResponse {
        token,
        user: UserResponse {
            id: user.id.to_string(),
            username: user.username,
            display_name: user.display_name,
            role: match user.role {
                tonevault_core::models::UserRole::Admin => "admin",
                tonevault_core::models::UserRole::User => "user",
                tonevault_core::models::UserRole::Guest => "guest",
            }.to_string(),
        },
    })).into_response()
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    if state.repo.get_user_by_username(&req.username).await.ok().flatten().is_some() {
        return (StatusCode::CONFLICT, Json(serde_json::json!({"error": "Username already exists"}))).into_response();
    }

    let create = tonevault_core::models::CreateUser {
        username: req.username,
        display_name: req.display_name,
        email: req.email,
        password: req.password,
        role: tonevault_core::models::UserRole::User,
    };

    match state.repo.create_user(&create).await {
        Ok(user) => {
            let token = format!("token-{}", user.id);
            (StatusCode::CREATED, Json(AuthResponse {
                token,
                user: UserResponse {
                    id: user.id.to_string(),
                    username: user.username,
                    display_name: user.display_name,
                    role: "user".to_string(),
                },
            })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create user: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Failed to create user"}))).into_response()
        }
    }
}

pub async fn get_me(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // TODO: Extract user from auth token
    (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Not implemented"}))).into_response()
}
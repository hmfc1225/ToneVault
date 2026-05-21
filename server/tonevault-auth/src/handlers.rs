use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use tonevault_core::models::*;

use crate::jwt::JwtManager;
use crate::middleware::AuthUser;
use crate::password::{hash_password, verify_password};

// Re-export AppState from the server crate — we define our own trait instead
// to avoid circular dependency. The handlers just need repo + jwt access.
use std::sync::Arc;
use tonevault_db::repository::Repository;

#[derive(Clone)]
pub struct AuthState {
    pub repo: Arc<dyn Repository>,
    pub jwt: Arc<JwtManager>,
}

#[derive(Deserialize)]
pub struct SetupRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserPublic,
}

fn role_str(role: &UserRole) -> String {
    match role {
        UserRole::Admin => "admin".to_string(),
        UserRole::User => "user".to_string(),
        UserRole::Guest => "guest".to_string(),
    }
}

pub async fn setup_status(State(state): State<AuthState>) -> impl IntoResponse {
    let users = state.repo.list_users().await.unwrap_or_default();
    let needs_setup = users.is_empty();
    Json(serde_json::json!({ "needs_setup": needs_setup }))
}

pub async fn setup(
    State(state): State<AuthState>,
    Json(req): Json<SetupRequest>,
) -> impl IntoResponse {
    // Check if any admin already exists
    match state.repo.list_users().await {
        Ok(users) if !users.is_empty() => {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({"error": "Server already set up"})),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to check existing users: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error"})),
            )
                .into_response();
        }
        _ => {}
    }

    let password_hash = hash_password(&req.password);

    let create_user = CreateUser {
        username: req.username,
        email: Some(req.email),
        password: password_hash,
        display_name: req.display_name,
        role: UserRole::Admin,
    };

    let user = match state.repo.create_user(&create_user).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Failed to create admin user: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create user"})),
            )
                .into_response();
        }
    };

    let role = role_str(&user.role);
    let access_token = match state.jwt.create_access_token(user.id, &role) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to create access token: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Token creation failed"})),
            )
                .into_response();
        }
    };

    let refresh_token = match state.jwt.create_refresh_token(user.id) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to create refresh token: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Token creation failed"})),
            )
                .into_response();
        }
    };

    let resp = AuthResponse {
        access_token,
        refresh_token,
        user: UserPublic {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            email: user.email,
            role: user.role,
        },
    };

    (StatusCode::CREATED, Json(resp)).into_response()
}

pub async fn login(
    State(state): State<AuthState>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let user = match state.repo.get_user_by_username(&req.username).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid credentials"})),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Database error during login: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Database error"})),
            )
                .into_response();
        }
    };

    if !user.is_active {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Account disabled"})),
        )
            .into_response();
    }

    if !verify_password(&req.password, &user.password_hash) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid credentials"})),
        )
            .into_response();
    }

    let role = role_str(&user.role);
    let access_token = match state.jwt.create_access_token(user.id, &role) {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Token creation failed"})),
            )
                .into_response();
        }
    };

    let refresh_token = match state.jwt.create_refresh_token(user.id) {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Token creation failed"})),
            )
                .into_response();
        }
    };

    let resp = AuthResponse {
        access_token,
        refresh_token,
        user: UserPublic {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            email: user.email,
            role: user.role,
        },
    };

    (StatusCode::OK, Json(resp)).into_response()
}

pub async fn refresh(
    State(state): State<AuthState>,
    Json(req): Json<RefreshRequest>,
) -> impl IntoResponse {
    let claims = match state.jwt.validate_refresh_token(&req.refresh_token) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid token"})),
            )
                .into_response();
        }
    };

    let access_token = match state.jwt.create_access_token(user_id, &claims.role) {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Token creation failed"})),
            )
                .into_response();
        }
    };

    let refresh_token = match state.jwt.create_refresh_token(user_id) {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Token creation failed"})),
            )
                .into_response();
        }
    };

    #[derive(Serialize)]
    struct RefreshResponse {
        access_token: String,
        refresh_token: String,
    }

    (StatusCode::OK, Json(RefreshResponse { access_token, refresh_token })).into_response()
}

pub async fn me_full(
    auth: AuthUser,
    State(state): State<AuthState>,
) -> impl IntoResponse {
    match state.repo.get_user_by_id(auth.user_id).await {
        Ok(Some(user)) => {
            #[derive(Serialize)]
            struct MeResponse {
                id: String,
                username: String,
                display_name: Option<String>,
                email: Option<String>,
                role: String,
            }
            Json(MeResponse {
                id: user.id.to_string(),
                username: user.username,
                display_name: user.display_name,
                email: user.email,
                role: role_str(&user.role),
            })
            .into_response()
        }
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "User not found"})),
        )
            .into_response(),
    }
}

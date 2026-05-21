use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::jwt::JwtManager;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub role: String,
}

#[derive(Debug)]
pub struct AuthError(pub StatusCode, pub String);

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({"error": self.1});
        (self.0, axum::Json(body)).into_response()
    }
}

fn extract_auth_from_parts(parts: &mut Parts) -> Result<AuthUser, AuthError> {
    let auth_header = parts
        .headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AuthError(StatusCode::UNAUTHORIZED, "Missing authorization header".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AuthError(StatusCode::UNAUTHORIZED, "Use Bearer token".into()))?;

    let jwt = parts
        .extensions
        .get::<JwtManager>()
        .ok_or_else(|| AuthError(StatusCode::INTERNAL_SERVER_ERROR, "Auth not configured".into()))?;

    let claims = jwt
        .validate_access_token(token)
        .map_err(|e| AuthError(StatusCode::UNAUTHORIZED, e.to_string()))?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError(StatusCode::UNAUTHORIZED, "Invalid user id in token".into()))?;

    Ok(AuthUser {
        user_id,
        role: claims.role,
    })
}

impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        extract_auth_from_parts(parts)
    }
}

#[derive(Debug, Clone)]
pub struct OptionalAuth(pub Option<AuthUser>);

impl<S: Send + Sync> FromRequestParts<S> for OptionalAuth {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(OptionalAuth(extract_auth_from_parts(parts).ok()))
    }
}

#[derive(Debug, Clone)]
pub struct AdminUser {
    pub user_id: Uuid,
}

impl<S: Send + Sync> FromRequestParts<S> for AdminUser {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = AuthUser::from_request_parts(parts, state).await?;
        if auth.role != "admin" {
            return Err(AuthError(StatusCode::FORBIDDEN, "Admin access required".into()));
        }
        Ok(AdminUser { user_id: auth.user_id })
    }
}

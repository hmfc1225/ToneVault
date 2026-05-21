use axum::extract::{Request, State};
use axum::http::header::{AUTHORIZATION, WWW_AUTHENTICATE};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use base64::Engine;

use crate::AppState;

pub async fn basic_auth_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let credentials = match auth_header {
        Some(h) => h.strip_prefix("Basic ").and_then(|encoded| {
            base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
        }),
        None => None,
    };

    if let Some(creds) = credentials {
        let parts: Vec<&str> = creds.splitn(2, ':').collect();
        if parts.len() == 2 {
            let username = parts[0];
            let password = parts[1];

            if let Ok(Some(user)) = state.repo.get_user_by_username(username).await {
                if user.is_active && tonevault_auth::password::verify_password(password, &user.password_hash) {
                    return next.run(req).await;
                }
            }
        }
    }

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(WWW_AUTHENTICATE, "Basic realm=\"ToneVault WebDAV\"")
        .body(axum::body::Body::from("Unauthorized"))
        .unwrap()
        .into_response()
}

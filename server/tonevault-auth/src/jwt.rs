use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
    pub token_type: String,
}

#[derive(Clone)]
pub struct JwtManager {
    secret: String,
    access_ttl: Duration,
    refresh_ttl: Duration,
}

impl JwtManager {
    pub fn new(secret: String, access_hours: i64, refresh_days: i64) -> Self {
        Self {
            secret,
            access_ttl: Duration::hours(access_hours),
            refresh_ttl: Duration::days(refresh_days),
        }
    }

    pub fn create_access_token(&self, user_id: Uuid, role: &str) -> Result<String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.to_string(),
            role: role.to_string(),
            exp: (now + self.access_ttl).timestamp(),
            iat: now.timestamp(),
            token_type: "access".to_string(),
        };
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| anyhow!("Failed to encode access token: {}", e))
    }

    pub fn create_refresh_token(&self, user_id: Uuid) -> Result<String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.to_string(),
            role: String::new(),
            exp: (now + self.refresh_ttl).timestamp(),
            iat: now.timestamp(),
            token_type: "refresh".to_string(),
        };
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| anyhow!("Failed to encode refresh token: {}", e))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let data = jsonwebtoken::decode::<Claims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(self.secret.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|e| anyhow!("Invalid token: {}", e))?;
        Ok(data.claims)
    }

    pub fn validate_access_token(&self, token: &str) -> Result<Claims> {
        let claims = self.validate_token(token)?;
        if claims.token_type != "access" {
            return Err(anyhow!("Expected access token, got {}", claims.token_type));
        }
        Ok(claims)
    }

    pub fn validate_refresh_token(&self, token: &str) -> Result<Claims> {
        let claims = self.validate_token(token)?;
        if claims.token_type != "refresh" {
            return Err(anyhow!("Expected refresh token, got {}", claims.token_type));
        }
        Ok(claims)
    }
}

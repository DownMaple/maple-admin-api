use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role_id: String,
    pub role_code: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
    pub token_type: String,
}

impl Claims {
    pub fn new_access_token(user_id: Uuid, role_id: Uuid, role_code: String, expiration_hours: i64) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(expiration_hours);
        
        Self {
            sub: user_id.to_string(),
            role_id: role_id.to_string(),
            role_code,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            token_type: "access".to_string(),
        }
    }

    pub fn new_refresh_token(user_id: Uuid, role_id: Uuid, role_code: String, expiration_days: i64) -> Self {
        let now = Utc::now();
        let exp = now + Duration::days(expiration_days);
        
        Self {
            sub: user_id.to_string(),
            role_id: role_id.to_string(),
            role_code,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            token_type: "refresh".to_string(),
        }
    }
}

pub struct JwtService {
    secret: String,
    access_token_expiration_hours: i64,
    refresh_token_expiration_days: i64,
}

impl JwtService {
    pub fn new(secret: String, expiration_hours: i64) -> Self {
        Self {
            secret,
            access_token_expiration_hours: expiration_hours,
            refresh_token_expiration_days: 7,
        }
    }

    pub fn generate_access_token(&self, user_id: Uuid, role_id: Uuid, role_code: String) -> Result<String, AppError> {
        let claims = Claims::new_access_token(user_id, role_id, role_code, self.access_token_expiration_hours);
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )?;
        Ok(token)
    }

    pub fn generate_refresh_token(&self, user_id: Uuid, role_id: Uuid, role_code: String) -> Result<String, AppError> {
        let claims = Claims::new_refresh_token(user_id, role_id, role_code, self.refresh_token_expiration_days);
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )?;
        Ok(token)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }
}

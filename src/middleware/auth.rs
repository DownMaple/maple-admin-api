use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;

use crate::utils::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user_id
    pub exp: i64,     // 过期时间
    pub iat: i64,     // 签发时间
    pub nbf: i64,     // 生效时间
}

impl Claims {
    pub fn new(user_id: Uuid, expiration_hours: i64) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(expiration_hours);
        
        Self {
            sub: user_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
        }
    }
}

pub struct JwtService {
    secret: String,
    expiration_hours: i64,
}

impl JwtService {
    pub fn new(secret: String, expiration_hours: i64) -> Self {
        Self {
            secret,
            expiration_hours,
        }
    }

    pub fn generate_token(&self, user_id: Uuid) -> Result<String, AppError> {
        let claims = Claims::new(user_id, self.expiration_hours);
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

#[handler]
pub async fn auth_middleware(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    // 从请求头获取 token
    let token = match extract_token_from_header(req) {
        Some(token) => token,
        None => {
            res.render(Json(crate::utils::ErrorResponse::new(
                401,
                "未提供认证令牌".to_string(),
            )));
            res.status_code(StatusCode::UNAUTHORIZED);
            ctrl.skip_rest();
            return;
        }
    };

    // 获取 JWT 服务
    let jwt_service = depot.get::<Arc<JwtService>>("jwt_service").unwrap();
    
    // 验证 token
    match jwt_service.validate_token(&token) {
        Ok(claims) => {
            // 将用户信息存入 depot，供后续处理器使用
            depot.insert("user_id", claims.sub.clone());
            depot.insert("claims", claims);
        }
        Err(_) => {
            res.render(Json(crate::utils::ErrorResponse::new(
                401,
                "无效的认证令牌".to_string(),
            )));
            res.status_code(StatusCode::UNAUTHORIZED);
            ctrl.skip_rest();
        }
    }
}

fn extract_token_from_header(req: &Request) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| {
            if h.starts_with("Bearer ") {
                Some(h[7..].to_string())
            } else {
                None
            }
        })
}

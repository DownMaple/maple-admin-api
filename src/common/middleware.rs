use salvo::prelude::*;
use std::sync::Arc;
use sea_orm::DatabaseConnection;

use super::jwt::{JwtService, Claims};
use super::error::ErrorResponse;

// 依赖注入中间件
pub struct DepsMiddleware {
    db: Option<Arc<DatabaseConnection>>,
    jwt_service: Arc<JwtService>,
}

impl DepsMiddleware {
    pub fn new(db: Option<Arc<DatabaseConnection>>, jwt_service: Arc<JwtService>) -> Self {
        Self { db, jwt_service }
    }
}

#[async_trait]
impl Handler for DepsMiddleware {
    async fn handle(&self, _req: &mut Request, depot: &mut Depot, _res: &mut Response, ctrl: &mut FlowCtrl) {
        if let Some(db) = &self.db {
            depot.insert("db", db.clone());
        }
        depot.insert("jwt_service", self.jwt_service.clone());
        ctrl.call_next(_req, depot, _res).await;
    }
}

// JWT 认证中间件
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
            res.render(Json(ErrorResponse::new(
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
            depot.insert("user_id", claims.sub.clone());
            depot.insert("role_id", claims.role_id.clone());
            depot.insert("role_code", claims.role_code.clone());
            depot.insert("claims", claims);
        }
        Err(_) => {
            res.render(Json(ErrorResponse::new(
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

use salvo::prelude::*;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::Arc;
use uuid::Uuid;

use super::error::ErrorResponse;
use super::jwt::{Claims, JwtService};
use crate::models::{role, user, user_role};

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
    async fn handle(
        &self,
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        if let Some(db) = &self.db {
            depot.insert("db", db.clone());
        }
        depot.insert("jwt_service", self.jwt_service.clone());
        ctrl.call_next(req, depot, res).await;
    }
}

async fn is_active_session(db: &DatabaseConnection, claims: &Claims) -> bool {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(user_id) => user_id,
        Err(_) => return false,
    };
    let role_id = match Uuid::parse_str(&claims.role_id) {
        Ok(role_id) => role_id,
        Err(_) => return false,
    };

    let user_item = match user::Entity::find_by_id(user_id).one(db).await {
        Ok(Some(user_item)) => user_item,
        _ => return false,
    };
    if user_item.deleted_time.is_some() || user_item.status != 1 {
        return false;
    }

    let role_item = match role::Entity::find_by_id(role_id).one(db).await {
        Ok(Some(role_item)) => role_item,
        _ => return false,
    };
    if role_item.deleted_time.is_some() || role_item.status != 1 {
        return false;
    }

    matches!(
        user_role::Entity::find()
            .filter(user_role::Column::UserId.eq(user_id))
            .filter(user_role::Column::RoleId.eq(role_id))
            .one(db)
            .await,
        Ok(Some(_))
    )
}

#[handler]
pub async fn auth_middleware(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    let token = match extract_token_from_header(req) {
        Some(token) => token,
        None => {
            res.render(Json(ErrorResponse::new(401, "未提供认证令牌".to_string())));
            res.status_code(StatusCode::UNAUTHORIZED);
            ctrl.skip_rest();
            return;
        }
    };

    let jwt_service = depot.get::<Arc<JwtService>>("jwt_service").unwrap().clone();
    let db = match depot.get::<Arc<DatabaseConnection>>("db") {
        Ok(db) => db.clone(),
        Err(_) => {
            res.render(Json(ErrorResponse::new(
                500,
                "数据库服务不可用，请稍后重试".to_string(),
            )));
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            ctrl.skip_rest();
            return;
        }
    };

    match jwt_service.validate_token(&token) {
        Ok(claims) => {
            if !is_active_session(db.as_ref(), &claims).await {
                res.render(Json(ErrorResponse::new(
                    401,
                    "当前登录状态已失效，请重新登录".to_string(),
                )));
                res.status_code(StatusCode::UNAUTHORIZED);
                ctrl.skip_rest();
                return;
            }

            depot.insert("user_id", claims.sub.clone());
            depot.insert("role_id", claims.role_id.clone());
            depot.insert("role_code", claims.role_code.clone());
            depot.insert("claims", claims);
            ctrl.call_next(req, depot, res).await;
        }
        Err(_) => {
            res.render(Json(ErrorResponse::new(401, "无效的认证令牌".to_string())));
            res.status_code(StatusCode::UNAUTHORIZED);
            ctrl.skip_rest();
        }
    }
}

fn extract_token_from_header(req: &Request) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(ToString::to_string)
}

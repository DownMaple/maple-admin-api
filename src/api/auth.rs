use salvo::prelude::*;
use salvo::oapi::extract::JsonBody;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;

use crate::middleware::JwtService;
use crate::utils::{ApiResponse, AppError};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[handler]
pub async fn login(
    req: JsonBody<LoginRequest>,
    depot: &Depot,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    let login_data = req.into_inner();
    
    // TODO: 这里需要实现实际的用户验证逻辑
    // 1. 从数据库查询用户
    // 2. 验证密码
    // 3. 生成 token
    
    // 临时模拟登录逻辑
    if login_data.username == "admin" && login_data.password == "admin123" {
        let jwt_service = depot.get::<Arc<JwtService>>("jwt_service").unwrap();
        let user_id = Uuid::new_v4();
        let token = jwt_service.generate_token(user_id)?;
        
        let response = LoginResponse {
            token,
            user_id: user_id.to_string(),
            username: login_data.username,
        };
        
        Ok(Json(ApiResponse::success(response)))
    } else {
        Err(AppError::Unauthorized)
    }
}

#[handler]
pub async fn register(
    req: JsonBody<RegisterRequest>,
    _depot: &Depot,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let _register_data = req.into_inner();
    
    // TODO: 实现用户注册逻辑
    // 1. 验证用户名和邮箱是否已存在
    // 2. 对密码进行哈希处理
    // 3. 保存到数据库
    
    Ok(Json(ApiResponse::success_with_message(serde_json::json!({}), "注册成功".to_string())))
}

#[handler]
pub async fn logout() -> Json<ApiResponse<serde_json::Value>> {
    // 在实际应用中，可能需要将 token 加入黑名单
    Json(ApiResponse::success_with_message(serde_json::json!({}), "登出成功".to_string()))
}

#[handler]
pub async fn current_user(depot: &Depot) -> Json<ApiResponse<serde_json::Value>> {
    let user_id = depot.get::<String>("user_id").unwrap();
    
    // TODO: 从数据库获取用户详细信息
    let user_info = serde_json::json!({
        "id": user_id,
        "username": "admin",
        "email": "admin@example.com",
        "roles": ["admin"]
    });
    
    Json(ApiResponse::success(user_info))
}

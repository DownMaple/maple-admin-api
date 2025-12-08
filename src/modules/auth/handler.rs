use salvo::prelude::*;
use salvo::oapi::extract::JsonBody;
use salvo::http::cookie::{Cookie, SameSite};
use uuid::Uuid;
use std::sync::Arc;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};

use crate::common::{ApiResponse, AppError, jwt::JwtService, crypto, rsa_crypto};
use crate::models::{user, role, user_role};
use super::dto::{LoginRequest, LoginResponse, RegisterRequest, UserRole, SwitchRoleRequest, SwitchRoleResponse};

/// 用户登录
#[endpoint(
    tags("认证"),
    responses(
        (status_code = 200, description = "登录成功"),
        (status_code = 401, description = "用户名或密码错误"),
        (status_code = 500, description = "服务器错误")
    )
)]
pub async fn login(
    req: JsonBody<LoginRequest>,
    depot: &Depot,
    res: &mut Response,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    let login_data = req.into_inner();
    
    let db = match depot.get::<Arc<DatabaseConnection>>("db") {
        Ok(db) => db,
        Err(_) => {
            tracing::error!("数据库连接不可用，无法处理登录请求");
            return Err(AppError::InternalServerError("数据库服务不可用，请稍后重试".to_string()));
        }
    };
    
    let jwt_service = depot.get::<Arc<JwtService>>("jwt_service").unwrap();
    
    // 首先查找用户（不限制状态和删除状态）
    let find_user = user::Entity::find()
        .filter(user::Column::Username.eq(&login_data.username))
        .one(db.as_ref())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    
    // 检查用户是否存在
    let user = match find_user {
        Some(u) => u,
        None => {
            tracing::warn!("登录失败：用户名 '{}' 不存在", login_data.username);
            return Err(AppError::BadRequest("用户账号不存在".to_string()));
        }
    };
    
    // 检查账号是否已被删除
    if user.deleted_time.is_some() {
        tracing::warn!("登录失败：用户 '{}' 已被删除", login_data.username);
        return Err(AppError::BadRequest("该账号已被删除，无法登录".to_string()));
    }
    
    // 检查账号状态
    if user.status != 1 {
        tracing::warn!("登录失败：用户 '{}' 账号已被禁用，状态: {}", login_data.username, user.status);
        return Err(AppError::BadRequest("该账号已被禁用，请联系管理员".to_string()));
    }
    
    // 验证密码
    let plain_password = if login_data.is_encrypted {
        // 前端已RSA加密，先解密得到明文密码
        tracing::debug!("密码已加密，开始 RSA 解密...");
        match rsa_crypto::decrypt_password(&login_data.password) {
            Ok(pwd) => {
                tracing::debug!("✅ RSA 密码解密成功");
                pwd
            },
            Err(e) => {
                tracing::error!("❌ RSA 密码解密失败: {}", e);
                return Err(e);
            }
        }
    } else {
        // 明文密码（仅用于测试或特殊场景）
        tracing::warn!("⚠️  使用明文密码登录，请确认是否为测试环境");
        login_data.password.clone()
    };
    
    // 使用bcrypt验证密码
    let password_valid = crypto::verify_password(&plain_password, &user.password)?;
    
    if !password_valid {
        tracing::warn!("登录失败：用户 '{}' 密码错误", login_data.username);
        return Err(AppError::BadRequest("密码错误".to_string()));
    }
    
    let user_roles = user_role::Entity::find()
        .filter(user_role::Column::UserId.eq(user.id))
        .find_also_related(role::Entity)
        .all(db.as_ref())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    
    if user_roles.is_empty() {
        return Err(AppError::Forbidden("用户没有分配角色".to_string()));
    }
    
    let roles: Vec<UserRole> = user_roles
        .iter()
        .filter_map(|(_, role_opt)| {
            role_opt.as_ref().map(|r| UserRole {
                role_id: r.id.to_string(),
                role_code: r.code.clone(),
                role_name: r.name.clone(),
            })
        })
        .collect();
    
    let selected_role = if let Some(role_id_str) = &login_data.role_id {
        let role_id = Uuid::parse_str(role_id_str)
            .map_err(|_| AppError::BadRequest("无效的角色ID".to_string()))?;
        user_roles
            .iter()
            .find(|(ur, _)| ur.role_id == role_id)
            .and_then(|(_, r)| r.as_ref())
            .ok_or(AppError::Forbidden("用户没有该角色权限".to_string()))?
    } else {
        user_roles[0].1.as_ref().unwrap()
    };
    
    let access_token = jwt_service.generate_access_token(
        user.id,
        selected_role.id,
        selected_role.code.clone(),
    )?;
    
    let refresh_token = jwt_service.generate_refresh_token(
        user.id,
        selected_role.id,
        selected_role.code.clone(),
    )?;
    
    let mut cookie = Cookie::new("refresh_token", refresh_token.clone());
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    res.add_cookie(cookie);
    
    let response = LoginResponse {
        id: user.id.to_string(),
        username: user.username,
        real_name: user.real_name,
        roles,
        access_token,
        refresh_token,
    };
    
    Ok(Json(ApiResponse::success(response)))
}

/// 用户注册
#[endpoint(tags("认证"))]
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

/// 用户登出
#[endpoint(tags("认证"))]
pub async fn logout(res: &mut Response) -> Json<ApiResponse<serde_json::Value>> {
    let mut cookie = Cookie::new("refresh_token", "");
    cookie.set_path("/");
    cookie.set_http_only(true);
    res.add_cookie(cookie);
    
    Json(ApiResponse::success_with_message(serde_json::json!({}), "登出成功".to_string()))
}

/// 切换角色
#[endpoint(tags("认证"))]
pub async fn switch_role(
    req: JsonBody<SwitchRoleRequest>,
    depot: &Depot,
    res: &mut Response,
) -> Result<Json<ApiResponse<SwitchRoleResponse>>, AppError> {
    let switch_data = req.into_inner();
    
    let db = match depot.get::<Arc<DatabaseConnection>>("db") {
        Ok(db) => db,
        Err(_) => {
            tracing::error!("数据库连接不可用，无法处理角色切换请求");
            return Err(AppError::InternalServerError("数据库服务不可用，请稍后重试".to_string()));
        }
    };
    
    let jwt_service = depot.get::<Arc<JwtService>>("jwt_service").unwrap();
    
    let user_id_str = match depot.get::<String>("user_id") {
        Ok(id) => id,
        Err(_) => return Err(AppError::Unauthorized),
    };
    
    let user_id = Uuid::parse_str(user_id_str.as_str())
        .map_err(|_| AppError::Unauthorized)?;
    
    let role_id = Uuid::parse_str(&switch_data.role_id)
        .map_err(|_| AppError::BadRequest("无效的角色ID".to_string()))?;
    
    let user_role_with_role = user_role::Entity::find()
        .filter(user_role::Column::UserId.eq(user_id))
        .filter(user_role::Column::RoleId.eq(role_id))
        .find_also_related(role::Entity)
        .one(db.as_ref())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    
    let (_, role_opt) = user_role_with_role.ok_or(AppError::Forbidden("用户没有该角色权限".to_string()))?;
    let role = role_opt.ok_or(AppError::InternalServerError("角色不存在".to_string()))?;
    
    let access_token = jwt_service.generate_access_token(
        user_id,
        role.id,
        role.code.clone(),
    )?;
    
    let refresh_token = jwt_service.generate_refresh_token(
        user_id,
        role.id,
        role.code.clone(),
    )?;
    
    let mut cookie = Cookie::new("refresh_token", refresh_token.clone());
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    res.add_cookie(cookie);
    
    let response = SwitchRoleResponse {
        access_token,
        refresh_token,
        role: UserRole {
            role_id: role.id.to_string(),
            role_code: role.code,
            role_name: role.name,
        },
    };
    
    Ok(Json(ApiResponse::success(response)))
}

/// 获取当前用户信息
#[endpoint(tags("认证"))]
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

/// 获取RSA公钥
#[endpoint(
    tags("认证"),
    responses(
        (status_code = 200, description = "成功获取公钥")
    )
)]
pub async fn get_public_key() -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let public_key = rsa_crypto::get_public_key()?;
    let response = serde_json::json!({
        "public_key": public_key
    });
    Ok(Json(ApiResponse::success(response)))
}

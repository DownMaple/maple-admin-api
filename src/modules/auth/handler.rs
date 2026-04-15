use salvo::http::cookie::{Cookie, SameSite};
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::Arc;
use uuid::Uuid;

use super::dto::{
    LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse, RegisterRequest,
    SwitchRoleRequest, SwitchRoleResponse, UserInfoResponse, UserRole,
};
use crate::common::{crypto, jwt::JwtService, rsa_crypto, ApiResponse, AppError};
use crate::models::{role, user, user_role};

fn get_db(depot: &Depot) -> Result<Arc<DatabaseConnection>, AppError> {
    depot
        .get::<Arc<DatabaseConnection>>("db")
        .cloned()
        .map_err(|_| AppError::InternalServerError("数据库服务不可用，请稍后重试".to_string()))
}

fn get_jwt_service(depot: &Depot) -> Arc<JwtService> {
    depot.get::<Arc<JwtService>>("jwt_service").unwrap().clone()
}

fn parse_uuid(value: &str, message: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(value).map_err(|_| AppError::BadRequest(message.to_string()))
}

fn role_is_available(role_item: &role::Model) -> bool {
    role_item.deleted_time.is_none() && role_item.status == 1
}

fn to_user_role(role_item: &role::Model) -> UserRole {
    UserRole {
        role_id: role_item.id.to_string(),
        role_code: role_item.code.clone(),
        role_name: role_item.name.clone(),
    }
}

async fn ensure_active_user<C>(db: &C, user_id: Uuid) -> Result<user::Model, AppError>
where
    C: ConnectionTrait,
{
    let user_item = user::Entity::find_by_id(user_id)
        .one(db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if user_item.deleted_time.is_some() || user_item.status != 1 {
        return Err(AppError::Forbidden("当前用户已被禁用或删除".to_string()));
    }

    Ok(user_item)
}

async fn get_available_roles_for_user<C>(db: &C, user_id: Uuid) -> Result<Vec<role::Model>, AppError>
where
    C: ConnectionTrait,
{
    let roles: Vec<role::Model> = user_role::Entity::find()
        .filter(user_role::Column::UserId.eq(user_id))
        .find_also_related(role::Entity)
        .all(db)
        .await?
        .into_iter()
        .filter_map(|(_, role_opt)| role_opt)
        .filter(role_is_available)
        .collect();

    if roles.is_empty() {
        return Err(AppError::Forbidden("用户没有可用角色".to_string()));
    }

    Ok(roles)
}

async fn ensure_active_user_role<C>(
    db: &C,
    user_id: Uuid,
    role_id: Uuid,
) -> Result<role::Model, AppError>
where
    C: ConnectionTrait,
{
    ensure_active_user(db, user_id).await?;

    let user_role_with_role = user_role::Entity::find()
        .filter(user_role::Column::UserId.eq(user_id))
        .filter(user_role::Column::RoleId.eq(role_id))
        .find_also_related(role::Entity)
        .one(db)
        .await?;

    let (_, role_opt) =
        user_role_with_role.ok_or(AppError::Forbidden("用户没有该角色权限".to_string()))?;
    let role_item = role_opt.ok_or(AppError::Forbidden("当前角色不存在".to_string()))?;

    if !role_is_available(&role_item) {
        return Err(AppError::Forbidden("当前角色已被禁用或删除".to_string()));
    }

    Ok(role_item)
}

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
    let db = get_db(depot)?;
    let jwt_service = get_jwt_service(depot);

    let find_user = user::Entity::find()
        .filter(user::Column::Username.eq(&login_data.username))
        .one(db.as_ref())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let user_item = match find_user {
        Some(user_item) => user_item,
        None => {
            tracing::warn!("登录失败：用户名 '{}' 不存在", login_data.username);
            return Err(AppError::BadRequest("用户账号不存在".to_string()));
        }
    };

    if user_item.deleted_time.is_some() {
        tracing::warn!("登录失败：用户 '{}' 已被删除", login_data.username);
        return Err(AppError::BadRequest("该账户已被删除，无法登录".to_string()));
    }

    if user_item.status != 1 {
        tracing::warn!(
            "登录失败：用户 '{}' 已被禁用，状态 {}",
            login_data.username,
            user_item.status
        );
        return Err(AppError::BadRequest(
            "该账户已被禁用，请联系管理员".to_string(),
        ));
    }

    let plain_password = if login_data.is_encrypted {
        match rsa_crypto::decrypt_password(&login_data.password) {
            Ok(password) => password,
            Err(error) => {
                tracing::error!("RSA 密码解密失败: {}", error);
                return Err(error);
            }
        }
    } else {
        tracing::warn!("当前使用明文密码登录，仅建议在测试环境中使用");
        login_data.password.clone()
    };

    let password_valid = crypto::verify_password(&plain_password, &user_item.password)?;
    if !password_valid {
        tracing::warn!("登录失败：用户 '{}' 密码错误", login_data.username);
        return Err(AppError::BadRequest("密码错误".to_string()));
    }

    let available_roles = get_available_roles_for_user(db.as_ref(), user_item.id).await?;
    let selected_role = if let Some(role_id_str) = &login_data.role_id {
        let role_id = parse_uuid(role_id_str, "无效的角色ID")?;
        ensure_active_user_role(db.as_ref(), user_item.id, role_id).await?
    } else {
        available_roles[0].clone()
    };

    let access_token = jwt_service.generate_access_token(
        user_item.id,
        selected_role.id,
        selected_role.code.clone(),
    )?;
    let refresh_token_value = jwt_service.generate_refresh_token(
        user_item.id,
        selected_role.id,
        selected_role.code.clone(),
    )?;

    let mut cookie = Cookie::new("refresh_token", refresh_token_value.clone());
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    res.add_cookie(cookie);

    let response = LoginResponse {
        id: user_item.id.to_string(),
        username: user_item.username,
        real_name: user_item.real_name,
        roles: available_roles.iter().map(to_user_role).collect(),
        access_token,
        refresh_token: refresh_token_value,
    };

    Ok(Json(ApiResponse::success(response)))
}

#[endpoint(tags("认证"))]
pub async fn register(
    req: JsonBody<RegisterRequest>,
    _depot: &Depot,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let _register_data = req.into_inner();

    Ok(Json(ApiResponse::success_with_message(
        serde_json::json!({}),
        "注册成功".to_string(),
    )))
}

#[endpoint(tags("认证"))]
pub async fn logout(res: &mut Response) -> Json<ApiResponse<serde_json::Value>> {
    let mut cookie = Cookie::new("refresh_token", "");
    cookie.set_path("/");
    cookie.set_http_only(true);
    res.add_cookie(cookie);

    Json(ApiResponse::success_with_message(
        serde_json::json!({}),
        "登出成功".to_string(),
    ))
}

#[endpoint(tags("认证"))]
pub async fn refresh_token(
    req: JsonBody<RefreshTokenRequest>,
    depot: &Depot,
    req_raw: &Request,
    res: &mut Response,
) -> Result<Json<ApiResponse<RefreshTokenResponse>>, AppError> {
    let req_data = req.into_inner();
    let db = get_db(depot)?;
    let jwt_service = get_jwt_service(depot);

    let refresh_token_value = req_data
        .refresh_token
        .or_else(|| {
            req_raw
                .cookie("refresh_token")
                .map(|cookie| cookie.value().to_string())
        })
        .ok_or(AppError::Unauthorized)?;

    let claims = jwt_service.validate_token(&refresh_token_value)?;
    if claims.token_type != "refresh" {
        return Err(AppError::Unauthorized);
    }

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let role_id = Uuid::parse_str(&claims.role_id).map_err(|_| AppError::Unauthorized)?;
    let role_item = ensure_active_user_role(db.as_ref(), user_id, role_id)
        .await
        .map_err(|_| AppError::Unauthorized)?;

    let access_token =
        jwt_service.generate_access_token(user_id, role_item.id, role_item.code.clone())?;
    let new_refresh_token =
        jwt_service.generate_refresh_token(user_id, role_item.id, role_item.code.clone())?;

    let mut cookie = Cookie::new("refresh_token", new_refresh_token.clone());
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    res.add_cookie(cookie);

    Ok(Json(ApiResponse::success(RefreshTokenResponse {
        access_token,
        refresh_token: new_refresh_token,
    })))
}

#[endpoint(tags("认证"))]
pub async fn switch_role(
    req: JsonBody<SwitchRoleRequest>,
    depot: &Depot,
    res: &mut Response,
) -> Result<Json<ApiResponse<SwitchRoleResponse>>, AppError> {
    let switch_data = req.into_inner();
    let db = get_db(depot)?;
    let jwt_service = get_jwt_service(depot);

    let user_id_str = match depot.get::<String>("user_id") {
        Ok(user_id) => user_id,
        Err(_) => return Err(AppError::Unauthorized),
    };
    let user_id = Uuid::parse_str(user_id_str.as_str()).map_err(|_| AppError::Unauthorized)?;
    let role_id = parse_uuid(&switch_data.role_id, "无效的角色ID")?;
    let role_item = ensure_active_user_role(db.as_ref(), user_id, role_id).await?;

    let access_token =
        jwt_service.generate_access_token(user_id, role_item.id, role_item.code.clone())?;
    let refresh_token_value =
        jwt_service.generate_refresh_token(user_id, role_item.id, role_item.code.clone())?;

    let mut cookie = Cookie::new("refresh_token", refresh_token_value.clone());
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    res.add_cookie(cookie);

    let response = SwitchRoleResponse {
        access_token,
        refresh_token: refresh_token_value,
        role: to_user_role(&role_item),
    };

    Ok(Json(ApiResponse::success(response)))
}

#[endpoint(
    tags("认证"),
    responses(
        (status_code = 200, description = "获取成功"),
        (status_code = 401, description = "未授权"),
        (status_code = 500, description = "服务器错误")
    )
)]
pub async fn get_user_info(depot: &Depot) -> Result<Json<ApiResponse<UserInfoResponse>>, AppError> {
    let user_id_str = depot
        .get::<String>("user_id")
        .map_err(|_| AppError::Unauthorized)?;
    let user_id = Uuid::parse_str(user_id_str.as_str()).map_err(|_| AppError::Unauthorized)?;

    let current_role_id = depot
        .get::<String>("role_id")
        .map_err(|_| AppError::Unauthorized)?
        .clone();
    let current_role_code = depot
        .get::<String>("role_code")
        .map_err(|_| AppError::Unauthorized)?
        .clone();

    let db = get_db(depot)?;
    let user_item = ensure_active_user(db.as_ref(), user_id).await?;
    let roles = get_available_roles_for_user(db.as_ref(), user_id)
        .await?
        .iter()
        .map(to_user_role)
        .collect();

    let response = UserInfoResponse {
        id: user_item.id.to_string(),
        user_name: user_item.username,
        real_name: user_item.real_name,
        email: user_item.email,
        phone: user_item.phone,
        avatar: user_item.avatar,
        status: user_item.status,
        roles,
        current_role_id,
        current_role_code,
    };

    Ok(Json(ApiResponse::success(response)))
}

#[endpoint(
    tags("认证"),
    responses((status_code = 200, description = "成功获取公钥"))
)]
pub async fn get_public_key() -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    let public_key = rsa_crypto::get_public_key()?;
    let response = serde_json::json!({
        "public_key": public_key
    });

    Ok(Json(ApiResponse::success(response)))
}

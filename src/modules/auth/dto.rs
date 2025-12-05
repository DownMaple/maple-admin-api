use serde::{Deserialize, Serialize};
use salvo::oapi::ToSchema;

/// 登录请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[salvo(schema(example = json!({
    "username": "superAdmin",
    "password": "superAdmin",
    "role_id": "a0000000-0000-0000-0000-000000000001"
})))]
pub struct LoginRequest {
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 角色ID，可选，不传则使用第一个角色
    pub role_id: Option<String>,
}

/// 用户角色信息
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserRole {
    /// 角色ID
    pub role_id: String,
    /// 角色代码
    pub role_code: String,
    /// 角色名称
    pub role_name: String,
}

/// 登录响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    /// 用户ID
    pub id: String,
    /// 用户名
    pub username: String,
    /// 真实姓名
    pub real_name: String,
    /// 用户拥有的所有角色
    pub roles: Vec<UserRole>,
    /// 访问令牌，有效期24小时
    pub access_token: String,
    /// 刷新令牌，有效期7天
    pub refresh_token: String,
}

/// 切换角色请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[salvo(schema(example = json!({
    "role_id": "a0000000-0000-0000-0000-000000000001"
})))]
pub struct SwitchRoleRequest {
    /// 要切换到的角色ID
    pub role_id: String,
}

/// 切换角色响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SwitchRoleResponse {
    /// 新的访问令牌
    pub access_token: String,
    /// 新的刷新令牌
    pub refresh_token: String,
    /// 当前角色信息
    pub role: UserRole,
}

/// 注册请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[salvo(schema(example = json!({
    "username": "testuser",
    "password": "password123",
    "email": "test@example.com"
})))]
pub struct RegisterRequest {
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 邮箱
    pub email: String,
}

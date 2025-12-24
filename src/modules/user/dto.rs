use serde::{Deserialize, Serialize};
use salvo::oapi::ToSchema;

/// 用户列表查询参数
#[derive(Debug, Deserialize, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserListQuery {
    /// 用户名（模糊搜索）
    pub username: Option<String>,
    /// 昵称（模糊搜索）
    pub real_name: Option<String>,
    /// 性别
    pub gender: Option<i16>,
    /// 邮箱（模糊搜索）
    pub email: Option<String>,
    /// 手机号（模糊搜索）
    pub phone: Option<String>,
    /// 用户状态：1-启用，0-禁用
    pub status: Option<i16>,
    /// 当前页码，默认1
    #[serde(default = "default_page")]
    pub page: u64,
    /// 每页数量，默认20
    #[serde(default = "default_page_size")]
    pub page_size: u64,
}

fn default_page() -> u64 { 1 }
fn default_page_size() -> u64 { 20 }

/// 用户列表响应项
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserListItem {
    pub id: String,
    pub username: String,
    pub real_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub avatar: Option<String>,
    pub status: i16,
    pub created_time: String,
}

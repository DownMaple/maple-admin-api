use salvo::prelude::*;
use salvo::oapi::extract::QueryParam;
use std::sync::Arc;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QueryOrder, PaginatorTrait, QuerySelect};

use crate::common::{ApiResponse, AppError, response::PageResponse, constants::MAX_PAGE_SIZE};
use crate::models::user;
use super::dto::{UserListQuery, UserListItem};

/// 获取用户列表（分页）
#[endpoint(
    tags("用户管理"),
    parameters(
        ("username" = Option<String>, Query, description = "用户名（模糊搜索）"),
        ("realName" = Option<String>, Query, description = "昵称（模糊搜索）"),
        ("gender" = Option<i16>, Query, description = "性别"),
        ("email" = Option<String>, Query, description = "邮箱（模糊搜索）"),
        ("phone" = Option<String>, Query, description = "手机号（模糊搜索）"),
        ("status" = Option<i16>, Query, description = "用户状态：1-启用，0-禁用"),
        ("page" = Option<u64>, Query, description = "当前页码，默认1"),
        ("pageSize" = Option<u64>, Query, description = "每页数量，默认20，最大100"),
    ),
    responses(
        (status_code = 200, description = "获取成功"),
        (status_code = 500, description = "服务器错误")
    )
)]
pub async fn get_user_list(
    query: QueryParam<UserListQuery, true>,
    depot: &Depot,
) -> Result<Json<ApiResponse<PageResponse<UserListItem>>>, AppError> {
    // 1. 获取数据库连接
    let db = depot.get::<Arc<DatabaseConnection>>("db")
        .map_err(|_| AppError::InternalServerError("数据库服务不可用".to_string()))?;

    // 2. 解析查询参数，处理分页边界
    let params = query.into_inner();
    let page = if params.page < 1 { 1 } else { params.page };
    let page_size = params.page_size.min(MAX_PAGE_SIZE).max(1);

    // 3. 构建查询条件
    let mut query_builder = user::Entity::find()
        .filter(user::Column::DeletedTime.is_null()); // 排除已删除用户

    // 用户名模糊搜索
    if let Some(ref username) = params.username {
        if !username.is_empty() {
            query_builder = query_builder.filter(user::Column::Username.contains(username));
        }
    }

    // 昵称模糊搜索
    if let Some(ref real_name) = params.real_name {
        if !real_name.is_empty() {
            query_builder = query_builder.filter(user::Column::RealName.contains(real_name));
        }
    }

    // 邮箱模糊搜索
    if let Some(ref email) = params.email {
        if !email.is_empty() {
            query_builder = query_builder.filter(user::Column::Email.contains(email));
        }
    }

    // 手机号模糊搜索
    if let Some(ref phone) = params.phone {
        if !phone.is_empty() {
            query_builder = query_builder.filter(user::Column::Phone.contains(phone));
        }
    }

    // 状态精确匹配
    if let Some(status) = params.status {
        query_builder = query_builder.filter(user::Column::Status.eq(status));
    }

    // 4. 查询总数（用于分页）
    let total = query_builder.clone()
        .count(db.as_ref())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // 5. 分页查询数据，按创建时间倒序
    let users = query_builder
        .order_by_desc(user::Column::CreatedTime)
        .offset((page - 1) * page_size)
        .limit(page_size)
        .all(db.as_ref())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // 6. 转换为响应结构（隐藏敏感字段如密码）
    let items: Vec<UserListItem> = users.into_iter().map(|u| UserListItem {
        id: u.id.to_string(),
        username: u.username,
        real_name: u.real_name,
        email: u.email,
        phone: u.phone,
        avatar: u.avatar,
        status: u.status,
        created_time: u.created_time.format("%Y-%m-%d %H:%M:%S").to_string(),
    }).collect();

    // 7. 返回分页响应
    let page_response = PageResponse::new(items, total, page, page_size);
    Ok(Json(ApiResponse::success(page_response)))
}
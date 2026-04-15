use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::Utc;
use salvo::oapi::extract::{JsonBody, PathParam};
use salvo::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use uuid::Uuid;

use super::dto::{
    BatchDeleteUsersRequest, CreateUserRequest, UpdateUserRequest, UserDetailResponse,
    UserListItem, UserListQuery,
};
use crate::common::{constants::MAX_PAGE_SIZE, crypto, ApiResponse, AppError, PageResponse};
use crate::models::{role, user, user_role};

const SUPER_ADMIN_USER_ID: &str = "b0000000-0000-0000-0000-000000000001";

#[endpoint(tags("用户管理"))]
pub async fn get_user_list(
    req: &mut Request,
    depot: &Depot,
) -> Result<Json<ApiResponse<PageResponse<UserListItem>>>, AppError> {
    let db = get_db(depot)?;
    let params = req
        .parse_queries::<UserListQuery>()
        .map_err(|err| AppError::BadRequest(format!("请求参数错误: {}", err)))?;
    let current = params.current.max(1);
    let size = params.size.max(1).min(MAX_PAGE_SIZE);

    let mut query_builder = user::Entity::find();

    if let Some(user_name) = normalize_optional(params.user_name) {
        query_builder = query_builder.filter(user::Column::Username.contains(&user_name));
    }
    if let Some(nick_name) = normalize_optional(params.nick_name) {
        query_builder = query_builder.filter(user::Column::NickName.contains(&nick_name));
    }
    if let Some(user_phone) = normalize_optional(params.user_phone) {
        query_builder = query_builder.filter(user::Column::Phone.contains(&user_phone));
    }
    if let Some(user_email) = normalize_optional(params.user_email) {
        query_builder = query_builder.filter(user::Column::Email.contains(&user_email));
    }
    if let Some(status) = params.status.as_deref() {
        query_builder = query_builder.filter(user::Column::Status.eq(parse_status(status)?));
    }
    if let Some(gender) = params.user_gender.as_deref() {
        query_builder = query_builder.filter(user::Column::Gender.eq(parse_gender(gender)?));
    }

    let total = query_builder.clone().count(db.as_ref()).await?;
    let users = query_builder
        .order_by_desc(user::Column::CreatedTime)
        .offset((current - 1) * size)
        .limit(size)
        .all(db.as_ref())
        .await?;

    let user_ids: Vec<Uuid> = users.iter().map(|item| item.id).collect();
    let role_code_map = get_user_role_codes(db.as_ref(), &user_ids).await?;

    let records = users
        .iter()
        .map(|item| build_user_list_item(item, &role_code_map))
        .collect();

    Ok(Json(ApiResponse::success(PageResponse::new(
        records, total, current, size,
    ))))
}

#[endpoint(tags("用户管理"))]
pub async fn get_user(
    id: PathParam<String>,
    depot: &Depot,
) -> Result<Json<ApiResponse<UserDetailResponse>>, AppError> {
    let db = get_db(depot)?;
    let user_id = parse_uuid(id.into_inner().as_str(), "无效的用户ID")?;

    let item = user::Entity::find_by_id(user_id)
        .one(db.as_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

    let role_code_map = get_user_role_codes(db.as_ref(), &[user_id]).await?;
    let detail = build_user_detail(&item, &role_code_map);

    Ok(Json(ApiResponse::success(detail)))
}

#[endpoint(tags("用户管理"))]
pub async fn create_user(
    req: JsonBody<CreateUserRequest>,
    depot: &Depot,
) -> Result<Json<ApiResponse<UserDetailResponse>>, AppError> {
    let db = get_db(depot)?;
    let current_user_id = get_current_user_id(depot)?;
    let data = req.into_inner();

    let user_name = data.user_name.trim();
    if user_name.is_empty() {
        return Err(AppError::BadRequest("用户名不能为空".to_string()));
    }
    if data.password.trim().is_empty() {
        return Err(AppError::BadRequest("密码不能为空".to_string()));
    }

    ensure_username_unique(db.as_ref(), user_name, None).await?;
    let role_ids = resolve_role_ids_by_codes(db.as_ref(), &data.user_roles).await?;
    let hashed_password = crypto::hash_password(data.password.trim())?;
    let nick_name = normalize_optional(data.nick_name).unwrap_or_else(|| user_name.to_string());
    let now = Utc::now().naive_utc();

    let txn = db.begin().await?;
    let new_user = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set(user_name.to_string()),
        password: Set(hashed_password),
        real_name: Set(nick_name.clone()),
        gender: Set(parse_optional_gender(data.user_gender.as_deref())?),
        nick_name: Set(Some(nick_name)),
        email: Set(normalize_optional(data.user_email)),
        phone: Set(normalize_optional(data.user_phone)),
        avatar: Set(None),
        status: Set(parse_status(&data.status)?),
        created_time: Set(now),
        created_id: Set(Some(current_user_id)),
        updated_time: Set(now),
        updated_id: Set(Some(current_user_id)),
        deleted_time: Set(None),
        deleted_id: Set(None),
    };

    let created = new_user.insert(&txn).await?;
    insert_user_roles(&txn, created.id, current_user_id, &role_ids).await?;
    txn.commit().await?;

    let role_code_map = get_user_role_codes(db.as_ref(), &[created.id]).await?;
    Ok(Json(ApiResponse::success_with_message(
        build_user_detail(&created, &role_code_map),
        "创建用户成功".to_string(),
    )))
}

#[endpoint(tags("用户管理"))]
pub async fn update_user(
    id: PathParam<String>,
    req: JsonBody<UpdateUserRequest>,
    depot: &Depot,
) -> Result<Json<ApiResponse<UserDetailResponse>>, AppError> {
    let db = get_db(depot)?;
    let current_user_id = get_current_user_id(depot)?;
    let user_id = parse_uuid(id.into_inner().as_str(), "无效的用户ID")?;
    let data = req.into_inner();

    let existing = user::Entity::find_by_id(user_id)
        .one(db.as_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

    validate_can_update_user(existing.id)?;
    let mut current_username = existing.username.clone();
    let mut active_model: user::ActiveModel = existing.into();

    if let Some(user_name) = data.user_name.as_deref() {
        let trimmed = user_name.trim();
        if trimmed.is_empty() {
            return Err(AppError::BadRequest("用户名不能为空".to_string()));
        }
        ensure_username_unique(db.as_ref(), trimmed, Some(user_id)).await?;
        current_username = trimmed.to_string();
        active_model.username = Set(trimmed.to_string());
    }

    if let Some(password) = data.password.as_deref() {
        let trimmed = password.trim();
        if !trimmed.is_empty() {
            active_model.password = Set(crypto::hash_password(trimmed)?);
        }
    }

    if data.user_gender.is_some() {
        active_model.gender = Set(parse_optional_gender(data.user_gender.as_deref())?);
    }

    if data.nick_name.is_some() {
        let nick_name = normalize_optional(data.nick_name).unwrap_or(current_username.clone());
        active_model.real_name = Set(nick_name.clone());
        active_model.nick_name = Set(Some(nick_name));
    }
    if data.user_phone.is_some() {
        active_model.phone = Set(normalize_optional(data.user_phone));
    }
    if data.user_email.is_some() {
        active_model.email = Set(normalize_optional(data.user_email));
    }
    if let Some(status) = data.status.as_deref() {
        active_model.status = Set(parse_status(status)?);
    }

    active_model.updated_time = Set(Utc::now().naive_utc());
    active_model.updated_id = Set(Some(current_user_id));

    let role_ids = if let Some(role_codes) = data.user_roles {
        Some(resolve_role_ids_by_codes(db.as_ref(), &role_codes).await?)
    } else {
        None
    };

    let txn = db.begin().await?;
    let updated = active_model.update(&txn).await?;
    if let Some(role_ids) = role_ids {
        replace_user_roles(&txn, updated.id, current_user_id, &role_ids).await?;
    }
    txn.commit().await?;

    let role_code_map = get_user_role_codes(db.as_ref(), &[updated.id]).await?;
    Ok(Json(ApiResponse::success_with_message(
        build_user_detail(&updated, &role_code_map),
        "更新用户成功".to_string(),
    )))
}

#[endpoint(tags("用户管理"))]
pub async fn delete_user(
    id: PathParam<String>,
    depot: &Depot,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let db = get_db(depot)?;
    let current_user_id = get_current_user_id(depot)?;
    let user_id = parse_uuid(id.into_inner().as_str(), "无效的用户ID")?;
    validate_can_delete_user(current_user_id, user_id)?;

    user::Entity::delete_by_id(user_id)
        .exec(db.as_ref())
        .await?;
    Ok(Json(ApiResponse::success_with_message(
        (),
        "删除用户成功".to_string(),
    )))
}

#[endpoint(tags("用户管理"))]
pub async fn batch_delete_users(
    req: JsonBody<BatchDeleteUsersRequest>,
    depot: &Depot,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let db = get_db(depot)?;
    let current_user_id = get_current_user_id(depot)?;
    let data = req.into_inner();
    if data.ids.is_empty() {
        return Err(AppError::BadRequest("请选择要删除的用户".to_string()));
    }

    let mut user_ids = Vec::with_capacity(data.ids.len());
    for id in data.ids {
        let user_id = parse_uuid(&id, "无效的用户ID")?;
        validate_can_delete_user(current_user_id, user_id)?;
        user_ids.push(user_id);
    }

    user::Entity::delete_many()
        .filter(user::Column::Id.is_in(user_ids))
        .exec(db.as_ref())
        .await?;

    Ok(Json(ApiResponse::success_with_message(
        (),
        "批量删除用户成功".to_string(),
    )))
}

fn get_db(depot: &Depot) -> Result<Arc<DatabaseConnection>, AppError> {
    depot
        .get::<Arc<DatabaseConnection>>("db")
        .cloned()
        .map_err(|_| AppError::InternalServerError("数据库服务不可用".to_string()))
}

fn get_current_user_id(depot: &Depot) -> Result<Uuid, AppError> {
    let user_id = depot
        .get::<String>("user_id")
        .map_err(|_| AppError::Unauthorized)?;
    parse_uuid(user_id.as_str(), "当前用户信息无效")
}

fn parse_uuid(value: &str, message: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(value).map_err(|_| AppError::BadRequest(message.to_string()))
}

fn parse_status(value: &str) -> Result<i16, AppError> {
    match value.trim() {
        "1" => Ok(1),
        "2" => Ok(2),
        _ => Err(AppError::BadRequest("无效的状态值".to_string())),
    }
}

fn parse_gender(value: &str) -> Result<i16, AppError> {
    match value.trim() {
        "1" => Ok(1),
        "2" => Ok(2),
        _ => Err(AppError::BadRequest("无效的性别值".to_string())),
    }
}

fn parse_optional_gender(value: Option<&str>) -> Result<Option<i16>, AppError> {
    value
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(parse_gender)
        .transpose()
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn status_to_api(value: i16) -> String {
    if value == 1 {
        "1".to_string()
    } else {
        "2".to_string()
    }
}

fn gender_to_api(value: Option<i16>) -> Option<String> {
    value.and_then(|gender| match gender {
        1 => Some("1".to_string()),
        2 => Some("2".to_string()),
        _ => None,
    })
}

async fn ensure_username_unique<C>(
    db: &C,
    username: &str,
    exclude_user_id: Option<Uuid>,
) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    let existing = user::Entity::find()
        .filter(user::Column::Username.eq(username))
        .all(db)
        .await?;

    let duplicated = existing
        .into_iter()
        .any(|item| Some(item.id) != exclude_user_id);
    if duplicated {
        return Err(AppError::BadRequest("用户名已存在".to_string()));
    }

    Ok(())
}

async fn resolve_role_ids_by_codes<C>(db: &C, role_codes: &[String]) -> Result<Vec<Uuid>, AppError>
where
    C: ConnectionTrait,
{
    let role_codes: Vec<String> = role_codes
        .iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    if role_codes.is_empty() {
        return Err(AppError::BadRequest("请至少选择一个角色".to_string()));
    }

    let roles = role::Entity::find()
        .filter(role::Column::Code.is_in(role_codes.clone()))
        .all(db)
        .await?;

    if roles.len() != role_codes.len() {
        return Err(AppError::BadRequest("存在无效的角色编码".to_string()));
    }

    Ok(roles.into_iter().map(|item| item.id).collect())
}

async fn insert_user_roles<C>(
    db: &C,
    user_id: Uuid,
    current_user_id: Uuid,
    role_ids: &[Uuid],
) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    for role_id in role_ids {
        let relation = user_role::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            role_id: Set(*role_id),
            created_time: Set(Utc::now().naive_utc()),
            created_id: Set(Some(current_user_id)),
        };
        relation.insert(db).await?;
    }

    Ok(())
}

async fn replace_user_roles<C>(
    db: &C,
    user_id: Uuid,
    current_user_id: Uuid,
    role_ids: &[Uuid],
) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    user_role::Entity::delete_many()
        .filter(user_role::Column::UserId.eq(user_id))
        .exec(db)
        .await?;
    insert_user_roles(db, user_id, current_user_id, role_ids).await
}

async fn get_user_role_codes<C>(
    db: &C,
    user_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<String>>, AppError>
where
    C: ConnectionTrait,
{
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = user_role::Entity::find()
        .filter(user_role::Column::UserId.is_in(user_ids.to_vec()))
        .find_also_related(role::Entity)
        .all(db)
        .await?;

    let mut role_map: HashMap<Uuid, Vec<String>> = HashMap::new();
    for (user_role_item, role_item) in rows {
        if let Some(role_item) = role_item {
            role_map
                .entry(user_role_item.user_id)
                .or_default()
                .push(role_item.code);
        }
    }

    Ok(role_map)
}

fn build_user_list_item(
    item: &user::Model,
    role_code_map: &HashMap<Uuid, Vec<String>>,
) -> UserListItem {
    UserListItem {
        id: item.id.to_string(),
        user_name: item.username.clone(),
        user_gender: gender_to_api(item.gender),
        nick_name: item
            .nick_name
            .clone()
            .unwrap_or_else(|| item.real_name.clone()),
        user_phone: item.phone.clone().unwrap_or_default(),
        user_email: item.email.clone().unwrap_or_default(),
        user_roles: role_code_map.get(&item.id).cloned().unwrap_or_default(),
        status: status_to_api(item.status),
        create_time: item.created_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        update_time: item.updated_time.format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

fn build_user_detail(
    item: &user::Model,
    role_code_map: &HashMap<Uuid, Vec<String>>,
) -> UserDetailResponse {
    UserDetailResponse {
        id: item.id.to_string(),
        user_name: item.username.clone(),
        user_gender: gender_to_api(item.gender),
        nick_name: item
            .nick_name
            .clone()
            .unwrap_or_else(|| item.real_name.clone()),
        user_phone: item.phone.clone().unwrap_or_default(),
        user_email: item.email.clone().unwrap_or_default(),
        user_roles: role_code_map.get(&item.id).cloned().unwrap_or_default(),
        status: status_to_api(item.status),
        create_time: item.created_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        update_time: item.updated_time.format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

fn validate_can_delete_user(current_user_id: Uuid, target_user_id: Uuid) -> Result<(), AppError> {
    let super_admin_user_id = Uuid::parse_str(SUPER_ADMIN_USER_ID)
        .map_err(|_| AppError::InternalServerError("系统用户配置错误".to_string()))?;

    if target_user_id == super_admin_user_id {
        return Err(AppError::BadRequest("超级管理员不能删除".to_string()));
    }
    if target_user_id == current_user_id {
        return Err(AppError::BadRequest("不能删除当前登录用户".to_string()));
    }

    Ok(())
}

fn validate_can_update_user(target_user_id: Uuid) -> Result<(), AppError> {
    let super_admin_user_id = Uuid::parse_str(SUPER_ADMIN_USER_ID)
        .map_err(|_| AppError::InternalServerError("绯荤粺鐢ㄦ埛閰嶇疆閿欒".to_string()))?;

    if target_user_id == super_admin_user_id {
        return Err(AppError::BadRequest(
            "瓒呯骇绠＄悊鍛樼敤鎴蜂俊鎭笉鍏佽淇敼".to_string(),
        ));
    }

    Ok(())
}

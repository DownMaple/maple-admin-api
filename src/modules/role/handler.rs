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
    BatchDeleteRolesRequest, CreateRoleRequest, RoleListItem, RoleListQuery, RoleOption,
    RolePermissionIdsResponse, UpdateRolePermissionIdsRequest, UpdateRoleRequest,
};
use crate::common::{constants::MAX_PAGE_SIZE, ApiResponse, AppError, PageResponse};
use crate::models::{menu, role, role_menu};

const SUPER_ADMIN_ROLE_ID: &str = "a0000000-0000-0000-0000-000000000001";

#[endpoint(tags("角色管理"))]
pub async fn get_role_list(
    req: &mut Request,
    depot: &Depot,
) -> Result<Json<ApiResponse<PageResponse<RoleListItem>>>, AppError> {
    let db = get_db(depot)?;
    let params = req
        .parse_queries::<RoleListQuery>()
        .map_err(|err| AppError::BadRequest(format!("请求参数错误: {}", err)))?;
    let current = params.current.max(1);
    let size = params.size.max(1).min(MAX_PAGE_SIZE);

    let mut query_builder = role::Entity::find();

    if let Some(role_name) = normalize_optional(params.role_name) {
        query_builder = query_builder.filter(role::Column::Name.contains(&role_name));
    }
    if let Some(role_code) = normalize_optional(params.role_code) {
        query_builder = query_builder.filter(role::Column::Code.contains(&role_code));
    }
    if let Some(status) = params.status.as_deref() {
        query_builder = query_builder.filter(role::Column::Status.eq(parse_status(status)?));
    }

    let total = query_builder.clone().count(db.as_ref()).await?;
    let roles = query_builder
        .order_by_desc(role::Column::CreatedTime)
        .offset((current - 1) * size)
        .limit(size)
        .all(db.as_ref())
        .await?;

    let records = roles
        .into_iter()
        .map(|item| RoleListItem {
            id: item.id.to_string(),
            role_name: item.name,
            role_code: item.code,
            role_desc: item.description.unwrap_or_default(),
            status: status_to_api(item.status),
            create_time: item.created_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            update_time: item.updated_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        })
        .collect();

    Ok(Json(ApiResponse::success(PageResponse::new(
        records, total, current, size,
    ))))
}

#[endpoint(tags("角色管理"))]
pub async fn get_enabled_roles(
    depot: &Depot,
) -> Result<Json<ApiResponse<Vec<RoleOption>>>, AppError> {
    let db = get_db(depot)?;

    let roles = role::Entity::find()
        .filter(role::Column::Status.eq(1))
        .order_by_asc(role::Column::CreatedTime)
        .all(db.as_ref())
        .await?;

    let data = roles
        .into_iter()
        .map(|item| RoleOption {
            id: item.id.to_string(),
            role_name: item.name,
            role_code: item.code,
        })
        .collect();

    Ok(Json(ApiResponse::success(data)))
}

#[endpoint(tags("角色管理"))]
pub async fn create_role(
    req: JsonBody<CreateRoleRequest>,
    depot: &Depot,
) -> Result<Json<ApiResponse<RoleListItem>>, AppError> {
    let db = get_db(depot)?;
    let current_user_id = get_current_user_id(depot)?;
    let data = req.into_inner();

    let role_name = data.role_name.trim();
    let role_code = data.role_code.trim();
    if role_name.is_empty() || role_code.is_empty() {
        return Err(AppError::BadRequest("角色名称和编码不能为空".to_string()));
    }

    ensure_role_code_unique(db.as_ref(), role_code, None).await?;

    let now = Utc::now().naive_utc();
    let new_role = role::ActiveModel {
        id: Set(Uuid::new_v4()),
        code: Set(role_code.to_string()),
        name: Set(role_name.to_string()),
        description: Set(normalize_optional(data.role_desc)),
        is_system: Set(false),
        status: Set(parse_status(&data.status)?),
        created_time: Set(now),
        created_id: Set(Some(current_user_id)),
        updated_time: Set(now),
        updated_id: Set(Some(current_user_id)),
        deleted_time: Set(None),
        deleted_id: Set(None),
    };

    let created = new_role.insert(db.as_ref()).await?;

    Ok(Json(ApiResponse::success_with_message(
        RoleListItem {
            id: created.id.to_string(),
            role_name: created.name,
            role_code: created.code,
            role_desc: created.description.unwrap_or_default(),
            status: status_to_api(created.status),
            create_time: created.created_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            update_time: created.updated_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        },
        "创建角色成功".to_string(),
    )))
}

#[endpoint(tags("角色管理"))]
pub async fn update_role(
    id: PathParam<String>,
    req: JsonBody<UpdateRoleRequest>,
    depot: &Depot,
) -> Result<Json<ApiResponse<RoleListItem>>, AppError> {
    let db = get_db(depot)?;
    let current_user_id = get_current_user_id(depot)?;
    let role_id = parse_uuid(id.into_inner().as_str(), "无效的角色ID")?;
    let data = req.into_inner();

    let existing = role::Entity::find_by_id(role_id)
        .one(db.as_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("角色不存在".to_string()))?;

    validate_can_update_role(&existing)?;
    let mut active_model: role::ActiveModel = existing.clone().into();

    if let Some(role_name) = data.role_name.as_deref() {
        let trimmed = role_name.trim();
        if trimmed.is_empty() {
            return Err(AppError::BadRequest("角色名称不能为空".to_string()));
        }
        active_model.name = Set(trimmed.to_string());
    }

    if let Some(role_code) = data.role_code.as_deref() {
        let trimmed = role_code.trim();
        if trimmed.is_empty() {
            return Err(AppError::BadRequest("角色编码不能为空".to_string()));
        }
        if existing.is_system && trimmed != existing.code {
            return Err(AppError::BadRequest("系统角色编码不允许修改".to_string()));
        }
        ensure_role_code_unique(db.as_ref(), trimmed, Some(role_id)).await?;
        active_model.code = Set(trimmed.to_string());
    }

    if data.role_desc.is_some() {
        active_model.description = Set(normalize_optional(data.role_desc));
    }

    if let Some(status) = data.status.as_deref() {
        if existing.is_system && parse_status(status)? != existing.status {
            return Err(AppError::BadRequest("系统角色状态不允许修改".to_string()));
        }
        active_model.status = Set(parse_status(status)?);
    }

    active_model.updated_time = Set(Utc::now().naive_utc());
    active_model.updated_id = Set(Some(current_user_id));

    let updated = active_model.update(db.as_ref()).await?;

    Ok(Json(ApiResponse::success_with_message(
        RoleListItem {
            id: updated.id.to_string(),
            role_name: updated.name,
            role_code: updated.code,
            role_desc: updated.description.unwrap_or_default(),
            status: status_to_api(updated.status),
            create_time: updated.created_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            update_time: updated.updated_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        },
        "更新角色成功".to_string(),
    )))
}

#[endpoint(tags("角色管理"))]
pub async fn delete_role(
    id: PathParam<String>,
    depot: &Depot,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let db = get_db(depot)?;
    let role_id = parse_uuid(id.into_inner().as_str(), "无效的角色ID")?;
    let current_role_id = get_current_role_id(depot)?;
    validate_can_delete_role(current_role_id, role_id)?;

    let existing = role::Entity::find_by_id(role_id)
        .one(db.as_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("角色不存在".to_string()))?;
    if existing.is_system {
        return Err(AppError::BadRequest("系统角色不能删除".to_string()));
    }

    role::Entity::delete_by_id(role_id)
        .exec(db.as_ref())
        .await?;

    Ok(Json(ApiResponse::success_with_message(
        (),
        "删除角色成功".to_string(),
    )))
}

#[endpoint(tags("角色管理"))]
pub async fn batch_delete_roles(
    req: JsonBody<BatchDeleteRolesRequest>,
    depot: &Depot,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let db = get_db(depot)?;
    let current_role_id = get_current_role_id(depot)?;
    let data = req.into_inner();
    if data.ids.is_empty() {
        return Err(AppError::BadRequest("请选择要删除的角色".to_string()));
    }

    let role_ids: Vec<Uuid> = data
        .ids
        .iter()
        .map(|item| parse_uuid(item, "无效的角色ID"))
        .collect::<Result<Vec<_>, _>>()?;

    for role_id in &role_ids {
        validate_can_delete_role(current_role_id, *role_id)?;
    }

    let existing_roles = role::Entity::find()
        .filter(role::Column::Id.is_in(role_ids.clone()))
        .all(db.as_ref())
        .await?;

    if existing_roles.iter().any(|item| item.is_system) {
        return Err(AppError::BadRequest("系统角色不能删除".to_string()));
    }

    role::Entity::delete_many()
        .filter(role::Column::Id.is_in(role_ids))
        .exec(db.as_ref())
        .await?;

    Ok(Json(ApiResponse::success_with_message(
        (),
        "批量删除角色成功".to_string(),
    )))
}

#[endpoint(tags("角色管理"))]
pub async fn get_role_menu_ids(
    id: PathParam<String>,
    depot: &Depot,
) -> Result<Json<ApiResponse<RolePermissionIdsResponse>>, AppError> {
    let db = get_db(depot)?;
    let role_id = parse_uuid(id.into_inner().as_str(), "无效的角色ID")?;
    ensure_role_exists(db.as_ref(), role_id).await?;

    let ids = if is_super_admin_role_id(role_id)? {
        get_all_permission_ids_by_types(db.as_ref(), &["catalog", "menu"]).await?
    } else {
        get_role_permission_ids_by_types(db.as_ref(), role_id, &["catalog", "menu"]).await?
    };
    Ok(Json(ApiResponse::success(RolePermissionIdsResponse {
        ids,
    })))
}

#[endpoint(tags("角色管理"))]
pub async fn update_role_menu_ids(
    id: PathParam<String>,
    req: JsonBody<UpdateRolePermissionIdsRequest>,
    depot: &Depot,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let db = get_db(depot)?;
    let current_user_id = get_current_user_id(depot)?;
    let role_id = parse_uuid(id.into_inner().as_str(), "无效的角色ID")?;
    ensure_role_exists(db.as_ref(), role_id).await?;

    validate_can_update_role_permissions(role_id)?;
    replace_role_permission_ids_by_types(
        db.as_ref(),
        role_id,
        current_user_id,
        &req.into_inner().ids,
        &["catalog", "menu"],
    )
    .await?;

    Ok(Json(ApiResponse::success_with_message(
        (),
        "菜单权限更新成功".to_string(),
    )))
}

#[endpoint(tags("角色管理"))]
pub async fn get_role_button_ids(
    id: PathParam<String>,
    depot: &Depot,
) -> Result<Json<ApiResponse<RolePermissionIdsResponse>>, AppError> {
    let db = get_db(depot)?;
    let role_id = parse_uuid(id.into_inner().as_str(), "无效的角色ID")?;
    ensure_role_exists(db.as_ref(), role_id).await?;

    let ids = if is_super_admin_role_id(role_id)? {
        get_all_permission_ids_by_types(db.as_ref(), &["button"]).await?
    } else {
        get_role_permission_ids_by_types(db.as_ref(), role_id, &["button"]).await?
    };
    Ok(Json(ApiResponse::success(RolePermissionIdsResponse {
        ids,
    })))
}

#[endpoint(tags("角色管理"))]
pub async fn update_role_button_ids(
    id: PathParam<String>,
    req: JsonBody<UpdateRolePermissionIdsRequest>,
    depot: &Depot,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let db = get_db(depot)?;
    let current_user_id = get_current_user_id(depot)?;
    let role_id = parse_uuid(id.into_inner().as_str(), "无效的角色ID")?;
    ensure_role_exists(db.as_ref(), role_id).await?;

    validate_can_update_role_permissions(role_id)?;
    replace_role_permission_ids_by_types(
        db.as_ref(),
        role_id,
        current_user_id,
        &req.into_inner().ids,
        &["button"],
    )
    .await?;

    Ok(Json(ApiResponse::success_with_message(
        (),
        "按钮权限更新成功".to_string(),
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

fn get_current_role_id(depot: &Depot) -> Result<Uuid, AppError> {
    let role_id = depot
        .get::<String>("role_id")
        .map_err(|_| AppError::Unauthorized)?;
    parse_uuid(role_id.as_str(), "当前角色信息无效")
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

fn status_to_api(value: i16) -> String {
    if value == 1 {
        "1".to_string()
    } else {
        "2".to_string()
    }
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

async fn ensure_role_exists<C>(db: &C, role_id: Uuid) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    let exists = role::Entity::find_by_id(role_id).one(db).await?.is_some();
    if !exists {
        return Err(AppError::NotFound("角色不存在".to_string()));
    }
    Ok(())
}

async fn ensure_role_code_unique<C>(
    db: &C,
    role_code: &str,
    exclude_role_id: Option<Uuid>,
) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    let roles = role::Entity::find()
        .filter(role::Column::Code.eq(role_code))
        .all(db)
        .await?;

    let duplicated = roles
        .into_iter()
        .any(|item| Some(item.id) != exclude_role_id);
    if duplicated {
        return Err(AppError::BadRequest("角色编码已存在".to_string()));
    }

    Ok(())
}

fn validate_can_delete_role(current_role_id: Uuid, target_role_id: Uuid) -> Result<(), AppError> {
    let super_admin_role_id = Uuid::parse_str(SUPER_ADMIN_ROLE_ID)
        .map_err(|_| AppError::InternalServerError("系统角色配置错误".to_string()))?;

    if target_role_id == super_admin_role_id {
        return Err(AppError::BadRequest("超级管理员角色不能删除".to_string()));
    }
    if target_role_id == current_role_id {
        return Err(AppError::BadRequest("不能删除当前使用中的角色".to_string()));
    }

    Ok(())
}

fn validate_can_update_role(existing: &role::Model) -> Result<(), AppError> {
    if existing.id == super_admin_role_id()? {
        return Err(AppError::BadRequest(
            "瓒呯骇绠＄悊鍛樿鑹蹭俊鎭笉鍏佽淇敼".to_string(),
        ));
    }

    Ok(())
}

fn validate_can_update_role_permissions(role_id: Uuid) -> Result<(), AppError> {
    if is_super_admin_role_id(role_id)? {
        return Err(AppError::BadRequest(
            "瓒呯骇绠＄悊鍛樿鑹叉潈闄愪笉鍏佽淇敼".to_string(),
        ));
    }

    Ok(())
}

fn super_admin_role_id() -> Result<Uuid, AppError> {
    Uuid::parse_str(SUPER_ADMIN_ROLE_ID)
        .map_err(|_| AppError::InternalServerError("绯荤粺瑙掕壊閰嶇疆閿欒".to_string()))
}

fn is_super_admin_role_id(role_id: Uuid) -> Result<bool, AppError> {
    Ok(role_id == super_admin_role_id()?)
}

async fn get_role_permission_ids_by_types<C>(
    db: &C,
    role_id: Uuid,
    menu_types: &[&str],
) -> Result<Vec<String>, AppError>
where
    C: ConnectionTrait,
{
    let rows = role_menu::Entity::find()
        .filter(role_menu::Column::RoleId.eq(role_id))
        .find_also_related(menu::Entity)
        .all(db)
        .await?;

    Ok(rows
        .into_iter()
        .filter_map(|(_, menu_item)| menu_item)
        .filter(|menu_item| menu_types.contains(&menu_item.menu_type.as_str()))
        .map(|menu_item| menu_item.id.to_string())
        .collect())
}

async fn get_all_permission_ids_by_types<C>(
    db: &C,
    menu_types: &[&str],
) -> Result<Vec<String>, AppError>
where
    C: ConnectionTrait,
{
    Ok(menu::Entity::find()
        .all(db)
        .await?
        .into_iter()
        .filter(|item| menu_types.contains(&item.menu_type.as_str()))
        .map(|item| item.id.to_string())
        .collect())
}

async fn replace_role_permission_ids_by_types<C>(
    db: &C,
    role_id: Uuid,
    current_user_id: Uuid,
    ids: &[String],
    menu_types: &[&str],
) -> Result<(), AppError>
where
    C: ConnectionTrait + TransactionTrait,
{
    let menu_ids: Vec<Uuid> = ids
        .iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect::<HashSet<_>>()
        .into_iter()
        .map(|item| parse_uuid(&item, "无效的菜单ID"))
        .collect::<Result<Vec<_>, _>>()?;

    let existing_menus = menu::Entity::find()
        .filter(menu::Column::Id.is_in(menu_ids.clone()))
        .all(db)
        .await?;

    if existing_menus.len() != menu_ids.len()
        || existing_menus
            .iter()
            .any(|item| !menu_types.contains(&item.menu_type.as_str()))
    {
        return Err(AppError::BadRequest("存在无效的权限节点".to_string()));
    }

    let final_menu_ids = collect_menu_ids_with_ancestors(db, &menu_ids, menu_types).await?;

    let txn = db.begin().await?;
    let current_relations = role_menu::Entity::find()
        .filter(role_menu::Column::RoleId.eq(role_id))
        .find_also_related(menu::Entity)
        .all(&txn)
        .await?;

    let delete_ids: Vec<Uuid> = current_relations
        .into_iter()
        .filter_map(|(relation, menu_item)| {
            menu_item
                .filter(|item| menu_types.contains(&item.menu_type.as_str()))
                .map(|_| relation.id)
        })
        .collect();

    if !delete_ids.is_empty() {
        role_menu::Entity::delete_many()
            .filter(role_menu::Column::Id.is_in(delete_ids))
            .exec(&txn)
            .await?;
    }

    for menu_id in final_menu_ids {
        let relation = role_menu::ActiveModel {
            id: Set(Uuid::new_v4()),
            role_id: Set(role_id),
            menu_id: Set(menu_id),
            created_time: Set(Utc::now().naive_utc()),
            created_id: Set(Some(current_user_id)),
        };
        relation.insert(&txn).await?;
    }

    txn.commit().await?;
    Ok(())
}

async fn collect_menu_ids_with_ancestors<C>(
    db: &C,
    menu_ids: &[Uuid],
    menu_types: &[&str],
) -> Result<Vec<Uuid>, AppError>
where
    C: ConnectionTrait,
{
    if menu_ids.is_empty() || !menu_types.iter().any(|item| *item == "catalog" || *item == "menu")
    {
        return Ok(menu_ids.to_vec());
    }

    let parent_map: HashMap<Uuid, Option<Uuid>> = menu::Entity::find()
        .filter(menu::Column::DeletedTime.is_null())
        .all(db)
        .await?
        .into_iter()
        .map(|item| (item.id, item.parent_id))
        .collect();

    Ok(collect_menu_ids_with_ancestors_from_map(&parent_map, menu_ids))
}

fn collect_menu_ids_with_ancestors_from_map(
    parent_map: &HashMap<Uuid, Option<Uuid>>,
    menu_ids: &[Uuid],
) -> Vec<Uuid> {
    let mut collected = HashSet::new();
    let mut pending = menu_ids.to_vec();

    while let Some(menu_id) = pending.pop() {
        if !collected.insert(menu_id) {
            continue;
        }

        if let Some(Some(parent_id)) = parent_map.get(&menu_id) {
            pending.push(*parent_id);
        }
    }

    collected.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::collect_menu_ids_with_ancestors_from_map;
    use std::collections::{HashMap, HashSet};
    use uuid::Uuid;

    #[test]
    fn test_collect_menu_ids_with_ancestors_from_map() {
        let root = Uuid::new_v4();
        let child = Uuid::new_v4();
        let leaf = Uuid::new_v4();
        let mut parent_map = HashMap::new();
        parent_map.insert(root, None);
        parent_map.insert(child, Some(root));
        parent_map.insert(leaf, Some(child));

        let result = collect_menu_ids_with_ancestors_from_map(&parent_map, &[leaf]);
        let result_set: HashSet<Uuid> = result.into_iter().collect();

        assert_eq!(result_set.len(), 3);
        assert!(result_set.contains(&root));
        assert!(result_set.contains(&child));
        assert!(result_set.contains(&leaf));
    }
}

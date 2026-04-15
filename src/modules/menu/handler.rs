use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::Utc;
use salvo::oapi::extract::{JsonBody, PathParam};
use salvo::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Set,
};
use uuid::Uuid;

use super::dto::{
    BatchDeleteMenusRequest, ButtonOptionResponse, CreateMenuRequest, MenuResponse,
    MenuTreeResponse, UpdateMenuRequest,
};
use crate::common::{ApiResponse, AppError};
use crate::models::{menu, role_menu};

const SUPER_ADMIN_ROLE_ID: &str = "a0000000-0000-0000-0000-000000000001";

#[endpoint(tags("菜单管理"))]
pub async fn get_menu_tree(
    depot: &Depot,
) -> Result<Json<ApiResponse<Vec<MenuResponse>>>, AppError> {
    let db = get_db(depot)?;
    let menus = fetch_all_menus(db.as_ref()).await?;
    Ok(Json(ApiResponse::success(build_menu_tree(&menus, None))))
}

#[endpoint(tags("菜单管理"))]
pub async fn get_menu_list(
    depot: &Depot,
) -> Result<Json<ApiResponse<Vec<MenuResponse>>>, AppError> {
    let db = get_db(depot)?;
    let menus = fetch_all_menus(db.as_ref()).await?;
    Ok(Json(ApiResponse::success(
        menus.iter().map(model_to_response).collect(),
    )))
}

#[endpoint(tags("菜单管理"))]
pub async fn get_user_menus(
    depot: &Depot,
) -> Result<Json<ApiResponse<Vec<MenuTreeResponse>>>, AppError> {
    let role_id = get_current_role_id(depot)?;
    let db = get_db(depot)?;
    let menus = fetch_all_menus(db.as_ref()).await?;

    if is_super_admin_role_id(role_id)? {
        let route_menus: Vec<menu::Model> = menus
            .into_iter()
            .filter(is_route_menu)
            .collect();

        return Ok(Json(ApiResponse::success(build_menu_tree_for_route(
            &route_menus, None,
        ))));
    }

    let role_menus = role_menu::Entity::find()
        .filter(role_menu::Column::RoleId.eq(role_id))
        .find_also_related(menu::Entity)
        .all(db.as_ref())
        .await?;

    let assigned_ids: HashSet<Uuid> = role_menus
        .into_iter()
        .filter_map(|(_, menu_item)| menu_item)
        .filter(|item| item.deleted_time.is_none() && is_route_menu(item))
        .map(|item| item.id)
        .collect();

    Ok(Json(ApiResponse::success(build_assigned_menu_tree_for_route(
        &menus,
        &assigned_ids,
        None,
    ))))
}

#[endpoint(tags("菜单管理"))]
pub async fn get_user_permissions(
    depot: &Depot,
) -> Result<Json<ApiResponse<Vec<String>>>, AppError> {
    let role_id = get_current_role_id(depot)?;
    let db = get_db(depot)?;

    let permissions = if is_super_admin_role_id(role_id)? {
        fetch_all_menus(db.as_ref())
            .await?
            .into_iter()
            .filter(|item| item.status == 1)
            .filter_map(|item| item.permission)
            .collect()
    } else {
        let role_menus = role_menu::Entity::find()
            .filter(role_menu::Column::RoleId.eq(role_id))
            .find_also_related(menu::Entity)
            .all(db.as_ref())
            .await?;

        role_menus
            .into_iter()
            .filter_map(|(_, menu_item)| menu_item)
            .filter(|item| item.deleted_time.is_none() && item.status == 1)
            .filter_map(|item| item.permission)
            .collect()
    };

    Ok(Json(ApiResponse::success(permissions)))
}

#[endpoint(tags("菜单管理"))]
pub async fn get_button_options(
    depot: &Depot,
) -> Result<Json<ApiResponse<Vec<ButtonOptionResponse>>>, AppError> {
    let db = get_db(depot)?;
    let menus = fetch_all_menus(db.as_ref()).await?;
    let menu_map: HashMap<Uuid, &menu::Model> = menus.iter().map(|item| (item.id, item)).collect();

    let data = menus
        .iter()
        .filter(|item| item.menu_type == "button")
        .map(|item| ButtonOptionResponse {
            id: item.id.to_string(),
            label: build_button_label(item, &menu_map),
            code: item.permission.clone().unwrap_or_default(),
        })
        .collect();

    Ok(Json(ApiResponse::success(data)))
}

#[endpoint(tags("菜单管理"))]
pub async fn get_menu(
    id: PathParam<String>,
    depot: &Depot,
) -> Result<Json<ApiResponse<MenuResponse>>, AppError> {
    let db = get_db(depot)?;
    let menu_id = parse_uuid(id.into_inner().as_str(), "无效的菜单ID")?;

    let menu_item = menu::Entity::find_by_id(menu_id)
        .one(db.as_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("菜单不存在".to_string()))?;

    Ok(Json(ApiResponse::success(model_to_response(&menu_item))))
}

#[endpoint(tags("菜单管理"))]
pub async fn create_menu(
    req: JsonBody<CreateMenuRequest>,
    depot: &Depot,
) -> Result<Json<ApiResponse<MenuResponse>>, AppError> {
    let db = get_db(depot)?;
    let current_user_id = get_current_user_id(depot).ok();
    let data = req.into_inner();

    validate_menu_type(&data.menu_type)?;
    let parent_id = data
        .parent_id
        .as_deref()
        .map(|value| parse_uuid(value, "无效的父级菜单ID"))
        .transpose()?;
    validate_parent_assignment(db.as_ref(), parent_id, &data.menu_type, None).await?;
    let now = Utc::now().naive_utc();

    let new_menu = menu::ActiveModel {
        id: Set(Uuid::new_v4()),
        parent_id: Set(parent_id),
        name: Set(data.name),
        menu_type: Set(data.menu_type),
        path: Set(normalize_optional(data.path)),
        component: Set(normalize_optional(data.component)),
        icon: Set(normalize_optional(data.icon)),
        permission: Set(normalize_optional(data.permission)),
        sort: Set(data.sort),
        is_show: Set(data.is_show),
        is_cache: Set(data.is_cache),
        is_external: Set(data.is_external),
        status: Set(1),
        created_time: Set(now),
        created_id: Set(current_user_id),
        updated_time: Set(now),
        updated_id: Set(current_user_id),
        deleted_time: Set(None),
        deleted_id: Set(None),
    };

    let created = new_menu.insert(db.as_ref()).await?;

    Ok(Json(ApiResponse::success_with_message(
        model_to_response(&created),
        "创建菜单成功".to_string(),
    )))
}

#[endpoint(tags("菜单管理"))]
pub async fn update_menu(
    id: PathParam<String>,
    req: JsonBody<UpdateMenuRequest>,
    depot: &Depot,
) -> Result<Json<ApiResponse<MenuResponse>>, AppError> {
    let db = get_db(depot)?;
    let current_user_id = get_current_user_id(depot).ok();
    let menu_id = parse_uuid(id.into_inner().as_str(), "无效的菜单ID")?;
    let data = req.into_inner();

    let existing = menu::Entity::find_by_id(menu_id)
        .one(db.as_ref())
        .await?
        .ok_or_else(|| AppError::NotFound("菜单不存在".to_string()))?;

    let next_parent_id = match data.parent_id.as_deref() {
        Some(parent_id) if parent_id.trim().is_empty() => None,
        Some(parent_id) => Some(parse_uuid(parent_id, "无效的父级菜单ID")?),
        None => existing.parent_id,
    };
    let next_menu_type = data
        .menu_type
        .clone()
        .unwrap_or_else(|| existing.menu_type.clone());
    validate_parent_assignment(db.as_ref(), next_parent_id, &next_menu_type, Some(existing.id))
        .await?;

    let mut active_model: menu::ActiveModel = existing.into();

    if let Some(parent_id) = data.parent_id {
        let parent_id = if parent_id.trim().is_empty() {
            None
        } else {
            Some(parse_uuid(&parent_id, "无效的父级菜单ID")?)
        };
        active_model.parent_id = Set(parent_id);
    }
    if let Some(name) = data.name {
        active_model.name = Set(name);
    }
    if let Some(menu_type) = data.menu_type {
        validate_menu_type(&menu_type)?;
        active_model.menu_type = Set(menu_type);
    }
    if data.path.is_some() {
        active_model.path = Set(normalize_optional(data.path));
    }
    if data.component.is_some() {
        active_model.component = Set(normalize_optional(data.component));
    }
    if data.icon.is_some() {
        active_model.icon = Set(normalize_optional(data.icon));
    }
    if data.permission.is_some() {
        active_model.permission = Set(normalize_optional(data.permission));
    }
    if let Some(sort) = data.sort {
        active_model.sort = Set(sort);
    }
    if let Some(is_show) = data.is_show {
        active_model.is_show = Set(is_show);
    }
    if let Some(is_cache) = data.is_cache {
        active_model.is_cache = Set(is_cache);
    }
    if let Some(is_external) = data.is_external {
        active_model.is_external = Set(is_external);
    }
    if let Some(status) = data.status {
        active_model.status = Set(status);
    }

    active_model.updated_time = Set(Utc::now().naive_utc());
    active_model.updated_id = Set(current_user_id);

    let updated = active_model.update(db.as_ref()).await?;

    Ok(Json(ApiResponse::success_with_message(
        model_to_response(&updated),
        "更新菜单成功".to_string(),
    )))
}

#[endpoint(tags("菜单管理"))]
pub async fn delete_menu(
    id: PathParam<String>,
    depot: &Depot,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let db = get_db(depot)?;
    let menu_id = parse_uuid(id.into_inner().as_str(), "无效的菜单ID")?;

    let exists = menu::Entity::find_by_id(menu_id)
        .one(db.as_ref())
        .await?
        .is_some();
    if !exists {
        return Err(AppError::NotFound("菜单不存在".to_string()));
    }

    menu::Entity::delete_by_id(menu_id)
        .exec(db.as_ref())
        .await?;

    Ok(Json(ApiResponse::success_with_message(
        (),
        "删除菜单成功".to_string(),
    )))
}

#[endpoint(tags("菜单管理"))]
pub async fn batch_delete_menus(
    req: JsonBody<BatchDeleteMenusRequest>,
    depot: &Depot,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let db = get_db(depot)?;
    let ids = req.into_inner().ids;
    if ids.is_empty() {
        return Err(AppError::BadRequest("请选择要删除的菜单".to_string()));
    }

    let menu_ids = ids
        .iter()
        .map(|item| parse_uuid(item, "存在无效的菜单ID"))
        .collect::<Result<Vec<_>, _>>()?;

    menu::Entity::delete_many()
        .filter(menu::Column::Id.is_in(menu_ids))
        .exec(db.as_ref())
        .await?;

    Ok(Json(ApiResponse::success_with_message(
        (),
        "批量删除菜单成功".to_string(),
    )))
}

async fn fetch_all_menus(db: &DatabaseConnection) -> Result<Vec<menu::Model>, AppError> {
    Ok(menu::Entity::find()
        .filter(menu::Column::DeletedTime.is_null())
        .order_by_asc(menu::Column::Sort)
        .all(db)
        .await?)
}

fn get_db(depot: &Depot) -> Result<Arc<DatabaseConnection>, AppError> {
    depot
        .get::<Arc<DatabaseConnection>>("db")
        .cloned()
        .map_err(|_| AppError::InternalServerError("数据库服务不可用".to_string()))
}

fn get_current_role_id(depot: &Depot) -> Result<Uuid, AppError> {
    let role_id = depot
        .get::<String>("role_id")
        .map_err(|_| AppError::Unauthorized)?;
    parse_uuid(role_id.as_str(), "当前角色信息无效")
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

fn is_super_admin_role_id(role_id: Uuid) -> Result<bool, AppError> {
    let super_admin_role_id = Uuid::parse_str(SUPER_ADMIN_ROLE_ID)
        .map_err(|_| AppError::InternalServerError("绯荤粺瑙掕壊閰嶇疆閿欒".to_string()))?;

    Ok(role_id == super_admin_role_id)
}

fn validate_menu_type(value: &str) -> Result<(), AppError> {
    if ["catalog", "menu", "button"].contains(&value) {
        Ok(())
    } else {
        Err(AppError::BadRequest("无效的菜单类型".to_string()))
    }
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

async fn validate_parent_assignment<C>(
    db: &C,
    parent_id: Option<Uuid>,
    menu_type: &str,
    current_menu_id: Option<Uuid>,
) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    let Some(parent_id) = parent_id else {
        return Ok(());
    };

    if Some(parent_id) == current_menu_id {
        return Err(AppError::BadRequest("父级菜单不能选择自己".to_string()));
    }

    let menus = menu::Entity::find()
        .filter(menu::Column::DeletedTime.is_null())
        .all(db)
        .await?;
    let menu_map: HashMap<Uuid, &menu::Model> = menus.iter().map(|item| (item.id, item)).collect();
    let parent_item = menu_map
        .get(&parent_id)
        .copied()
        .ok_or_else(|| AppError::BadRequest("父级菜单不存在".to_string()))?;

    if parent_item.menu_type == "button" {
        return Err(AppError::BadRequest("按钮节点不能作为父级菜单".to_string()));
    }

    if menu_type == "catalog" && parent_item.menu_type != "catalog" {
        return Err(AppError::BadRequest("目录只能挂载到目录节点下".to_string()));
    }

    if menu_type == "button" && parent_item.menu_type != "menu" {
        return Err(AppError::BadRequest("按钮只能挂载到菜单节点下".to_string()));
    }

    if let Some(current_menu_id) = current_menu_id {
        let mut descendants = HashSet::new();
        let mut pending = vec![current_menu_id];

        while let Some(menu_id) = pending.pop() {
            for child in menus.iter().filter(|item| item.parent_id == Some(menu_id)) {
                if descendants.insert(child.id) {
                    pending.push(child.id);
                }
            }
        }

        if descendants.contains(&parent_id) {
            return Err(AppError::BadRequest(
                "父级菜单不能选择当前菜单的子节点".to_string(),
            ));
        }
    }

    Ok(())
}

fn model_to_response(item: &menu::Model) -> MenuResponse {
    MenuResponse {
        id: item.id.to_string(),
        parent_id: item.parent_id.map(|id| id.to_string()),
        name: item.name.clone(),
        menu_type: item.menu_type.clone(),
        path: item.path.clone(),
        component: item.component.clone(),
        icon: item.icon.clone(),
        permission: item.permission.clone(),
        sort: item.sort,
        is_show: item.is_show,
        is_cache: item.is_cache,
        is_external: item.is_external,
        status: item.status,
        children: None,
    }
}

fn model_to_route_response(item: &menu::Model, children: Vec<MenuTreeResponse>) -> MenuTreeResponse {
    MenuTreeResponse {
        id: item.id.to_string(),
        name: item.name.clone(),
        menu_type: item.menu_type.clone(),
        path: item.path.clone(),
        component: item.component.clone(),
        icon: item.icon.clone(),
        sort: item.sort,
        is_show: item.is_show,
        is_cache: item.is_cache,
        is_external: item.is_external,
        children: if children.is_empty() {
            None
        } else {
            Some(children)
        },
    }
}

fn build_menu_tree(menus: &[menu::Model], parent_id: Option<Uuid>) -> Vec<MenuResponse> {
    let mut result: Vec<MenuResponse> = menus
        .iter()
        .filter(|item| item.parent_id == parent_id)
        .map(|item| {
            let children = build_menu_tree(menus, Some(item.id));
            let mut response = model_to_response(item);
            if !children.is_empty() {
                response.children = Some(children);
            }
            response
        })
        .collect();

    result.sort_by_key(|item| item.sort);
    result
}

fn build_menu_tree_for_route(
    menus: &[menu::Model],
    parent_id: Option<Uuid>,
) -> Vec<MenuTreeResponse> {
    let mut children_map: HashMap<Option<Uuid>, Vec<&menu::Model>> = HashMap::new();
    for item in menus {
        children_map.entry(item.parent_id).or_default().push(item);
    }

    fn build_recursive(
        children_map: &HashMap<Option<Uuid>, Vec<&menu::Model>>,
        parent_id: Option<Uuid>,
    ) -> Vec<MenuTreeResponse> {
        let mut result: Vec<MenuTreeResponse> = children_map
            .get(&parent_id)
            .map(|children| {
                children
                    .iter()
                    .map(|item| {
                        let sub_children = build_recursive(children_map, Some(item.id));
                        model_to_route_response(item, sub_children)
                    })
                    .collect()
            })
            .unwrap_or_default();

        result.sort_by_key(|item| item.sort);
        result
    }

    build_recursive(&children_map, parent_id)
}

fn build_assigned_menu_tree_for_route(
    menus: &[menu::Model],
    assigned_ids: &HashSet<Uuid>,
    parent_id: Option<Uuid>,
) -> Vec<MenuTreeResponse> {
    let mut children_map: HashMap<Option<Uuid>, Vec<&menu::Model>> = HashMap::new();
    for item in menus {
        children_map.entry(item.parent_id).or_default().push(item);
    }

    fn build_recursive(
        children_map: &HashMap<Option<Uuid>, Vec<&menu::Model>>,
        assigned_ids: &HashSet<Uuid>,
        parent_id: Option<Uuid>,
    ) -> Vec<MenuTreeResponse> {
        let mut result: Vec<MenuTreeResponse> = children_map
            .get(&parent_id)
            .map(|children| {
                children
                    .iter()
                    .filter_map(|item| {
                        let sub_children =
                            build_recursive(children_map, assigned_ids, Some(item.id));
                        let matched = assigned_ids.contains(&item.id);
                        if !is_route_menu(item) || (!matched && sub_children.is_empty()) {
                            return None;
                        }

                        Some(model_to_route_response(item, sub_children))
                    })
                    .collect()
            })
            .unwrap_or_default();

        result.sort_by_key(|item| item.sort);
        result
    }

    build_recursive(&children_map, assigned_ids, parent_id)
}

fn is_route_menu(item: &menu::Model) -> bool {
    item.status == 1 && item.menu_type != "button"
}

fn build_button_label(item: &menu::Model, menu_map: &HashMap<Uuid, &menu::Model>) -> String {
    let mut labels = vec![item.name.clone()];
    let mut current_parent_id = item.parent_id;

    while let Some(parent_id) = current_parent_id {
        if let Some(parent) = menu_map.get(&parent_id) {
            labels.push(parent.name.clone());
            current_parent_id = parent.parent_id;
        } else {
            break;
        }
    }

    labels.reverse();
    labels.join(" / ")
}

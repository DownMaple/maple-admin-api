use chrono::Utc;
use salvo::oapi::extract::{JsonBody, PathParam};
use salvo::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::dto::{CreateMenuRequest, MenuResponse, MenuTreeResponse, UpdateMenuRequest};
use crate::common::{ApiResponse, AppError};
use crate::models::{menu, role_menu};

/// 获取菜单列表（树形结构）
#[endpoint(
    tags("菜单管理"),
    responses(
        (status_code = 200, description = "获取成功"),
        (status_code = 500, description = "服务器错误")
    )
)]
pub async fn get_menu_tree(
    depot: &Depot,
) -> Result<Json<ApiResponse<Vec<MenuResponse>>>, AppError> {
    let db = depot
        .get::<Arc<DatabaseConnection>>("db")
        .map_err(|_| AppError::InternalServerError("数据库服务不可用".to_string()))?;

    let menus = menu::Entity::find()
        .filter(menu::Column::DeletedTime.is_null())
        .order_by_asc(menu::Column::Sort)
        .all(db.as_ref())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let tree = build_menu_tree(&menus, None);
    Ok(Json(ApiResponse::success(tree)))
}

/// 获取菜单列表（列表结构）
#[endpoint(
    tags("全部菜单列表"),
    responses(
        (status_code = 200, description = "获取成功"),
        (status_code = 500, description = "服务器错误")
    )
)]
pub async fn get_menu_list(
    depot: &Depot,
) -> Result<Json<ApiResponse<Vec<MenuResponse>>>, AppError> {
    let db = depot
        .get::<Arc<DatabaseConnection>>("db")
        .map_err(|_| AppError::InternalServerError("数据库服务不可用".to_string()))?;

    let menus = menu::Entity::find()
        .filter(menu::Column::DeletedTime.is_null())
        .order_by_asc(menu::Column::Sort)
        .all(db.as_ref())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(Json(ApiResponse::success(
        menus.into_iter().map(|m| model_to_response(&m)).collect(),
    )))
}

/// 获取当前用户的菜单（根据角色权限）
#[endpoint(
    tags("菜单管理"),
    responses(
        (status_code = 200, description = "获取成功"),
        (status_code = 401, description = "未授权"),
        (status_code = 500, description = "服务器错误")
    )
)]
pub async fn get_user_menus(
    depot: &Depot,
) -> Result<Json<ApiResponse<Vec<MenuTreeResponse>>>, AppError> {
    let role_id_str = depot
        .get::<String>("role_id")
        .map_err(|_| AppError::Unauthorized)?;

    let role_id = Uuid::parse_str(role_id_str.as_str()).map_err(|_| AppError::Unauthorized)?;

    let db = depot
        .get::<Arc<DatabaseConnection>>("db")
        .map_err(|_| AppError::InternalServerError("数据库服务不可用".to_string()))?;

    // 查询角色关联的菜单
    let role_menus = role_menu::Entity::find()
        .filter(role_menu::Column::RoleId.eq(role_id))
        .find_also_related(menu::Entity)
        .all(db.as_ref())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // 过滤出有效的菜单（已启用、显示）
    let menus: Vec<menu::Model> = role_menus
        .into_iter()
        .filter_map(|(_, m)| m)
        .filter(|m| m.deleted_time.is_none() && m.status == 1 && m.is_show)
        .collect();

    let tree = build_menu_tree_for_route(&menus, None);
    Ok(Json(ApiResponse::success(tree)))
}

/// 获取当前用户的权限列表
#[endpoint(
    tags("菜单管理"),
    responses(
        (status_code = 200, description = "获取成功"),
        (status_code = 401, description = "未授权"),
        (status_code = 500, description = "服务器错误")
    )
)]
pub async fn get_user_permissions(
    depot: &Depot,
) -> Result<Json<ApiResponse<Vec<String>>>, AppError> {
    let role_id_str = depot
        .get::<String>("role_id")
        .map_err(|_| AppError::Unauthorized)?;

    let role_id = Uuid::parse_str(role_id_str.as_str()).map_err(|_| AppError::Unauthorized)?;

    let db = depot
        .get::<Arc<DatabaseConnection>>("db")
        .map_err(|_| AppError::InternalServerError("数据库服务不可用".to_string()))?;

    let role_menus = role_menu::Entity::find()
        .filter(role_menu::Column::RoleId.eq(role_id))
        .find_also_related(menu::Entity)
        .all(db.as_ref())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let permissions: Vec<String> = role_menus
        .into_iter()
        .filter_map(|(_, m)| m)
        .filter(|m| m.deleted_time.is_none() && m.status == 1)
        .filter_map(|m| m.permission)
        .collect();

    Ok(Json(ApiResponse::success(permissions)))
}

/// 获取单个菜单详情
#[endpoint(
    tags("菜单管理"),
    responses(
        (status_code = 200, description = "获取成功"),
        (status_code = 404, description = "菜单不存在"),
        (status_code = 500, description = "服务器错误")
    )
)]
pub async fn get_menu(
    id: PathParam<String>,
    depot: &Depot,
) -> Result<Json<ApiResponse<MenuResponse>>, AppError> {
    let menu_id = Uuid::parse_str(&id.into_inner())
        .map_err(|_| AppError::BadRequest("无效的菜单ID".to_string()))?;

    let db = depot
        .get::<Arc<DatabaseConnection>>("db")
        .map_err(|_| AppError::InternalServerError("数据库服务不可用".to_string()))?;

    let menu = menu::Entity::find_by_id(menu_id)
        .filter(menu::Column::DeletedTime.is_null())
        .one(db.as_ref())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or(AppError::NotFound("菜单不存在".to_string()))?;

    Ok(Json(ApiResponse::success(model_to_response(&menu))))
}

/// 创建菜单
#[endpoint(
    tags("菜单管理"),
    responses(
        (status_code = 200, description = "创建成功"),
        (status_code = 400, description = "参数错误"),
        (status_code = 500, description = "服务器错误")
    )
)]
pub async fn create_menu(
    req: JsonBody<CreateMenuRequest>,
    depot: &Depot,
) -> Result<Json<ApiResponse<MenuResponse>>, AppError> {
    let data = req.into_inner();

    // 验证菜单类型
    if !["catalog", "menu", "button"].contains(&data.menu_type.as_str()) {
        return Err(AppError::BadRequest("无效的菜单类型".to_string()));
    }

    let db = depot
        .get::<Arc<DatabaseConnection>>("db")
        .map_err(|_| AppError::InternalServerError("数据库服务不可用".to_string()))?;

    let user_id = depot
        .get::<String>("user_id")
        .ok()
        .and_then(|s| Uuid::parse_str(s.as_str()).ok());

    let parent_id = data
        .parent_id
        .as_ref()
        .map(|s| Uuid::parse_str(s))
        .transpose()
        .map_err(|_| AppError::BadRequest("无效的父菜单ID".to_string()))?;

    let now = Utc::now().naive_utc();
    let new_menu = menu::ActiveModel {
        id: Set(Uuid::new_v4()),
        parent_id: Set(parent_id),
        name: Set(data.name),
        menu_type: Set(data.menu_type),
        path: Set(data.path),
        component: Set(data.component),
        icon: Set(data.icon),
        permission: Set(data.permission),
        sort: Set(data.sort),
        is_show: Set(data.is_show),
        is_cache: Set(data.is_cache),
        is_external: Set(data.is_external),
        status: Set(1),
        created_time: Set(now),
        created_id: Set(user_id),
        updated_time: Set(now),
        updated_id: Set(user_id),
        deleted_time: Set(None),
        deleted_id: Set(None),
    };

    let menu = new_menu
        .insert(db.as_ref())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(Json(ApiResponse::success_with_message(
        model_to_response(&menu),
        "创建成功".to_string(),
    )))
}

/// 更新菜单
#[endpoint(
    tags("菜单管理"),
    responses(
        (status_code = 200, description = "更新成功"),
        (status_code = 404, description = "菜单不存在"),
        (status_code = 500, description = "服务器错误")
    )
)]
pub async fn update_menu(
    id: PathParam<String>,
    req: JsonBody<UpdateMenuRequest>,
    depot: &Depot,
) -> Result<Json<ApiResponse<MenuResponse>>, AppError> {
    let menu_id = Uuid::parse_str(&id.into_inner())
        .map_err(|_| AppError::BadRequest("无效的菜单ID".to_string()))?;

    let data = req.into_inner();

    let db = depot
        .get::<Arc<DatabaseConnection>>("db")
        .map_err(|_| AppError::InternalServerError("数据库服务不可用".to_string()))?;

    let user_id = depot
        .get::<String>("user_id")
        .ok()
        .and_then(|s| Uuid::parse_str(s.as_str()).ok());

    let existing = menu::Entity::find_by_id(menu_id)
        .filter(menu::Column::DeletedTime.is_null())
        .one(db.as_ref())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or(AppError::NotFound("菜单不存在".to_string()))?;

    let mut active_model: menu::ActiveModel = existing.into();

    if let Some(parent_id) = data.parent_id {
        let pid = if parent_id.is_empty() {
            None
        } else {
            Some(
                Uuid::parse_str(&parent_id)
                    .map_err(|_| AppError::BadRequest("无效的父菜单ID".to_string()))?,
            )
        };
        active_model.parent_id = Set(pid);
    }
    if let Some(name) = data.name {
        active_model.name = Set(name);
    }
    if let Some(menu_type) = data.menu_type {
        if !["catalog", "menu", "button"].contains(&menu_type.as_str()) {
            return Err(AppError::BadRequest("无效的菜单类型".to_string()));
        }
        active_model.menu_type = Set(menu_type);
    }
    if data.path.is_some() {
        active_model.path = Set(data.path);
    }
    if data.component.is_some() {
        active_model.component = Set(data.component);
    }
    if data.icon.is_some() {
        active_model.icon = Set(data.icon);
    }
    if data.permission.is_some() {
        active_model.permission = Set(data.permission);
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
    active_model.updated_id = Set(user_id);

    let updated = active_model
        .update(db.as_ref())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(Json(ApiResponse::success_with_message(
        model_to_response(&updated),
        "更新成功".to_string(),
    )))
}

/// 删除菜单（软删除）
#[endpoint(
    tags("菜单管理"),
    responses(
        (status_code = 200, description = "删除成功"),
        (status_code = 404, description = "菜单不存在"),
        (status_code = 500, description = "服务器错误")
    )
)]
pub async fn delete_menu(
    id: PathParam<String>,
    depot: &Depot,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let menu_id = Uuid::parse_str(&id.into_inner())
        .map_err(|_| AppError::BadRequest("无效的菜单ID".to_string()))?;

    let db = depot
        .get::<Arc<DatabaseConnection>>("db")
        .map_err(|_| AppError::InternalServerError("数据库服务不可用".to_string()))?;

    let user_id = depot
        .get::<String>("user_id")
        .ok()
        .and_then(|s| Uuid::parse_str(s.as_str()).ok());

    let existing = menu::Entity::find_by_id(menu_id)
        .filter(menu::Column::DeletedTime.is_null())
        .one(db.as_ref())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or(AppError::NotFound("菜单不存在".to_string()))?;

    let mut active_model: menu::ActiveModel = existing.into();
    active_model.deleted_time = Set(Some(Utc::now().naive_utc()));
    active_model.deleted_id = Set(user_id);

    active_model
        .update(db.as_ref())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(Json(ApiResponse::success_with_message(
        (),
        "删除成功".to_string(),
    )))
}

// ========== 辅助函数 ==========

fn model_to_response(m: &menu::Model) -> MenuResponse {
    MenuResponse {
        id: m.id.to_string(),
        parent_id: m.parent_id.map(|id| id.to_string()),
        name: m.name.clone(),
        menu_type: m.menu_type.clone(),
        path: m.path.clone(),
        component: m.component.clone(),
        icon: m.icon.clone(),
        permission: m.permission.clone(),
        sort: m.sort,
        is_show: m.is_show,
        is_cache: m.is_cache,
        is_external: m.is_external,
        status: m.status,
        children: None,
    }
}

fn build_menu_tree(menus: &[menu::Model], parent_id: Option<Uuid>) -> Vec<MenuResponse> {
    let mut result: Vec<MenuResponse> = menus
        .iter()
        .filter(|m| m.parent_id == parent_id)
        .map(|m| {
            let children = build_menu_tree(menus, Some(m.id));
            let mut response = model_to_response(m);
            if !children.is_empty() {
                response.children = Some(children);
            }
            response
        })
        .collect();

    result.sort_by_key(|m| m.sort);
    result
}

fn build_menu_tree_for_route(
    menus: &[menu::Model],
    parent_id: Option<Uuid>,
) -> Vec<MenuTreeResponse> {
    // 构建父子关系映射
    let mut children_map: HashMap<Option<Uuid>, Vec<&menu::Model>> = HashMap::new();
    for m in menus {
        children_map.entry(m.parent_id).or_default().push(m);
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
                    .map(|m| {
                        let sub_children = build_recursive(children_map, Some(m.id));
                        MenuTreeResponse {
                            id: m.id.to_string(),
                            name: m.name.clone(),
                            menu_type: m.menu_type.clone(),
                            path: m.path.clone(),
                            component: m.component.clone(),
                            icon: m.icon.clone(),
                            sort: m.sort,
                            children: if sub_children.is_empty() {
                                None
                            } else {
                                Some(sub_children)
                            },
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        result.sort_by_key(|m| m.sort);
        result
    }

    build_recursive(&children_map, parent_id)
}

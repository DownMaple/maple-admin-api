use serde::{Deserialize, Serialize};
use salvo::oapi::ToSchema;

/// 菜单类型
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MenuType {
    /// 目录
    Catalog,
    /// 菜单
    Menu,
    /// 按钮
    Button,
}

impl MenuType {
    pub fn as_str(&self) -> &str {
        match self {
            MenuType::Catalog => "catalog",
            MenuType::Menu => "menu",
            MenuType::Button => "button",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "catalog" => Some(MenuType::Catalog),
            "menu" => Some(MenuType::Menu),
            "button" => Some(MenuType::Button),
            _ => None,
        }
    }
}

/// 创建菜单请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateMenuRequest {
    /// 父菜单ID
    pub parent_id: Option<String>,
    /// 菜单名称
    pub name: String,
    /// 菜单类型
    pub menu_type: String,
    /// 路由路径
    pub path: Option<String>,
    /// 组件路径
    pub component: Option<String>,
    /// 图标
    pub icon: Option<String>,
    /// 权限标识
    pub permission: Option<String>,
    /// 排序
    #[serde(default)]
    pub sort: i32,
    /// 是否显示
    #[serde(default = "default_true")]
    pub is_show: bool,
    /// 是否缓存
    #[serde(default)]
    pub is_cache: bool,
    /// 是否外链
    #[serde(default)]
    pub is_external: bool,
}

/// 更新菜单请求
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMenuRequest {
    /// 父菜单ID
    pub parent_id: Option<String>,
    /// 菜单名称
    pub name: Option<String>,
    /// 菜单类型
    pub menu_type: Option<String>,
    /// 路由路径
    pub path: Option<String>,
    /// 组件路径
    pub component: Option<String>,
    /// 图标
    pub icon: Option<String>,
    /// 权限标识
    pub permission: Option<String>,
    /// 排序
    pub sort: Option<i32>,
    /// 是否显示
    pub is_show: Option<bool>,
    /// 是否缓存
    pub is_cache: Option<bool>,
    /// 是否外链
    pub is_external: Option<bool>,
    /// 状态
    pub status: Option<i16>,
}

/// 菜单响应
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MenuResponse {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub menu_type: String,
    pub path: Option<String>,
    pub component: Option<String>,
    pub icon: Option<String>,
    pub permission: Option<String>,
    pub sort: i32,
    pub is_show: bool,
    pub is_cache: bool,
    pub is_external: bool,
    pub status: i16,
    /// 子菜单
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<MenuResponse>>,
}

/// 菜单树响应（用于前端路由）
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MenuTreeResponse {
    pub id: String,
    pub name: String,
    pub menu_type: String,
    pub path: Option<String>,
    pub component: Option<String>,
    pub icon: Option<String>,
    pub sort: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<MenuTreeResponse>>,
}

fn default_true() -> bool {
    true
}

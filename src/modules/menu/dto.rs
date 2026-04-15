use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MenuType {
    Catalog,
    Menu,
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

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "catalog" => Some(MenuType::Catalog),
            "menu" => Some(MenuType::Menu),
            "button" => Some(MenuType::Button),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateMenuRequest {
    pub parent_id: Option<String>,
    pub name: String,
    pub menu_type: String,
    pub path: Option<String>,
    pub component: Option<String>,
    pub icon: Option<String>,
    pub permission: Option<String>,
    #[serde(default)]
    pub sort: i32,
    #[serde(default = "default_true")]
    pub is_show: bool,
    #[serde(default)]
    pub is_cache: bool,
    #[serde(default)]
    pub is_external: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMenuRequest {
    pub parent_id: Option<String>,
    pub name: Option<String>,
    pub menu_type: Option<String>,
    pub path: Option<String>,
    pub component: Option<String>,
    pub icon: Option<String>,
    pub permission: Option<String>,
    pub sort: Option<i32>,
    pub is_show: Option<bool>,
    pub is_cache: Option<bool>,
    pub is_external: Option<bool>,
    pub status: Option<i16>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteMenusRequest {
    pub ids: Vec<String>,
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<MenuResponse>>,
}

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
    pub is_show: bool,
    pub is_cache: bool,
    pub is_external: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<MenuTreeResponse>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ButtonOptionResponse {
    pub id: String,
    pub label: String,
    pub code: String,
}

fn default_true() -> bool {
    true
}

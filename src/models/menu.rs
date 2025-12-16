use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "menus")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
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
    pub created_time: DateTime,
    pub created_id: Option<Uuid>,
    pub updated_time: DateTime,
    pub updated_id: Option<Uuid>,
    pub deleted_time: Option<DateTime>,
    pub deleted_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::role_menu::Entity")]
    RoleMenus,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentId",
        to = "Column::Id"
    )]
    Parent,
}

impl Related<super::role_menu::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RoleMenus.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

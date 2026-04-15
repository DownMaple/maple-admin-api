use super::handler;
use crate::common::middleware::auth_middleware;
use salvo::prelude::*;

pub fn routes() -> Router {
    Router::with_path("menu")
        .hoop(auth_middleware)
        .push(Router::with_path("list").get(handler::get_menu_list))
        .push(Router::with_path("tree").get(handler::get_menu_tree))
        .push(Router::with_path("buttons").get(handler::get_button_options))
        .push(Router::with_path("getUserRoutes").get(handler::get_user_menus))
        .push(Router::with_path("permissions").get(handler::get_user_permissions))
        .push(Router::with_path("batch-delete").post(handler::batch_delete_menus))
        .push(Router::new().post(handler::create_menu))
        .push(
            Router::with_path("{id}")
                .get(handler::get_menu)
                .put(handler::update_menu)
                .delete(handler::delete_menu),
        )
}

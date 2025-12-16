use salvo::prelude::*;
use crate::common::middleware::auth_middleware;
use super::handler;

pub fn routes() -> Router {
    Router::with_path("menu")
        .hoop(auth_middleware)
        .push(Router::with_path("list").get(handler::get_menu_list))
        .push(Router::with_path("getUserRoutes").get(handler::get_user_menus))
        .push(Router::with_path("permissions").get(handler::get_user_permissions))
        .push(Router::new().post(handler::create_menu))
        .push(
            Router::with_path("<id>")
                .get(handler::get_menu)
                .put(handler::update_menu)
                .delete(handler::delete_menu)
        )
}

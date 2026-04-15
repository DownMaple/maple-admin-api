use crate::common::middleware::auth_middleware;
use crate::modules::role::handler;
use salvo::prelude::*;

pub fn routes() -> Router {
    Router::with_path("role")
        .hoop(auth_middleware)
        .get(handler::get_role_list)
        .post(handler::create_role)
        .push(Router::with_path("enabled").get(handler::get_enabled_roles))
        .push(Router::with_path("batch-delete").post(handler::batch_delete_roles))
        .push(
            Router::with_path("{id}")
                .put(handler::update_role)
                .delete(handler::delete_role)
                .push(
                    Router::with_path("menus")
                        .get(handler::get_role_menu_ids)
                        .put(handler::update_role_menu_ids),
                )
                .push(
                    Router::with_path("buttons")
                        .get(handler::get_role_button_ids)
                        .put(handler::update_role_button_ids),
                ),
        )
}

use crate::common::middleware::auth_middleware;
use crate::modules::user::handler;
use salvo::prelude::*;

pub fn routes() -> Router {
    Router::with_path("user")
        .hoop(auth_middleware)
        .get(handler::get_user_list)
        .post(handler::create_user)
        .push(Router::with_path("batch-delete").post(handler::batch_delete_users))
        .push(
            Router::with_path("{id}")
                .get(handler::get_user)
                .put(handler::update_user)
                .delete(handler::delete_user),
        )
}

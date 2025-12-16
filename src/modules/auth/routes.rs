use salvo::prelude::*;
use crate::common::middleware::auth_middleware;
use super::handler;

pub fn routes() -> Router {
    Router::with_path("auth")
        .push(Router::with_path("login").post(handler::login))
        .push(Router::with_path("register").post(handler::register))
        .push(Router::with_path("logout").post(handler::logout))
        .push(Router::with_path("publicKey").get(handler::get_public_key))
        .push(
            Router::with_path("switchRole")
                .hoop(auth_middleware)
                .post(handler::switch_role)
        )
        .push(
            Router::with_path("getUserInfo")
                .hoop(auth_middleware)
                .get(handler::get_user_info)
        )
}

pub mod auth;
pub mod health;

use salvo::prelude::*;
use crate::middleware::auth_middleware;

pub fn create_router() -> Router {
    Router::new()
        .push(
            Router::with_path("api/v1")
                .push(
                    Router::with_path("health")
                        .get(health::health_check)
                )
                .push(
                    Router::with_path("auth")
                        .push(Router::with_path("login").post(auth::login))
                        .push(Router::with_path("register").post(auth::register))
                        .push(Router::with_path("logout").post(auth::logout))
                )
                .push(
                    Router::with_path("user")
                        .hoop(auth_middleware)  // 需要认证的路由
                        .push(Router::with_path("current").get(auth::current_user))
                )
        )
}

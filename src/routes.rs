use crate::modules;
use salvo::oapi::OpenApi;
use salvo::prelude::*;

pub fn create_router() -> Router {
    Router::with_path("api/v1")
        .push(modules::health::routes())
        .push(modules::auth::routes())
        .push(modules::user::routes())
        .push(modules::role::routes())
        .push(modules::menu::routes())
}

pub fn create_openapi() -> OpenApi {
    let router = create_router();
    OpenApi::new("Maple Admin API", "1.0.0").merge_router(&router)
}

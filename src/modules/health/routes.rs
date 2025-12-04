use salvo::prelude::*;
use super::handler;

pub fn routes() -> Router {
    Router::with_path("health")
        .get(handler::health_check)
}

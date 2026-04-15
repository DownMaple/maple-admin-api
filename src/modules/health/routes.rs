use super::handler;
use salvo::prelude::*;

pub fn routes() -> Router {
    Router::with_path("health").get(handler::health_check)
}

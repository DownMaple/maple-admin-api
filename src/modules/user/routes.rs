use salvo::Router;
use crate::modules::user::handler;

pub fn routes() -> Router {
    Router::with_path("user")
        .push(Router::with_path("getUserList").get(handler::get_user_list))
}
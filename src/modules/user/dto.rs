use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserListQuery {
    pub user_name: Option<String>,
    pub user_gender: Option<String>,
    pub nick_name: Option<String>,
    pub user_phone: Option<String>,
    pub user_email: Option<String>,
    pub status: Option<String>,
    #[serde(default = "default_current")]
    pub current: u64,
    #[serde(default = "default_size")]
    pub size: u64,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserListItem {
    pub id: String,
    pub user_name: String,
    pub user_gender: Option<String>,
    pub nick_name: String,
    pub user_phone: String,
    pub user_email: String,
    pub user_roles: Vec<String>,
    pub status: String,
    pub create_time: String,
    pub update_time: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserDetailResponse {
    pub id: String,
    pub user_name: String,
    pub user_gender: Option<String>,
    pub nick_name: String,
    pub user_phone: String,
    pub user_email: String,
    pub user_roles: Vec<String>,
    pub status: String,
    pub create_time: String,
    pub update_time: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub user_name: String,
    pub password: String,
    pub user_gender: Option<String>,
    pub nick_name: Option<String>,
    pub user_phone: Option<String>,
    pub user_email: Option<String>,
    pub user_roles: Vec<String>,
    pub status: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub user_name: Option<String>,
    pub password: Option<String>,
    pub user_gender: Option<String>,
    pub nick_name: Option<String>,
    pub user_phone: Option<String>,
    pub user_email: Option<String>,
    pub user_roles: Option<Vec<String>>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteUsersRequest {
    pub ids: Vec<String>,
}

fn default_current() -> u64 {
    1
}

fn default_size() -> u64 {
    10
}

#[cfg(test)]
mod tests {
    use super::UserListQuery;
    use salvo::http::uri::Uri;
    use salvo::prelude::Request;

    fn build_request(uri: &str) -> Request {
        let mut req = Request::new();
        req.set_uri(uri.parse::<Uri>().unwrap());
        req
    }

    #[test]
    fn test_parse_user_list_query_query_params() {
        let mut req =
            build_request("http://127.0.0.1:5801/api/v1/user?current=1&size=10&userName=admin");

        let query = req.parse_queries::<UserListQuery>().unwrap();

        assert_eq!(query.current, 1);
        assert_eq!(query.size, 10);
        assert_eq!(query.user_name.as_deref(), Some("admin"));
    }

    #[test]
    fn test_parse_user_list_query_defaults() {
        let mut req = build_request("http://127.0.0.1:5801/api/v1/user");

        let query = req.parse_queries::<UserListQuery>().unwrap();

        assert_eq!(query.current, 1);
        assert_eq!(query.size, 10);
    }
}

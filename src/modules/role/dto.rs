use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoleListQuery {
    pub role_name: Option<String>,
    pub role_code: Option<String>,
    pub status: Option<String>,
    #[serde(default = "default_current")]
    pub current: u64,
    #[serde(default = "default_size")]
    pub size: u64,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RoleListItem {
    pub id: String,
    pub role_name: String,
    pub role_code: String,
    pub role_desc: String,
    pub status: String,
    pub create_time: String,
    pub update_time: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RoleOption {
    pub id: String,
    pub role_name: String,
    pub role_code: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RolePermissionIdsResponse {
    pub ids: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleRequest {
    pub role_name: String,
    pub role_code: String,
    pub role_desc: Option<String>,
    pub status: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoleRequest {
    pub role_name: Option<String>,
    pub role_code: Option<String>,
    pub role_desc: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteRolesRequest {
    pub ids: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRolePermissionIdsRequest {
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
    use super::RoleListQuery;
    use salvo::http::uri::Uri;
    use salvo::prelude::Request;

    fn build_request(uri: &str) -> Request {
        let mut req = Request::new();
        req.set_uri(uri.parse::<Uri>().unwrap());
        req
    }

    #[test]
    fn test_parse_role_list_query_query_params() {
        let mut req =
            build_request("http://127.0.0.1:5801/api/v1/role?current=2&size=20&roleName=admin");

        let query = req.parse_queries::<RoleListQuery>().unwrap();

        assert_eq!(query.current, 2);
        assert_eq!(query.size, 20);
        assert_eq!(query.role_name.as_deref(), Some("admin"));
    }

    #[test]
    fn test_parse_role_list_query_defaults() {
        let mut req = build_request("http://127.0.0.1:5801/api/v1/role");

        let query = req.parse_queries::<RoleListQuery>().unwrap();

        assert_eq!(query.current, 1);
        assert_eq!(query.size, 10);
    }
}

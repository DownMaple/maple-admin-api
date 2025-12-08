use serde::{Deserialize, Serialize};
use salvo::oapi::ToSchema;

/// API 响应结构
#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
#[salvo(schema(bound = "T: ToSchema + 'static"))]
pub struct ApiResponse<T> {
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T>
where
    T: Serialize + Send,
{
    pub fn success(data: T) -> Self {
        Self {
            code: 200,
            message: "success".to_string(),
            data: Some(data),
        }
    }

    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            code: 200,
            message,
            data: Some(data),
        }
    }

    pub fn ok() -> ApiResponse<()> {
        ApiResponse {
            code: 200,
            message: "success".to_string(),
            data: None,
        }
    }

    pub fn ok_with_message(message: String) -> ApiResponse<()> {
        ApiResponse {
            code: 200,
            message,
            data: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PageResponse<T> {
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub items: Vec<T>,
}

impl<T> PageResponse<T>
where
    T: Serialize,
{
    pub fn new(items: Vec<T>, total: u64, page: u64, page_size: u64) -> Self {
        Self {
            total,
            page,
            page_size,
            items,
        }
    }
}


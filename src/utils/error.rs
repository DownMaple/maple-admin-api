use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
    
    #[error("未授权")]
    Unauthorized,
    
    #[error("禁止访问")]
    Forbidden,
    
    #[error("未找到资源")]
    NotFound,
    
    #[error("请求参数错误: {0}")]
    BadRequest(String),
    
    #[error("内部服务器错误: {0}")]
    InternalServerError(String),
    
    #[error("JWT错误: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    
    #[error("密码哈希错误")]
    BcryptError(#[from] bcrypt::BcryptError),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ErrorResponse {
    pub fn new(code: u16, message: String) -> Self {
        Self {
            code,
            message,
            details: None,
        }
    }

    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }
}

#[async_trait]
impl Writer for AppError {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        let status_code = self.status_code();
        let error_response = ErrorResponse::new(
            status_code.as_u16(),
            self.to_string(),
        );
        
        res.status_code(status_code);
        res.render(Json(error_response));
    }
}

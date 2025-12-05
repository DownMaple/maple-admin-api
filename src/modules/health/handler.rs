use salvo::prelude::*;
use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::common::{ApiResponse};

/// 健康检查响应
#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthStatus {
    /// 服务状态
    pub status: String,
    /// 数据库连接状态
    pub database: String,
    /// 版本号
    pub version: String,
}

/// 健康检查
#[endpoint(tags("系统"))]
pub async fn health_check(depot: &Depot) -> Json<ApiResponse<HealthStatus>> {
    // 检查数据库连接
    let db_status = if depot.get::<Arc<sea_orm::DatabaseConnection>>("db").is_ok() {
        "connected"
    } else {
        "disconnected"
    };

    let health = HealthStatus {
        status: "healthy".to_string(),
        database: db_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Json(ApiResponse::success(health))
}

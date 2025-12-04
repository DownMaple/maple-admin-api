use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::common::{ApiResponse};

#[derive(Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub database: String,
    pub version: String,
}

#[handler]
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

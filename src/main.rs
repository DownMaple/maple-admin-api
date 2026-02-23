mod common;
mod models;
mod modules;
mod routes;

use std::sync::Arc;
use salvo::prelude::*;
use salvo::cors::Cors;
use salvo::http::Method;
use salvo::logging::Logger;
use salvo::compression::Compression;
use salvo::oapi::swagger_ui::SwaggerUi;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // 加载配置
    let config = common::AppConfig::from_env();
    tracing::info!("配置加载成功");

    // 确保 PostgreSQL 服务已启动
    if let Err(e) = common::postgres_service::ensure_postgres_running().await {
        tracing::warn!("⚠️  PostgreSQL 服务启动失败: {}", e);
        tracing::warn!("⚠️  应用将尝试继续连接数据库...");
    }

    // 初始化数据库
    let db = common::database::init_db().await;
    
    if db.is_some() {
        tracing::info!("✅ 数据库初始化成功");
    } else {
        tracing::warn!("⚠️  数据库未连接，应用将在无数据库模式下运行");
    }

    // 初始化 RSA 密钥管理器
    if let Err(e) = common::rsa_crypto::init_key_manager() {
        tracing::error!("❌ RSA 密钥管理器初始化失败: {}", e);
        return Err(e.into());
    }
    tracing::info!("✅ RSA 密钥管理器初始化成功");

    // 创建 JWT 服务
    let jwt_service = Arc::new(common::jwt::JwtService::new(
        config.jwt.secret.clone(),
        config.jwt.expiration_hours,
    ));

    // 配置 CORS  
    let cors = Cors::new()
        .allow_origin(&config.cors.allow_origins)
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
            Method::PATCH,
        ])
        .allow_headers(vec!["Content-Type", "Authorization", "Accept", "X-Requested-With"])
        .allow_credentials(true);

    // 创建 OpenAPI 文档
    let doc = routes::create_openapi();
    
    // 创建路由
    let router = Router::new()
        .hoop(Logger::new())
        .hoop(cors.into_handler())
        .hoop(Compression::new())
        .hoop(common::middleware::DepsMiddleware::new(db.map(Arc::new), jwt_service))
        .push(routes::create_router())
        .push(doc.into_router("/api-doc/openapi.json"))
        .push(SwaggerUi::new("/api-doc/openapi.json").into_router("/swagger"));

    // 创建服务
    let acceptor = TcpListener::new(format!("{}:{}", config.server.host, config.server.port))
        .bind()
        .await;
    
    let server = Server::new(acceptor);


    
    tracing::info!(
        "🚀 服务器启动成功，监听地址: http://{}:{}",
        config.server.host,
        config.server.port
    );

    tracing::info!(
        "🚀可视化接口文档地址: http://{}:{}/swagger",
        config.server.host,
        config.server.port,
    );

    // 创建 Service
    let service = Service::new(router);

    // 启动服务器
    server.serve(service).await;

    Ok(())
}

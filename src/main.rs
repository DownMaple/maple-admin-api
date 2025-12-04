mod api;
mod config;
mod db;
mod middleware;
mod utils;

use std::sync::Arc;


use salvo::prelude::*;
use salvo::cors::Cors;
use salvo::http::Method;
use salvo::logging::Logger;
use salvo::compression::Compression;
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
    let config = config::AppConfig::from_env();
    tracing::info!("配置加载成功");

    // 初始化数据库
    let db = db::init_db().await?;
    tracing::info!("数据库初始化成功");

    // 创建 JWT 服务
    let jwt_service = Arc::new(middleware::JwtService::new(
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

    // 创建路由
    let router = Router::new()
        .hoop(Logger::new())
        .hoop(cors.into_handler())
        .hoop(Compression::new())
        .hoop(middleware::create_deps_middleware(Arc::new(db), jwt_service))
        .push(api::create_router());

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

    // 创建 Service
    let service = Service::new(router);

    // 启动服务器
    server.serve(service).await;

    Ok(())
}

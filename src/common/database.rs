use sea_orm::{Database, DatabaseConnection, DbErr};
use std::env;
use std::time::Duration;

pub async fn init_db() -> Option<DatabaseConnection> {
    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            tracing::error!("❌ DATABASE_URL 环境变量未设置");
            eprintln!("\n⚠️  警告: DATABASE_URL 环境变量未设置");
            eprintln!("⚠️  警告: 应用将在无数据库模式下运行");
            eprintln!("⚠️  警告: 所有需要数据库的接口将返回错误\n");
            return None;
        }
    };
    
    let mut opt = sea_orm::ConnectOptions::new(database_url.clone());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);

    match Database::connect(opt).await {
        Ok(db) => {
            tracing::info!("✅ 数据库连接成功");
            Some(db)
        }
        Err(e) => {
            tracing::error!("❌ 数据库连接失败: {}", e);
            tracing::error!("数据库URL: {}", database_url);
            tracing::warn!("⚠️  应用将在无数据库模式下运行");
            tracing::warn!("⚠️  所有需要数据库的接口将返回错误");
            
            eprintln!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("⚠️  数据库连接失败警告");
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("错误信息: {}", e);
            eprintln!("数据库URL: {}", database_url);
            eprintln!("\n可能的原因:");
            eprintln!("  1. 数据库服务未启动");
            eprintln!("  2. 数据库连接信息配置错误");
            eprintln!("  3. 网络连接问题");
            eprintln!("  4. 数据库权限不足");
            eprintln!("\n建议操作:");
            eprintln!("  1. 检查数据库是否运行: docker ps | grep postgres");
            eprintln!("  2. 启动数据库: docker-compose up -d");
            eprintln!("  3. 检查 .env 文件中的 DATABASE_URL 配置");
            eprintln!("\n⚠️  应用将继续运行，但数据库相关功能不可用");
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
            
            None
        }
    }
}

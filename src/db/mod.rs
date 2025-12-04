use sea_orm::{Database, DatabaseConnection, DbErr};
use std::env;
use std::time::Duration;

pub async fn init_db() -> Result<DatabaseConnection, DbErr> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    let mut opt = sea_orm::ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);

    let db = Database::connect(opt).await?;
    
    tracing::info!("数据库连接成功");
    
    Ok(db)
}

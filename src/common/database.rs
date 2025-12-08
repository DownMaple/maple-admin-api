use sea_orm::{Database, DatabaseConnection, DbErr, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait};
use std::env;
use std::time::Duration;
use crate::models::{user, role, user_role};
use uuid::Uuid;
use chrono;

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
            
            if let Err(e) = init_default_data(&db).await {
                tracing::error!("❌ 初始化默认数据失败: {}", e);
            } else {
                tracing::info!("✅ 默认数据初始化完成");
            }
            
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

async fn init_default_data(db: &DatabaseConnection) -> Result<(), DbErr> {
    let super_admin_role_id = Uuid::parse_str("a0000000-0000-0000-0000-000000000001")
        .map_err(|e| DbErr::Custom(format!("UUID解析失败: {}", e)))?;
    let super_admin_user_id = Uuid::parse_str("b0000000-0000-0000-0000-000000000001")
        .map_err(|e| DbErr::Custom(format!("UUID解析失败: {}", e)))?;

    let role_exists = role::Entity::find_by_id(super_admin_role_id)
        .one(db)
        .await?
        .is_some();

    if !role_exists {
        let role = role::ActiveModel {
            id: Set(super_admin_role_id),
            code: Set("superAdmin".to_string()),
            name: Set("超级管理员".to_string()),
            description: Set(Some("系统超级管理员，拥有所有权限，不可编辑删除".to_string())),
            is_system: Set(true),
            status: Set(1),
            created_time: Set(chrono::Utc::now().naive_utc()),
            created_id: Set(None),
            updated_time: Set(chrono::Utc::now().naive_utc()),
            updated_id: Set(None),
            deleted_time: Set(None),
            deleted_id: Set(None),
        };
        role.insert(db).await?;
        tracing::info!("✅ 创建超级管理员角色成功");
    }

    let user_exists = user::Entity::find_by_id(super_admin_user_id)
        .one(db)
        .await?
        .is_some();

    if !user_exists {
        let user = user::ActiveModel {
            id: Set(super_admin_user_id),
            username: Set("superAdmin".to_string()),
            password: Set("$2b$12$qMUWsD1wyBanEjPn6uEjJ.mPfHrtpxfqgsIpOtX9.zgGyrStoNB2W".to_string()),
            real_name: Set("超级管理员".to_string()),
            email: Set(None),
            phone: Set(None),
            avatar: Set(None),
            status: Set(1),
            created_time: Set(chrono::Utc::now().naive_utc()),
            created_id: Set(None),
            updated_time: Set(chrono::Utc::now().naive_utc()),
            updated_id: Set(None),
            deleted_time: Set(None),
            deleted_id: Set(None),
        };
        user.insert(db).await?;
        tracing::info!("✅ 创建超级管理员用户成功（用户名: superAdmin, 密码: superAdmin）");

        let user_role_exists = user_role::Entity::find()
            .filter(user_role::Column::UserId.eq(super_admin_user_id))
            .filter(user_role::Column::RoleId.eq(super_admin_role_id))
            .one(db)
            .await?
            .is_some();

        if !user_role_exists {
            let user_role = user_role::ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(super_admin_user_id),
                role_id: Set(super_admin_role_id),
                created_time: Set(chrono::Utc::now().naive_utc()),
                created_id: Set(None),
            };
            user_role.insert(db).await?;
            tracing::info!("✅ 关联超级管理员用户和角色成功");
        }
    }

    Ok(())
}

use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, Database, DatabaseConnection, DbErr,
    EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

use crate::models::{role, user, user_role};

const MIGRATIONS_DIR: &str = "migrations";

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

            match run_sql_migrations(&db).await {
                Ok(count) => tracing::info!("✅ 数据库迁移执行完成，共执行 {} 个脚本", count),
                Err(e) => {
                    tracing::error!("❌ 数据库迁移执行失败: {}", e);
                    return None;
                }
            }

            if let Err(e) = init_default_data(&db).await {
                tracing::error!("❌ 初始化默认数据失败: {}", e);
            } else {
                tracing::info!("✅ 默认数据初始化完成");
            }

            Some(db)
        }
        Err(e) => {
            tracing::error!("❌ 数据库连接失败: {}", e);
            tracing::error!("数据库 URL: {}", database_url);
            tracing::warn!("⚠️  应用将在无数据库模式下运行");
            tracing::warn!("⚠️  所有需要数据库的接口将返回错误");

            eprintln!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("⚠️  数据库连接失败警告");
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("错误信息: {}", e);
            eprintln!("数据库 URL: {}", database_url);
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

async fn run_sql_migrations(db: &DatabaseConnection) -> Result<usize, DbErr> {
    let migration_files = migration_file_paths()?;
    let mut executed = 0;

    for path in migration_files {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| DbErr::Custom(format!("无法识别迁移文件名: {}", path.display())))?;
        let sql = fs::read_to_string(&path)
            .map_err(|e| DbErr::Custom(format!("读取迁移文件 {} 失败: {}", file_name, e)))?;

        if sql.trim().is_empty() {
            tracing::warn!("⚠️  跳过空迁移文件: {}", file_name);
            continue;
        }

        tracing::info!("执行数据库迁移: {}", file_name);
        db.execute_unprepared(&sql)
            .await
            .map_err(|e| DbErr::Custom(format!("执行迁移 {} 失败: {}", file_name, e)))?;
        executed += 1;
    }

    Ok(executed)
}

fn migration_file_paths() -> Result<Vec<PathBuf>, DbErr> {
    let migrations_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(MIGRATIONS_DIR);
    let mut paths = Vec::new();

    let entries = fs::read_dir(&migrations_dir).map_err(|e| {
        DbErr::Custom(format!(
            "读取迁移目录 {} 失败: {}",
            migrations_dir.display(),
            e
        ))
    })?;

    for entry in entries {
        let path = entry
            .map_err(|e| DbErr::Custom(format!("读取迁移目录项失败: {}", e)))?
            .path();

        if path.extension().and_then(|ext| ext.to_str()) == Some("sql") {
            paths.push(path);
        }
    }

    paths.sort();
    Ok(paths)
}

async fn init_default_data(db: &DatabaseConnection) -> Result<(), DbErr> {
    let super_admin_role_id = Uuid::parse_str("a0000000-0000-0000-0000-000000000001")
        .map_err(|e| DbErr::Custom(format!("UUID 解析失败: {}", e)))?;
    let super_admin_user_id = Uuid::parse_str("b0000000-0000-0000-0000-000000000001")
        .map_err(|e| DbErr::Custom(format!("UUID 解析失败: {}", e)))?;

    let role_exists = role::Entity::find_by_id(super_admin_role_id)
        .one(db)
        .await?
        .is_some();

    if !role_exists {
        let now = Utc::now().naive_utc();
        let role = role::ActiveModel {
            id: Set(super_admin_role_id),
            code: Set("superAdmin".to_string()),
            name: Set("超级管理员".to_string()),
            description: Set(Some(
                "系统超级管理员，拥有所有权限，不可编辑删除".to_string(),
            )),
            is_system: Set(true),
            status: Set(1),
            created_time: Set(now),
            created_id: Set(None),
            updated_time: Set(now),
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
        let now = Utc::now().naive_utc();
        let user = user::ActiveModel {
            id: Set(super_admin_user_id),
            username: Set("superAdmin".to_string()),
            password: Set(
                "$2b$12$qMUWsD1wyBanEjPn6uEjJ.mPfHrtpxfqgsIpOtX9.zgGyrStoNB2W".to_string(),
            ),
            real_name: Set("超级管理员".to_string()),
            gender: Set(None),
            nick_name: Set(Some("超级管理员".to_string())),
            email: Set(None),
            phone: Set(None),
            avatar: Set(None),
            status: Set(1),
            created_time: Set(now),
            created_id: Set(None),
            updated_time: Set(now),
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
                created_time: Set(Utc::now().naive_utc()),
                created_id: Set(None),
            };
            user_role.insert(db).await?;
            tracing::info!("✅ 关联超级管理员用户和角色成功");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::migration_file_paths;

    #[test]
    fn test_migration_file_paths_sorted() {
        let paths = migration_file_paths().expect("should load migration files");
        let file_names: Vec<String> = paths
            .iter()
            .map(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
                    .to_string()
            })
            .collect();

        let mut sorted = file_names.clone();
        sorted.sort();

        assert_eq!(file_names, sorted);
        assert!(file_names.iter().any(|name| name == "001_init_tables.sql"));
        assert!(file_names.iter().any(|name| name == "002_menu_tables.sql"));
    }
}

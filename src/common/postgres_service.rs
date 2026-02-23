// PostgreSQL 服务管理模块
// 用于在 Windows 系统上检测和启动 PostgreSQL 服务

use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

/// 检测 PostgreSQL 服务是否正在运行
/// 使用 pg_isready 命令检测连接状态
pub fn is_postgres_running() -> bool {
    let output = Command::new("pg_isready")
        .args(["-h", "localhost", "-p", "5432"])
        .output();
    
    match output {
        Ok(result) => result.status.success(),
        Err(_) => false,
    }
}

/// 启动 PostgreSQL 服务
/// 在 Windows 上使用 pg_ctl start 命令
/// 使用非阻塞方式启动，避免 Windows 上的阻塞问题
pub fn start_postgres_service() -> Result<(), String> {
    use std::process::Stdio;
    
    // 首先尝试获取 PGDATA 环境变量
    let pgdata = std::env::var("PGDATA").map_err(|_| {
        "PGDATA 环境变量未设置。请确保已正确配置 PostgreSQL 环境变量。".to_string()
    })?;
    
    tracing::info!("📂 PostgreSQL 数据目录: {}", pgdata);
    tracing::info!("⏳ 正在后台启动 PostgreSQL 服务...");
    
    // 使用 spawn() 非阻塞方式启动 pg_ctl
    // 不使用 -w 参数，让 pg_ctl 立即返回
    // 将 stdin/stdout/stderr 设置为 null，避免管道阻塞
    Command::new("pg_ctl")
        .args(["start", "-D", &pgdata, "-l", "postgresql.log"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("执行 pg_ctl 失败: {}", e))?;
    
    Ok(())
}

/// 确保 PostgreSQL 服务正在运行
/// 如果服务未运行，则尝试启动服务并等待其就绪
pub async fn ensure_postgres_running() -> Result<(), String> {
    // 检查服务是否已运行
    if is_postgres_running() {
        tracing::info!("✅ PostgreSQL 服务已在运行");
        return Ok(());
    }
    
    tracing::info!("🔍 PostgreSQL 服务未运行，正在启动...");
    
    // 尝试启动服务
    start_postgres_service()?;
    
    // 等待服务启动完成（最多等待 30 秒）
    let max_attempts = 30;
    for attempt in 1..=max_attempts {
        sleep(Duration::from_secs(1)).await;
        
        if is_postgres_running() {
            tracing::info!("✅ PostgreSQL 服务启动成功（等待了 {} 秒）", attempt);
            return Ok(());
        }
        
        if attempt % 5 == 0 {
            tracing::info!("⏳ 等待 PostgreSQL 服务启动... ({}/{})", attempt, max_attempts);
        }
    }
    
    Err(format!(
        "PostgreSQL 服务在 {} 秒内未能启动成功",
        max_attempts
    ))
}

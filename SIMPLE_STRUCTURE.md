# 简化的项目结构设计

## 设计原则

**按功能模块垂直划分**

每个模块包含该功能的所有代码（模型、服务、处理器、路由）

## 目录结构

```
maple-admin-api/
├── Cargo.toml
├── .env
├── .gitignore
├── README.md
├── docker-compose.yml
│
└── src/
    ├── main.rs                    # 程序入口
    │
    ├── common/                    # 公共模块
    │   ├── mod.rs
    │   ├── config.rs              # 配置管理
    │   ├── database.rs            # 数据库连接
    │   ├── error.rs               # 错误定义
    │   ├── response.rs            # 响应格式
    │   ├── middleware.rs          # 全局中间件
    │   ├── jwt.rs                 # JWT 工具
    │   ├── crypto.rs              # 加密工具
    │   └── constants.rs           # 常量
    │
    └── modules/                   # 业务模块
        │
        ├── auth/                  # 认证模块
        │   ├── mod.rs
        │   ├── model.rs           # 数据模型
        │   ├── service.rs         # 业务逻辑
        │   ├── handler.rs         # HTTP 处理器
        │   ├── dto.rs             # 请求/响应 DTO
        │   └── routes.rs          # 路由定义
        │
        ├── user/                  # 用户模块
        │   ├── mod.rs
        │   ├── model.rs
        │   ├── service.rs
        │   ├── handler.rs
        │   ├── dto.rs
        │   └── routes.rs
        │
        ├── role/                  # 角色模块
        │   ├── mod.rs
        │   ├── model.rs
        │   ├── service.rs
        │   ├── handler.rs
        │   ├── dto.rs
        │   └── routes.rs
        │
        ├── permission/            # 权限模块
        │   ├── mod.rs
        │   ├── model.rs
        │   ├── service.rs
        │   ├── handler.rs
        │   ├── dto.rs
        │   └── routes.rs
        │
        ├── menu/                  # 菜单模块
        │   ├── mod.rs
        │   ├── model.rs
        │   ├── service.rs
        │   ├── handler.rs
        │   ├── dto.rs
        │   └── routes.rs
        │
        ├── audit_log/             # 审计日志模块
        │   ├── mod.rs
        │   ├── model.rs
        │   ├── service.rs
        │   ├── handler.rs
        │   ├── dto.rs
        │   └── routes.rs
        │
        ├── system/                # 系统管理模块
        │   ├── mod.rs
        │   ├── model.rs
        │   ├── service.rs
        │   ├── handler.rs
        │   ├── dto.rs
        │   └── routes.rs
        │
        └── health/                # 健康检查模块
            ├── mod.rs
            ├── handler.rs
            └── routes.rs
```

## 模块文件说明

每个业务模块包含：

- **mod.rs**: 模块导出
- **model.rs**: 数据模型（实体、SeaORM Entity）
- **service.rs**: 业务逻辑（数据库操作、业务规则）
- **handler.rs**: HTTP 请求处理器
- **dto.rs**: 数据传输对象（请求/响应结构）
- **routes.rs**: 路由定义

## 模块示例：Auth 模块

```rust
// modules/auth/model.rs
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime,
}

// modules/auth/dto.rs
#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
}

// modules/auth/service.rs
pub struct AuthService {
    db: Arc<DatabaseConnection>,
}

impl AuthService {
    pub async fn login(&self, req: LoginRequest) -> Result<LoginResponse> {
        // 业务逻辑
    }
    
    pub async fn register(&self, req: RegisterRequest) -> Result<()> {
        // 业务逻辑
    }
}

// modules/auth/handler.rs
#[handler]
pub async fn login(req: JsonBody<LoginRequest>, depot: &Depot) -> Result<Json<ApiResponse<LoginResponse>>> {
    let service = AuthService::from_depot(depot);
    let result = service.login(req.into_inner()).await?;
    Ok(Json(ApiResponse::success(result)))
}

// modules/auth/routes.rs
pub fn routes() -> Router {
    Router::with_path("auth")
        .push(Router::with_path("login").post(login))
        .push(Router::with_path("register").post(register))
        .push(Router::with_path("logout").post(logout))
}
```

## Common 模块说明

公共模块包含跨模块共享的代码：

- **config.rs**: 应用配置（从环境变量加载）
- **database.rs**: 数据库连接池管理
- **error.rs**: 统一错误定义
- **response.rs**: 统一响应格式
- **middleware.rs**: 全局中间件（CORS、日志、认证等）
- **jwt.rs**: JWT 工具函数
- **crypto.rs**: 加密工具函数
- **constants.rs**: 全局常量

## Main.rs 结构

```rust
mod common;
mod modules;

use salvo::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt().init();
    
    // 加载配置
    let config = common::config::AppConfig::from_env();
    
    // 初始化数据库
    let db = common::database::init_db(&config.database.url).await?;
    
    // 创建路由
    let router = Router::new()
        .push(Router::with_path("api/v1")
            .push(modules::health::routes())
            .push(modules::auth::routes())
            .push(modules::user::routes())
            // ... 其他模块路由
        );
    
    // 启动服务器
    Server::new(TcpListener::bind(&format!("{}:{}", config.server.host, config.server.port)))
        .serve(router)
        .await;
    
    Ok(())
}
```

## 优势

1. **简单直观**：每个模块一个文件夹，结构清晰
2. **易于理解**：新人可以快速找到对应功能的代码
3. **独立开发**：不同模块可以并行开发
4. **易于扩展**：新增功能只需添加新模块文件夹
5. **低学习成本**：不需要理解复杂的分层架构

## 迁移步骤

1. ✅ 初始化 git 仓库
2. 创建 common 和 modules 目录
3. 迁移现有代码到新结构
4. 更新 main.rs
5. 测试验证

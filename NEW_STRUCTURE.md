# 新的项目结构设计

## 设计原则

**垂直划分（功能模块） + 水平划分（分层架构）**

- 第一层：按业务功能垂直划分模块
- 第二层：每个模块内部按分层架构水平划分

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
    ├── common/                    # 公共模块（跨模块共享）
    │   ├── mod.rs
    │   ├── config/                # 配置管理
    │   │   ├── mod.rs
    │   │   └── app_config.rs
    │   ├── database/              # 数据库连接
    │   │   ├── mod.rs
    │   │   └── connection.rs
    │   ├── middleware/            # 全局中间件
    │   │   ├── mod.rs
    │   │   ├── cors.rs
    │   │   ├── logger.rs
    │   │   └── error_handler.rs
    │   ├── errors/                # 错误定义
    │   │   ├── mod.rs
    │   │   ├── app_error.rs
    │   │   └── error_codes.rs
    │   ├── responses/             # 统一响应格式
    │   │   ├── mod.rs
    │   │   ├── api_response.rs
    │   │   └── page_response.rs
    │   ├── utils/                 # 工具函数
    │   │   ├── mod.rs
    │   │   ├── jwt.rs
    │   │   ├── crypto.rs
    │   │   ├── datetime.rs
    │   │   └── validator.rs
    │   └── constants.rs           # 常量定义
    │
    └── modules/                   # 业务模块（垂直划分）
        │
        ├── auth/                  # 认证模块
        │   ├── mod.rs
        │   ├── domain/            # 领域层
        │   │   ├── mod.rs
        │   │   ├── entities.rs    # 实体
        │   │   └── services.rs    # 领域服务
        │   ├── application/       # 应用层
        │   │   ├── mod.rs
        │   │   ├── dto.rs         # 数据传输对象
        │   │   └── use_cases.rs   # 用例
        │   ├── infrastructure/    # 基础设施层
        │   │   ├── mod.rs
        │   │   └── repository.rs  # 仓储实现
        │   └── presentation/      # 表现层
        │       ├── mod.rs
        │       ├── handlers.rs    # HTTP 处理器
        │       ├── routes.rs      # 路由定义
        │       └── middleware.rs  # 模块中间件
        │
        ├── user/                  # 用户模块
        │   ├── mod.rs
        │   ├── domain/
        │   │   ├── mod.rs
        │   │   ├── entities.rs
        │   │   └── services.rs
        │   ├── application/
        │   │   ├── mod.rs
        │   │   ├── dto.rs
        │   │   └── use_cases.rs
        │   ├── infrastructure/
        │   │   ├── mod.rs
        │   │   └── repository.rs
        │   └── presentation/
        │       ├── mod.rs
        │       ├── handlers.rs
        │       ├── routes.rs
        │       └── middleware.rs
        │
        ├── role/                  # 角色模块
        │   ├── mod.rs
        │   ├── domain/
        │   ├── application/
        │   ├── infrastructure/
        │   └── presentation/
        │
        ├── permission/            # 权限模块
        │   ├── mod.rs
        │   ├── domain/
        │   ├── application/
        │   ├── infrastructure/
        │   └── presentation/
        │
        ├── menu/                  # 菜单模块
        │   ├── mod.rs
        │   ├── domain/
        │   ├── application/
        │   ├── infrastructure/
        │   └── presentation/
        │
        ├── audit_log/             # 审计日志模块
        │   ├── mod.rs
        │   ├── domain/
        │   ├── application/
        │   ├── infrastructure/
        │   └── presentation/
        │
        └── system/                # 系统管理模块
            ├── mod.rs
            ├── domain/
            ├── application/
            ├── infrastructure/
            └── presentation/
```

## 模块说明

### Common（公共模块）
跨模块共享的代码，包括：
- **config**: 应用配置
- **database**: 数据库连接池
- **middleware**: 全局中间件（CORS、日志、错误处理）
- **errors**: 统一错误定义
- **responses**: 统一响应格式
- **utils**: 工具函数（JWT、加密、验证等）
- **constants**: 全局常量

### Modules（业务模块）
每个业务模块采用 DDD 分层架构：

#### 1. Domain（领域层）
- **entities.rs**: 实体和值对象
- **services.rs**: 领域服务（核心业务逻辑）

#### 2. Application（应用层）
- **dto.rs**: 数据传输对象（请求/响应）
- **use_cases.rs**: 用例实现（协调领域对象）

#### 3. Infrastructure（基础设施层）
- **repository.rs**: 数据访问实现
- 其他外部服务集成

#### 4. Presentation（表现层）
- **handlers.rs**: HTTP 请求处理器
- **routes.rs**: 路由定义
- **middleware.rs**: 模块特定中间件

## 依赖关系

```
presentation → application → domain ← infrastructure
       ↓            ↓           ↓           ↓
                   common (公共模块)
```

- presentation 依赖 application
- application 依赖 domain
- infrastructure 实现 domain 的接口
- 所有层都可以使用 common

## 模块示例：Auth 模块

```rust
// modules/auth/domain/entities.rs
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

// modules/auth/domain/services.rs
pub struct AuthService {
    // 领域服务：密码验证、token 生成等
}

// modules/auth/application/dto.rs
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub struct LoginResponse {
    pub token: String,
    pub user: UserDto,
}

// modules/auth/application/use_cases.rs
pub async fn login(req: LoginRequest) -> Result<LoginResponse> {
    // 协调领域对象完成登录
}

// modules/auth/infrastructure/repository.rs
pub struct UserRepository {
    // 数据库访问实现
}

// modules/auth/presentation/handlers.rs
#[handler]
pub async fn login_handler(req: JsonBody<LoginRequest>) -> Result<Json<ApiResponse<LoginResponse>>> {
    // HTTP 请求处理
}

// modules/auth/presentation/routes.rs
pub fn routes() -> Router {
    Router::with_path("auth")
        .push(Router::with_path("login").post(login_handler))
}
```

## 优势

1. **清晰的模块边界**：每个功能模块独立，易于理解和维护
2. **分层架构**：每个模块内部职责明确，易于测试
3. **高内聚低耦合**：模块间通过明确的接口交互
4. **易于扩展**：新增功能只需添加新模块
5. **团队协作友好**：不同成员可并行开发不同模块
6. **符合 DDD 原则**：领域驱动设计，业务逻辑清晰

## 迁移计划

1. ✅ 初始化 git 仓库
2. 创建 common 公共模块
3. 创建 auth 模块（作为示例）
4. 迁移现有代码到新结构
5. 测试验证
6. 逐步添加其他模块

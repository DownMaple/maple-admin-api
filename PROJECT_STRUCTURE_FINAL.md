# Maple Admin API - 最终项目结构

## ✅ 重构完成

项目已成功重构为**按功能模块垂直划分**的结构。

## 当前目录结构

```
maple-admin-api/
├── Cargo.toml
├── .env
├── .env.example
├── .gitignore
├── README.md
├── docker-compose.yml
├── init-db.sh
│
└── src/
    ├── main.rs                         # 程序入口
    │
    ├── common/                         # 公共模块
    │   ├── mod.rs                      # 模块导出
    │   ├── config.rs                   # 配置管理
    │   ├── database.rs                 # 数据库连接
    │   ├── error.rs                    # 错误定义
    │   ├── response.rs                 # 响应格式
    │   ├── middleware.rs               # 全局中间件
    │   ├── jwt.rs                      # JWT 工具
    │   ├── crypto.rs                   # 加密工具
    │   └── constants.rs                # 常量定义
    │
    └── modules/                        # 业务模块
        ├── mod.rs                      # 模块导出
        │
        ├── health/                     # ✅ 健康检查模块
        │   ├── mod.rs
        │   ├── handler.rs
        │   └── routes.rs
        │
        ├── auth/                       # ✅ 认证模块
        │   ├── mod.rs
        │   ├── dto.rs                  # 数据传输对象
        │   ├── service.rs              # 业务逻辑
        │   ├── handler.rs              # HTTP 处理器
        │   └── routes.rs               # 路由定义
        │
        ├── user/                       # ⏳ 用户模块（待实现）
        │   └── mod.rs
        │
        ├── role/                       # ⏳ 角色模块（待实现）
        │   └── mod.rs
        │
        ├── permission/                 # ⏳ 权限模块（待实现）
        │   └── mod.rs
        │
        ├── menu/                       # ⏳ 菜单模块（待实现）
        │   └── mod.rs
        │
        ├── audit_log/                  # ⏳ 审计日志模块（待实现）
        │   └── mod.rs
        │
        └── system/                     # ⏳ 系统管理模块（待实现）
            └── mod.rs
```

## 模块说明

### Common（公共模块）

所有业务模块共享的代码：

| 文件 | 说明 |
|------|------|
| `config.rs` | 应用配置（从环境变量加载） |
| `database.rs` | 数据库连接池管理 |
| `error.rs` | 统一错误定义（AppError, ErrorResponse） |
| `response.rs` | 统一响应格式（ApiResponse, PageResponse） |
| `middleware.rs` | 全局中间件（依赖注入、JWT认证） |
| `jwt.rs` | JWT 工具（生成、验证 token） |
| `crypto.rs` | 加密工具（密码哈希、验证） |
| `constants.rs` | 全局常量 |

### Modules（业务模块）

#### ✅ Health 模块
- **功能**: 健康检查
- **端点**: `GET /api/v1/health`
- **状态**: 已实现

#### ✅ Auth 模块
- **功能**: 用户认证
- **端点**: 
  - `POST /api/v1/auth/login` - 登录
  - `POST /api/v1/auth/register` - 注册
  - `POST /api/v1/auth/logout` - 登出
  - `GET /api/v1/auth/current` - 获取当前用户（需认证）
- **状态**: 已实现（使用模拟数据）

#### ⏳ 其他模块
- User（用户管理）
- Role（角色管理）
- Permission（权限管理）
- Menu（菜单管理）
- AuditLog（审计日志）
- System（系统管理）

## 模块开发模板

每个新模块应包含以下文件：

```rust
// modules/example/mod.rs
mod dto;
mod model;      // 可选：如果需要数据库模型
mod service;
mod handler;
mod routes;

pub use routes::routes;

// modules/example/dto.rs
// 定义请求和响应的数据结构

// modules/example/model.rs
// 定义数据库实体（SeaORM）

// modules/example/service.rs
// 实现业务逻辑

// modules/example/handler.rs
// 实现 HTTP 请求处理器

// modules/example/routes.rs
// 定义路由
pub fn routes() -> Router {
    Router::with_path("example")
        .push(Router::with_path("list").get(handler::list))
        .push(Router::with_path("create").post(handler::create))
}
```

## API 测试

### 健康检查
```bash
curl http://localhost:3000/api/v1/health
```

### 登录
```bash
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'
```

### 获取当前用户（需要 token）
```bash
TOKEN="your_jwt_token"
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/api/v1/auth/current
```

## Git 提交历史

```
* 275d6c9 - 清理: 删除旧的目录结构
* 1f14f9e - 重构: 按功能模块垂直划分项目结构
* d840ed1 - Initial commit: 基础项目架构
```

## 下一步计划

1. **实现 User 模块**
   - 创建用户数据模型（SeaORM Entity）
   - 实现用户 CRUD 操作
   - 完善用户认证逻辑

2. **实现 Role 和 Permission 模块**
   - 设计 RBAC 权限系统
   - 实现角色和权限管理

3. **实现其他业务模块**
   - Menu（菜单管理）
   - AuditLog（审计日志）
   - System（系统配置）

## 优势总结

✅ **清晰的模块边界**: 每个功能独立在自己的目录中  
✅ **易于理解**: 新人可以快速找到对应功能的代码  
✅ **便于协作**: 不同开发者可以并行开发不同模块  
✅ **易于扩展**: 新增功能只需添加新模块目录  
✅ **低耦合**: 模块间通过明确的接口交互  
✅ **可测试**: 每个模块可以独立测试  

## 注意事项

- 所有新模块都应该在 `src/modules/mod.rs` 中声明
- 所有路由都应该在 `src/main.rs` 中注册
- 公共功能应该放在 `common/` 目录
- 遵循统一的错误处理和响应格式

# Maple Admin API 开发计划

## ✅ 已完成的基础架构

### 1. 项目基础设施
- ✅ Rust + Salvo 框架搭建
- ✅ PostgreSQL 数据库连接（使用 Docker）
- ✅ SeaORM 作为 ORM 框架
- ✅ 环境变量配置管理
- ✅ 项目目录结构组织

### 2. 核心功能模块
- ✅ JWT 认证中间件
- ✅ CORS 跨域配置
- ✅ 全局错误处理
- ✅ 统一响应格式
- ✅ 请求日志中间件
- ✅ 依赖注入机制

### 3. 已实现的 API 端点
- ✅ `GET /api/v1/health` - 健康检查
- ✅ `POST /api/v1/auth/login` - 用户登录
- ✅ `POST /api/v1/auth/register` - 用户注册（占位）
- ✅ `POST /api/v1/auth/logout` - 用户登出
- ✅ `GET /api/v1/user/current` - 获取当前用户（需认证）

## 📋 待开发的功能模块

### 1. 用户管理模块
```rust
// src/models/user.rs
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**需要实现的功能：**
- 用户 CRUD 操作
- 密码加密与验证
- 用户状态管理（启用/禁用）
- 用户信息更新
- 密码重置功能

### 2. 角色权限管理
```rust
// src/models/role.rs
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub permissions: Vec<Permission>,
}

// src/models/permission.rs
pub struct Permission {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub resource: String,
    pub action: String,
}
```

**需要实现的功能：**
- 角色 CRUD 操作
- 权限 CRUD 操作
- 用户角色关联
- 角色权限分配
- 基于 RBAC 的访问控制中间件

### 3. 菜单管理模块
```rust
// src/models/menu.rs
pub struct Menu {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub path: String,
    pub icon: Option<String>,
    pub component: Option<String>,
    pub sort_order: i32,
    pub visible: bool,
}
```

**需要实现的功能：**
- 菜单树结构管理
- 菜单权限关联
- 动态菜单生成
- 菜单排序

### 4. 系统日志模块
```rust
// src/models/audit_log.rs
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub resource: String,
    pub ip_address: String,
    pub user_agent: String,
    pub request_data: Option<JsonValue>,
    pub response_data: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
}
```

**需要实现的功能：**
- 操作日志记录
- 登录日志记录
- 日志查询与分析
- 日志导出

### 5. 文件管理模块
```rust
// src/models/file.rs
pub struct File {
    pub id: Uuid,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub size: i64,
    pub storage_path: String,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
}
```

**需要实现的功能：**
- 文件上传（支持多文件）
- 文件下载
- 文件预览
- 文件删除
- 存储策略（本地/云存储）

### 6. 系统配置模块
```rust
// src/models/config.rs
pub struct SystemConfig {
    pub id: Uuid,
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub config_type: ConfigType,
}
```

**需要实现的功能：**
- 系统参数配置
- 动态配置加载
- 配置缓存机制

### 7. 数据字典模块
```rust
// src/models/dictionary.rs
pub struct Dictionary {
    pub id: Uuid,
    pub type_code: String,
    pub type_name: String,
    pub items: Vec<DictionaryItem>,
}

pub struct DictionaryItem {
    pub id: Uuid,
    pub dict_id: Uuid,
    pub label: String,
    pub value: String,
    pub sort_order: i32,
}
```

**需要实现的功能：**
- 字典类型管理
- 字典项管理
- 字典缓存

## 🔧 技术改进建议

### 1. 数据库相关
- [ ] 实现数据库迁移（使用 sea-orm-migration）
- [ ] 添加数据库连接池监控
- [ ] 实现事务处理机制
- [ ] 添加数据库备份恢复功能

### 2. 安全性增强
- [ ] 实现 API 限流（Rate Limiting）
- [ ] 添加请求签名验证
- [ ] 实现 CSRF 防护
- [ ] 添加 SQL 注入防护
- [ ] 实现敏感数据加密存储

### 3. 性能优化
- [ ] 添加 Redis 缓存层
- [ ] 实现接口响应缓存
- [ ] 添加数据库查询优化
- [ ] 实现异步任务队列

### 4. 监控与运维
- [ ] 添加 Prometheus 指标收集
- [ ] 实现健康检查详细信息
- [ ] 添加错误追踪（Sentry）
- [ ] 实现日志聚合分析

### 5. 开发体验
- [ ] 添加 Swagger/OpenAPI 文档
- [ ] 实现自动化测试
- [ ] 添加 Docker 容器化部署
- [ ] 实现 CI/CD 流程

## 🚀 下一步开发建议

1. **首先实现用户管理模块**
   - 创建用户表迁移
   - 实现用户 CRUD API
   - 完善用户认证逻辑

2. **然后实现角色权限系统**
   - 设计 RBAC 数据模型
   - 实现权限检查中间件
   - 添加动态权限配置

3. **最后完善业务功能**
   - 根据实际需求添加业务模块
   - 优化系统性能
   - 完善监控和日志

## 📝 注意事项

- 所有敏感信息都应该通过环境变量配置
- 生产环境需要更改 JWT_SECRET
- 建议使用 HTTPS 部署
- 定期更新依赖库版本
- 做好数据备份策略

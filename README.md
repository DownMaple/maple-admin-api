# Maple Admin API

基于 Rust + Salvo 框架构建的后台管理系统 API 服务。

## 技术栈

- **Web 框架**: Salvo 0.80
- **ORM**: SeaORM
- **数据库**: PostgreSQL
- **认证**: JWT
- **异步运行时**: Tokio

## 项目结构

```
maple-admin-api/
├── src/
│   ├── api/           # API 路由处理器
│   ├── config/        # 配置管理
│   ├── db/            # 数据库连接和迁移
│   ├── middleware/    # 中间件（认证、日志等）
│   ├── models/        # 数据模型
│   ├── services/      # 业务逻辑层
│   ├── utils/         # 工具函数
│   └── main.rs        # 程序入口
├── .env               # 环境变量配置
├── Cargo.toml         # 项目依赖
└── docker-compose.yml # Docker 配置
```

## 快速开始

### 1. 启动数据库

```bash
# 使用 Docker Compose 启动 PostgreSQL
docker-compose up -d

# 或者手动运行 PostgreSQL 容器
docker run --name maple_postgres \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=maple_admin \
  -p 5432:5432 \
  -d postgres:16-alpine
```

### 2. 配置环境变量

复制 `.env.example` 到 `.env` 并修改配置：

```bash
cp .env.example .env
```

### 3. 运行项目

```bash
# 安装依赖并运行
cargo run

# 或者使用 watch 模式开发
cargo install cargo-watch
cargo watch -x run
```

服务器将在 `http://127.0.0.1:3000` 启动。

## API 端点

### 公开接口

- `GET /api/v1/health` - 健康检查
- `POST /api/v1/auth/login` - 用户登录
- `POST /api/v1/auth/register` - 用户注册

### 需要认证的接口

- `GET /api/v1/user/current` - 获取当前用户信息
- `POST /api/v1/auth/logout` - 用户登出

## 认证

使用 JWT Bearer Token 认证，在请求头中添加：

```
Authorization: Bearer <your_jwt_token>
```

## 测试 API

### 登录测试

```bash
curl -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'
```

### 健康检查

```bash
curl http://localhost:3000/api/v1/health
```

## 开发计划

- [ ] 用户管理模块
- [ ] 角色权限管理
- [ ] 菜单管理
- [ ] 系统日志
- [ ] 文件上传
- [ ] 数据字典
- [ ] 系统配置

## 环境变量说明

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| DATABASE_URL | 数据库连接字符串 | - |
| JWT_SECRET | JWT 密钥 | - |
| JWT_EXPIRATION_HOURS | JWT 过期时间（小时） | 24 |
| SERVER_HOST | 服务器监听地址 | 127.0.0.1 |
| SERVER_PORT | 服务器端口 | 3000 |
| RUST_LOG | 日志级别 | info |

## License

MIT

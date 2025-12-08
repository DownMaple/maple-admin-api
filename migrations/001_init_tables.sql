-- 创建用户表
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    real_name VARCHAR(100) NOT NULL,
    email VARCHAR(100),
    phone VARCHAR(20),
    avatar VARCHAR(500),
    status SMALLINT NOT NULL DEFAULT 1,
    created_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_id UUID,
    updated_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_id UUID,
    deleted_time TIMESTAMP,
    deleted_id UUID
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_users_deleted_time ON users(deleted_time);

-- 创建角色表
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    status SMALLINT NOT NULL DEFAULT 1,
    created_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_id UUID,
    updated_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_id UUID,
    deleted_time TIMESTAMP,
    deleted_id UUID
);

CREATE INDEX idx_roles_code ON roles(code);
CREATE INDEX idx_roles_status ON roles(status);
CREATE INDEX idx_roles_deleted_time ON roles(deleted_time);

-- 创建用户角色关联表
CREATE TABLE IF NOT EXISTS user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    created_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_id UUID,
    UNIQUE(user_id, role_id)
);

CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);

-- 插入超级管理员角色
INSERT INTO roles (id, code, name, description, is_system, status)
VALUES (
    'a0000000-0000-0000-0000-000000000001'::UUID,
    'superAdmin',
    '超级管理员',
    '系统超级管理员，拥有所有权限，不可编辑删除',
    TRUE,
    1
);

-- 插入超级管理员用户
-- 密码: superAdmin (使用bcrypt加密)
INSERT INTO users (id, username, password, real_name, status)
VALUES (
    'b0000000-0000-0000-0000-000000000001'::UUID,
    'superAdmin',
    '$2b$12$qMUWsD1wyBanEjPn6uEjJ.mPfHrtpxfqgsIpOtX9.zgGyrStoNB2W',
    'superAdmin',
    1
);

-- 关联超级管理员用户和角色
INSERT INTO user_roles (user_id, role_id)
VALUES (
    'b0000000-0000-0000-0000-000000000001'::UUID,
    'a0000000-0000-0000-0000-000000000001'::UUID
);

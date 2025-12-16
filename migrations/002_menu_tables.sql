-- 创建菜单表
CREATE TABLE IF NOT EXISTS menus (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID,                              -- 父菜单ID（NULL表示顶级目录）
    name VARCHAR(50) NOT NULL,                   -- 菜单名称
    menu_type VARCHAR(20) NOT NULL,              -- 类型：catalog=目录, menu=菜单, button=按钮
    path VARCHAR(255),                           -- 路由路径
    component VARCHAR(255),                      -- 前端组件路径
    icon VARCHAR(100),                           -- 图标
    permission VARCHAR(100),                     -- 权限标识
    sort INTEGER NOT NULL DEFAULT 0,             -- 排序值
    is_show BOOLEAN NOT NULL DEFAULT TRUE,       -- 是否显示
    is_cache BOOLEAN NOT NULL DEFAULT FALSE,     -- 是否缓存
    is_external BOOLEAN NOT NULL DEFAULT FALSE,  -- 是否外链
    status SMALLINT NOT NULL DEFAULT 1,          -- 状态：1-正常，0-禁用
    created_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_id UUID,
    updated_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_id UUID,
    deleted_time TIMESTAMP,
    deleted_id UUID,
    -- 自引用外键
    FOREIGN KEY (parent_id) REFERENCES menus(id) ON DELETE CASCADE,
    -- 类型约束
    CONSTRAINT chk_menu_type CHECK (menu_type IN ('catalog', 'menu', 'button'))
);

CREATE INDEX idx_menus_parent_id ON menus(parent_id);
CREATE INDEX idx_menus_status ON menus(status);
CREATE INDEX idx_menus_sort ON menus(sort);
CREATE INDEX idx_menus_deleted_time ON menus(deleted_time);

-- 创建角色菜单关联表
CREATE TABLE IF NOT EXISTS role_menus (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    menu_id UUID NOT NULL REFERENCES menus(id) ON DELETE CASCADE,
    created_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_id UUID,
    UNIQUE(role_id, menu_id)
);

CREATE INDEX idx_role_menus_role_id ON role_menus(role_id);
CREATE INDEX idx_role_menus_menu_id ON role_menus(menu_id);

-- 插入默认菜单数据
-- 首页
INSERT INTO menus (id, parent_id, name, menu_type, path, component, icon, permission, sort, is_show)
VALUES (
    'c0000000-0000-0000-0000-000000000001'::UUID,
    NULL,
    '首页',
    'menu',
    '/home',
    '/views/home/index',
    'mdi:home',
    NULL,
    0,
    TRUE
);

-- 系统管理目录
INSERT INTO menus (id, parent_id, name, menu_type, path, component, icon, permission, sort, is_show)
VALUES (
    'c0000000-0000-0000-0000-000000000100'::UUID,
    NULL,
    '系统管理',
    'catalog',
    '/system',
    NULL,
    'mdi:cog',
    NULL,
    100,
    TRUE
);

-- 用户管理菜单
INSERT INTO menus (id, parent_id, name, menu_type, path, component, icon, permission, sort, is_show)
VALUES (
    'c0000000-0000-0000-0000-000000000101'::UUID,
    'c0000000-0000-0000-0000-000000000100'::UUID,
    '用户管理',
    'menu',
    '/system/user',
    '/views/system/user/index',
    'mdi:account',
    'system:user:list',
    1,
    TRUE
);

-- 用户管理按钮
INSERT INTO menus (id, parent_id, name, menu_type, path, component, icon, permission, sort, is_show)
VALUES 
    ('c0000000-0000-0000-0000-000000000111'::UUID, 'c0000000-0000-0000-0000-000000000101'::UUID, '新增用户', 'button', NULL, NULL, NULL, 'system:user:add', 1, FALSE),
    ('c0000000-0000-0000-0000-000000000112'::UUID, 'c0000000-0000-0000-0000-000000000101'::UUID, '编辑用户', 'button', NULL, NULL, NULL, 'system:user:edit', 2, FALSE),
    ('c0000000-0000-0000-0000-000000000113'::UUID, 'c0000000-0000-0000-0000-000000000101'::UUID, '删除用户', 'button', NULL, NULL, NULL, 'system:user:delete', 3, FALSE);

-- 角色管理菜单
INSERT INTO menus (id, parent_id, name, menu_type, path, component, icon, permission, sort, is_show)
VALUES (
    'c0000000-0000-0000-0000-000000000102'::UUID,
    'c0000000-0000-0000-0000-000000000100'::UUID,
    '角色管理',
    'menu',
    '/system/role',
    '/views/system/role/index',
    'mdi:account-group',
    'system:role:list',
    2,
    TRUE
);

-- 菜单管理菜单
INSERT INTO menus (id, parent_id, name, menu_type, path, component, icon, permission, sort, is_show)
VALUES (
    'c0000000-0000-0000-0000-000000000103'::UUID,
    'c0000000-0000-0000-0000-000000000100'::UUID,
    '菜单管理',
    'menu',
    '/system/menu',
    '/views/system/menu/index',
    'mdi:menu',
    'system:menu:list',
    3,
    TRUE
);

-- 为超级管理员分配所有菜单权限
INSERT INTO role_menus (role_id, menu_id)
SELECT 
    'a0000000-0000-0000-0000-000000000001'::UUID,
    id
FROM menus;

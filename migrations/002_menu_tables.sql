CREATE TABLE IF NOT EXISTS menus (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID,
    name VARCHAR(50) NOT NULL,
    menu_type VARCHAR(20) NOT NULL,
    path VARCHAR(255),
    component VARCHAR(255),
    icon VARCHAR(100),
    permission VARCHAR(100),
    sort INTEGER NOT NULL DEFAULT 0,
    is_show BOOLEAN NOT NULL DEFAULT TRUE,
    is_cache BOOLEAN NOT NULL DEFAULT FALSE,
    is_external BOOLEAN NOT NULL DEFAULT FALSE,
    status SMALLINT NOT NULL DEFAULT 1,
    created_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_id UUID,
    updated_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_id UUID,
    deleted_time TIMESTAMP,
    deleted_id UUID,
    FOREIGN KEY (parent_id) REFERENCES menus(id) ON DELETE CASCADE,
    CONSTRAINT chk_menu_type CHECK (menu_type IN ('catalog', 'menu', 'button'))
);

CREATE INDEX IF NOT EXISTS idx_menus_parent_id ON menus(parent_id);
CREATE INDEX IF NOT EXISTS idx_menus_status ON menus(status);
CREATE INDEX IF NOT EXISTS idx_menus_sort ON menus(sort);
CREATE INDEX IF NOT EXISTS idx_menus_deleted_time ON menus(deleted_time);

CREATE TABLE IF NOT EXISTS role_menus (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    menu_id UUID NOT NULL REFERENCES menus(id) ON DELETE CASCADE,
    created_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_id UUID,
    UNIQUE(role_id, menu_id)
);

CREATE INDEX IF NOT EXISTS idx_role_menus_role_id ON role_menus(role_id);
CREATE INDEX IF NOT EXISTS idx_role_menus_menu_id ON role_menus(menu_id);

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
)
ON CONFLICT (id) DO NOTHING;

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
)
ON CONFLICT (id) DO NOTHING;

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
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO menus (id, parent_id, name, menu_type, path, component, icon, permission, sort, is_show)
VALUES
    ('c0000000-0000-0000-0000-000000000111'::UUID, 'c0000000-0000-0000-0000-000000000101'::UUID, '新增用户', 'button', NULL, NULL, NULL, 'system:user:add', 1, FALSE),
    ('c0000000-0000-0000-0000-000000000112'::UUID, 'c0000000-0000-0000-0000-000000000101'::UUID, '编辑用户', 'button', NULL, NULL, NULL, 'system:user:edit', 2, FALSE),
    ('c0000000-0000-0000-0000-000000000113'::UUID, 'c0000000-0000-0000-0000-000000000101'::UUID, '删除用户', 'button', NULL, NULL, NULL, 'system:user:delete', 3, FALSE)
ON CONFLICT (id) DO NOTHING;

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
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO menus (id, parent_id, name, menu_type, path, component, icon, permission, sort, is_show)
VALUES
    ('c0000000-0000-0000-0000-000000000121'::UUID, 'c0000000-0000-0000-0000-000000000102'::UUID, '新增角色', 'button', NULL, NULL, NULL, 'system:role:add', 1, FALSE),
    ('c0000000-0000-0000-0000-000000000122'::UUID, 'c0000000-0000-0000-0000-000000000102'::UUID, '编辑角色', 'button', NULL, NULL, NULL, 'system:role:edit', 2, FALSE),
    ('c0000000-0000-0000-0000-000000000123'::UUID, 'c0000000-0000-0000-0000-000000000102'::UUID, '删除角色', 'button', NULL, NULL, NULL, 'system:role:delete', 3, FALSE),
    ('c0000000-0000-0000-0000-000000000124'::UUID, 'c0000000-0000-0000-0000-000000000102'::UUID, '菜单授权', 'button', NULL, NULL, NULL, 'system:role:menu', 4, FALSE),
    ('c0000000-0000-0000-0000-000000000125'::UUID, 'c0000000-0000-0000-0000-000000000102'::UUID, '按钮授权', 'button', NULL, NULL, NULL, 'system:role:button', 5, FALSE)
ON CONFLICT (id) DO NOTHING;

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
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO menus (id, parent_id, name, menu_type, path, component, icon, permission, sort, is_show)
VALUES
    ('c0000000-0000-0000-0000-000000000131'::UUID, 'c0000000-0000-0000-0000-000000000103'::UUID, '新增菜单', 'button', NULL, NULL, NULL, 'system:menu:add', 1, FALSE),
    ('c0000000-0000-0000-0000-000000000132'::UUID, 'c0000000-0000-0000-0000-000000000103'::UUID, '编辑菜单', 'button', NULL, NULL, NULL, 'system:menu:edit', 2, FALSE),
    ('c0000000-0000-0000-0000-000000000133'::UUID, 'c0000000-0000-0000-0000-000000000103'::UUID, '删除菜单', 'button', NULL, NULL, NULL, 'system:menu:delete', 3, FALSE)
ON CONFLICT (id) DO NOTHING;

INSERT INTO role_menus (role_id, menu_id)
SELECT
    'a0000000-0000-0000-0000-000000000001'::UUID,
    id
FROM menus
ON CONFLICT (role_id, menu_id) DO NOTHING;

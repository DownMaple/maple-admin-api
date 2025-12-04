#!/bin/bash

echo "正在创建 PostgreSQL 数据库..."

# 创建数据库
docker exec -it postgres psql -U postgres -c "CREATE DATABASE maple_admin;" 2>/dev/null || echo "数据库可能已存在"

echo "数据库创建完成！"
echo "数据库连接信息："
echo "  Host: localhost"
echo "  Port: 5432"
echo "  Database: maple_admin"
echo "  User: postgres"
echo "  Password: postgres"

#!/bin/bash
# 手动运行数据库迁移

set -e

export PGPASSWORD=secret
DB_HOST=localhost
DB_PORT=5433
DB_USER=postgres
DB_NAME=aiw3_monitor

echo "🔧 手动运行数据库迁移..."

# 创建数据库（如果不存在）
echo "📦 创建数据库..."
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -tc "SELECT 1 FROM pg_database WHERE datname = '$DB_NAME'" | grep -q 1 || \
    psql -h $DB_HOST -p $DB_PORT -U $DB_USER -c "CREATE DATABASE $DB_NAME"

# 启用 TimescaleDB 扩展
echo "🔌 启用 TimescaleDB 扩展..."
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;"

# 运行迁移文件
echo "🔄 运行迁移文件..."
for migration in migrations/*.sql; do
    echo "  - 运行 $migration"
    psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -f "$migration" || {
        echo "⚠️  迁移 $migration 失败，继续..."
    }
done

echo "✅ 数据库迁移完成！"


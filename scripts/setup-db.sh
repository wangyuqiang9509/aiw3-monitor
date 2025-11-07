#!/bin/bash
# 数据库初始化脚本

set -e

echo "🔧 Setting up AIWS Monitor database..."

# 默认配置
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-aiw3_monitor}"
DB_USER="${DB_USER:-postgres}"
DB_PASSWORD="${DB_PASSWORD:-secret}"

# 检查 PostgreSQL 是否运行
echo "Checking PostgreSQL connection..."
until PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -c '\q' 2>/dev/null; do
  echo "Waiting for PostgreSQL to be ready..."
  sleep 2
done

echo "✅ PostgreSQL is ready"

# 创建数据库(如果不存在)
echo "Creating database if not exists..."
PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -tc "SELECT 1 FROM pg_database WHERE datname = '$DB_NAME'" | grep -q 1 || \
PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -c "CREATE DATABASE $DB_NAME"

echo "✅ Database ready"

# 运行迁移
echo "Running migrations..."
export DATABASE_URL="postgres://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME"

cd backend

# 使用 sqlx-cli 运行迁移
if command -v sqlx &> /dev/null; then
    sqlx migrate run
else
    echo "⚠️  sqlx-cli not found. Installing..."
    cargo install sqlx-cli --no-default-features --features postgres
    sqlx migrate run
fi

echo "✅ Migrations completed"

echo "🎉 Database setup complete!"
echo ""
echo "Connection details:"
echo "  Host: $DB_HOST"
echo "  Port: $DB_PORT"
echo "  Database: $DB_NAME"
echo "  User: $DB_USER"
echo ""
echo "Next steps:"
echo "  1. Copy config.example.toml to config.toml"
echo "  2. Update config.toml with your settings"
echo "  3. Run: cargo run"


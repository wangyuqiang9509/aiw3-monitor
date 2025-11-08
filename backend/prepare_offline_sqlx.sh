#!/bin/bash
# 准备 SQLX 离线模式数据

set -e

echo "🔧 准备 SQLX 离线模式..."

# 检查 Docker 是否运行
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker 未运行，请先启动 Docker"
    exit 1
fi

# 启动数据库（如果未运行）
echo "📦 检查数据库容器..."
if ! docker ps | grep -q aiw3-postgres; then
    echo "🚀 启动数据库容器..."
    cd ../deployment/docker
    docker-compose up -d postgres
    echo "⏳ 等待数据库就绪..."
    sleep 15
    cd ../../backend
fi

# 设置数据库 URL
export DATABASE_URL="postgresql://postgres:secret@localhost:5432/aiw3_monitor"

# 检查 sqlx-cli 是否安装
if ! command -v sqlx &> /dev/null; then
    echo "📦 安装 sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

# 运行迁移
echo "🔄 运行数据库迁移..."
sqlx database create 2>/dev/null || true
sqlx migrate run

# 生成离线数据
echo "📝 生成 SQLX 离线数据..."
cargo sqlx prepare

echo "✅ SQLX 离线数据准备完成！"
echo "现在可以使用 SQLX_OFFLINE=true cargo build 进行编译"

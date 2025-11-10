#!/bin/bash
# AIWS 监控系统一键启动脚本
# 用途：启动所有监控服务（Docker 容器 + Rust 应用）

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 项目根目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}    AIWS 区块链监控系统 - 一键启动${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# 检查 Docker 是否运行
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}✗ Docker 未运行，请先启动 Docker${NC}"
    exit 1
fi

# 1. 启动 Docker 容器
echo -e "${BLUE}[1/4]${NC} 启动 Docker 容器..."
cd deployment/docker

# 检查容器是否已经运行
if docker-compose ps | grep -q "Up"; then
    echo -e "${YELLOW}  容器已在运行，重启以应用新配置...${NC}"
    docker-compose restart prometheus
else
    echo -e "${YELLOW}  启动所有容器...${NC}"
    docker-compose up -d
fi

# 等待服务就绪
echo -e "${YELLOW}  等待服务启动...${NC}"
sleep 5

# 检查容器状态
POSTGRES_STATUS=$(docker inspect -f '{{.State.Health.Status}}' aiw3-postgres 2>/dev/null || echo "not_found")
if [ "$POSTGRES_STATUS" = "healthy" ]; then
    echo -e "${GREEN}  ✓ PostgreSQL: 运行中${NC}"
else
    echo -e "${YELLOW}  ⚠ PostgreSQL: 启动中...${NC}"
    # 等待 PostgreSQL 健康
    for i in {1..30}; do
        POSTGRES_STATUS=$(docker inspect -f '{{.State.Health.Status}}' aiw3-postgres 2>/dev/null || echo "not_found")
        if [ "$POSTGRES_STATUS" = "healthy" ]; then
            echo -e "${GREEN}  ✓ PostgreSQL: 就绪${NC}"
            break
        fi
        sleep 1
    done
fi

if docker ps | grep -q aiw3-prometheus; then
    echo -e "${GREEN}  ✓ Prometheus: 运行中${NC}"
else
    echo -e "${RED}  ✗ Prometheus: 启动失败${NC}"
fi

if docker ps | grep -q aiw3-grafana; then
    echo -e "${GREEN}  ✓ Grafana: 运行中${NC}"
else
    echo -e "${RED}  ✗ Grafana: 启动失败${NC}"
fi

if docker ps | grep -q aiw3-alertmanager; then
    echo -e "${GREEN}  ✓ AlertManager: 运行中${NC}"
else
    echo -e "${RED}  ✗ AlertManager: 启动失败${NC}"
fi

cd "$SCRIPT_DIR"

# 2. 检查 Rust 应用是否已编译
echo ""
echo -e "${BLUE}[2/4]${NC} 检查 Rust 应用..."
if [ ! -f "backend/target/release/aiw3_monitor_bin" ]; then
    echo -e "${YELLOW}  应用未编译，正在编译...${NC}"
    cd backend
    cargo build --release
    cd "$SCRIPT_DIR"
    echo -e "${GREEN}  ✓ 编译完成${NC}"
else
    echo -e "${GREEN}  ✓ 应用已编译${NC}"
fi

# 3. 启动 Rust 监控应用
echo ""
echo -e "${BLUE}[3/4]${NC} 启动监控应用..."

# 检查是否已经运行
if pgrep -f "aiw3_monitor_bin" > /dev/null; then
    echo -e "${YELLOW}  监控应用已在运行，重启...${NC}"
    pkill -f "aiw3_monitor_bin"
    sleep 2
fi

# 启动应用
cd backend
RUST_LOG=info nohup ./target/release/aiw3_monitor_bin > /tmp/aiw3-monitor.log 2>&1 &
MONITOR_PID=$!
cd "$SCRIPT_DIR"

# 等待应用启动
echo -e "${YELLOW}  等待应用启动...${NC}"
for i in {1..15}; do
    if curl -s http://localhost:9090/metrics > /dev/null 2>&1; then
        echo -e "${GREEN}  ✓ 监控应用启动成功 (PID: $MONITOR_PID)${NC}"
        break
    fi
    if [ $i -eq 15 ]; then
        echo -e "${RED}  ✗ 监控应用启动超时${NC}"
        echo -e "${YELLOW}  查看日志: tail -f /tmp/aiw3-monitor.log${NC}"
        exit 1
    fi
    sleep 1
done

# 4. 验证所有服务
echo ""
echo -e "${BLUE}[4/4]${NC} 验证服务状态..."

# 检查 Rust 应用
if curl -s http://localhost:9090/metrics | grep -q "aiw3_chain_block_height"; then
    BLOCK_HEIGHT=$(curl -s http://localhost:9090/metrics | grep "^aiw3_chain_block_height" | head -1 | awk '{print $2}')
    echo -e "${GREEN}  ✓ Rust 应用: 正常 (区块高度: $BLOCK_HEIGHT)${NC}"
else
    echo -e "${RED}  ✗ Rust 应用: 指标端点异常${NC}"
fi

# 检查 Prometheus
if curl -s http://localhost:9091/api/v1/targets | grep -q "aiw3-monitor"; then
    echo -e "${GREEN}  ✓ Prometheus: 正常${NC}"
else
    echo -e "${YELLOW}  ⚠ Prometheus: 可能未抓取到数据${NC}"
fi

# 检查 Grafana
if curl -s http://localhost:3000/api/health | grep -q "ok"; then
    echo -e "${GREEN}  ✓ Grafana: 正常${NC}"
else
    echo -e "${YELLOW}  ⚠ Grafana: 响应异常${NC}"
fi

# 显示访问信息
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✓ 所有服务已启动！${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${BLUE}📊 访问地址：${NC}"
echo -e "  ${YELLOW}Grafana 仪表板:${NC}    http://localhost:3000"
echo -e "                      用户名: admin"
echo -e "                      密码: admin"
echo ""
echo -e "  ${YELLOW}Prometheus UI:${NC}     http://localhost:9091"
echo -e "  ${YELLOW}AlertManager:${NC}      http://localhost:9093"
echo -e "  ${YELLOW}监控指标端点:${NC}      http://localhost:9090/metrics"
echo ""
echo -e "${BLUE}📝 日志文件：${NC}"
echo -e "  ${YELLOW}监控应用:${NC}          tail -f /tmp/aiw3-monitor.log"
echo -e "  ${YELLOW}PostgreSQL:${NC}        docker logs -f aiw3-postgres"
echo -e "  ${YELLOW}Prometheus:${NC}        docker logs -f aiw3-prometheus"
echo -e "  ${YELLOW}Grafana:${NC}           docker logs -f aiw3-grafana"
echo ""
echo -e "${BLUE}🛑 停止服务：${NC}"
echo -e "  ${YELLOW}./stop-all-services.sh${NC}"
echo ""
echo -e "${BLUE}💡 提示：${NC}"
echo -e "  - 首次登录 Grafana 需要修改密码（可以跳过）"
echo -e "  - 数据每 30 秒采集一次"
echo -e "  - Prometheus 每 30 秒抓取一次指标"
echo ""

# 自动打开 Grafana（可选）
read -p "是否在浏览器中打开 Grafana？(y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if command -v xdg-open > /dev/null 2>&1; then
        xdg-open "http://localhost:3000" 2>/dev/null &
    elif command -v open > /dev/null 2>&1; then
        open "http://localhost:3000" 2>/dev/null &
    else
        echo -e "${YELLOW}无法自动打开浏览器，请手动访问 http://localhost:3000${NC}"
    fi
fi

echo -e "${GREEN}🎉 启动完成！${NC}"


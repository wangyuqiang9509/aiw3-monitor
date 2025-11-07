#!/bin/bash
# AIWS 监控可视化快速启动脚本

set -e

echo "🚀 启动 AIWS 监控可视化系统..."
echo ""

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 项目根目录
PROJECT_ROOT="/home/yukeen/aiw3-monitor"
cd "$PROJECT_ROOT"

# 1. 检查并启动数据库
echo -e "${BLUE}[1/3]${NC} 检查数据库..."
if ! docker ps | grep -q aiw3-postgres; then
    echo -e "${YELLOW}数据库未运行，正在启动...${NC}"
    docker start aiw3-postgres 2>/dev/null || {
        echo -e "${YELLOW}数据库容器不存在，正在创建...${NC}"
        docker run -d \
            --name aiw3-postgres \
            -p 5433:5432 \
            -e POSTGRES_PASSWORD=secret \
            -e POSTGRES_DB=aiw3_monitor \
            timescale/timescaledb:latest-pg15
        echo "等待数据库启动..."
        sleep 5
    }
fi
echo -e "${GREEN}✓${NC} 数据库运行中"

# 2. 检查并启动监控应用
echo -e "${BLUE}[2/3]${NC} 检查监控应用..."
if ! curl -s http://localhost:9090/health > /dev/null 2>&1; then
    echo -e "${YELLOW}监控应用未运行，正在启动...${NC}"
    
    # 停止旧进程（如果存在）
    pkill -f aiw3_monitor_bin 2>/dev/null || true
    sleep 1
    
    # 启动监控应用
    cd "$PROJECT_ROOT/backend"
    RUST_LOG=info ./target/release/aiw3_monitor_bin > /tmp/aiw3-monitor.log 2>&1 &
    
    # 等待启动
    echo "等待监控应用启动..."
    for i in {1..10}; do
        if curl -s http://localhost:9090/health > /dev/null 2>&1; then
            break
        fi
        sleep 1
    done
fi

if curl -s http://localhost:9090/health > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} 监控应用运行中"
else
    echo -e "${YELLOW}⚠${NC} 监控应用启动失败，请检查日志: tail -f /tmp/aiw3-monitor.log"
    exit 1
fi

# 3. 检查并启动 Web 服务器
echo -e "${BLUE}[3/3]${NC} 检查 Web 服务器..."
if ! curl -s http://localhost:8080/ > /dev/null 2>&1; then
    echo -e "${YELLOW}Web 服务器未运行，正在启动...${NC}"
    
    # 停止旧进程（如果存在）
    pkill -f "python3 -m http.server 8080" 2>/dev/null || true
    sleep 1
    
    # 启动 Web 服务器
    cd "$PROJECT_ROOT"
    python3 -m http.server 8080 > /tmp/http-server.log 2>&1 &
    sleep 2
fi

if curl -s http://localhost:8080/ > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Web 服务器运行中"
else
    echo -e "${YELLOW}⚠${NC} Web 服务器启动失败"
    exit 1
fi

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✓ 所有服务已启动！${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "📊 ${BLUE}可视化面板:${NC} http://localhost:8080/visualization.html"
echo -e "📈 ${BLUE}Prometheus 指标:${NC} http://localhost:9090/metrics"
echo -e "💚 ${BLUE}健康检查:${NC} http://localhost:9090/health"
echo ""
echo -e "📝 ${BLUE}查看日志:${NC}"
echo -e "   监控应用: tail -f /tmp/aiw3-monitor.log"
echo -e "   Web 服务器: tail -f /tmp/http-server.log"
echo ""
echo -e "🛑 ${BLUE}停止服务:${NC}"
echo -e "   $PROJECT_ROOT/stop-visualization.sh"
echo ""
echo -e "${YELLOW}提示: 可视化面板每 5 秒自动刷新数据${NC}"
echo ""

# 显示当前指标
echo -e "${BLUE}当前监控数据:${NC}"
curl -s http://localhost:9090/metrics | grep -E "^aiw3_chain_(block_height|node_count|tx_pool_size|sync_status)" | while read line; do
    echo "  $line"
done
echo ""

# 自动打开浏览器（可选）
if command -v xdg-open > /dev/null 2>&1; then
    echo -e "${YELLOW}正在打开浏览器...${NC}"
    xdg-open "http://localhost:8080/visualization.html" 2>/dev/null &
elif command -v open > /dev/null 2>&1; then
    echo -e "${YELLOW}正在打开浏览器...${NC}"
    open "http://localhost:8080/visualization.html" 2>/dev/null &
fi

echo -e "${GREEN}🎉 启动完成！${NC}"


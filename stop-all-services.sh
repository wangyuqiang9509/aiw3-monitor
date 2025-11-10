#!/bin/bash
# AIWS 监控系统停止脚本
# 用途：优雅停止所有监控服务

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
echo -e "${BLUE}    AIWS 区块链监控系统 - 停止服务${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# 1. 停止 Rust 监控应用
echo -e "${BLUE}[1/2]${NC} 停止监控应用..."
if pgrep -f "aiw3_monitor_bin" > /dev/null; then
    MONITOR_PID=$(pgrep -f "aiw3_monitor_bin")
    echo -e "${YELLOW}  正在停止监控应用 (PID: $MONITOR_PID)...${NC}"
    pkill -TERM -f "aiw3_monitor_bin"
    
    # 等待进程优雅退出
    for i in {1..10}; do
        if ! pgrep -f "aiw3_monitor_bin" > /dev/null; then
            echo -e "${GREEN}  ✓ 监控应用已停止${NC}"
            break
        fi
        sleep 1
    done
    
    # 如果还在运行，强制停止
    if pgrep -f "aiw3_monitor_bin" > /dev/null; then
        echo -e "${YELLOW}  强制停止监控应用...${NC}"
        pkill -KILL -f "aiw3_monitor_bin"
        echo -e "${GREEN}  ✓ 监控应用已强制停止${NC}"
    fi
else
    echo -e "${YELLOW}  监控应用未运行${NC}"
fi

# 2. 停止 Docker 容器
echo ""
echo -e "${BLUE}[2/2]${NC} 停止 Docker 容器..."

cd deployment/docker

# 检查是否有容器在运行
if docker-compose ps | grep -q "Up"; then
    echo -e "${YELLOW}  正在停止容器...${NC}"
    docker-compose stop
    
    # 检查各个容器状态
    if ! docker ps | grep -q aiw3-postgres; then
        echo -e "${GREEN}  ✓ PostgreSQL: 已停止${NC}"
    fi
    
    if ! docker ps | grep -q aiw3-prometheus; then
        echo -e "${GREEN}  ✓ Prometheus: 已停止${NC}"
    fi
    
    if ! docker ps | grep -q aiw3-grafana; then
        echo -e "${GREEN}  ✓ Grafana: 已停止${NC}"
    fi
    
    if ! docker ps | grep -q aiw3-alertmanager; then
        echo -e "${GREEN}  ✓ AlertManager: 已停止${NC}"
    fi
else
    echo -e "${YELLOW}  容器未运行${NC}"
fi

cd "$SCRIPT_DIR"

# 显示停止信息
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✓ 所有服务已停止！${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${BLUE}💡 提示：${NC}"
echo -e "  - 容器已停止但未删除，数据已保留"
echo -e "  - 重新启动: ${YELLOW}./start-all-services.sh${NC}"
echo -e "  - 完全清理: ${YELLOW}cd deployment/docker && docker-compose down -v${NC}"
echo ""
echo -e "${BLUE}📝 日志文件保留在：${NC}"
echo -e "  ${YELLOW}/tmp/aiw3-monitor.log${NC}"
echo ""


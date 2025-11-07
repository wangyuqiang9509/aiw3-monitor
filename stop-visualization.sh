#!/bin/bash
# AIWS 监控可视化停止脚本

set -e

echo "🛑 停止 AIWS 监控可视化系统..."
echo ""

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 1. 停止 Web 服务器
echo -e "${BLUE}[1/3]${NC} 停止 Web 服务器..."
if pkill -f "python3 -m http.server 8080" 2>/dev/null; then
    echo -e "${GREEN}✓${NC} Web 服务器已停止"
else
    echo -e "${YELLOW}⚠${NC} Web 服务器未运行"
fi

# 2. 停止监控应用
echo -e "${BLUE}[2/3]${NC} 停止监控应用..."
if pkill -f aiw3_monitor_bin 2>/dev/null; then
    echo -e "${GREEN}✓${NC} 监控应用已停止"
else
    echo -e "${YELLOW}⚠${NC} 监控应用未运行"
fi

# 3. 停止数据库（可选）
echo -e "${BLUE}[3/3]${NC} 停止数据库..."
read -p "是否停止数据库容器？(y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if docker stop aiw3-postgres 2>/dev/null; then
        echo -e "${GREEN}✓${NC} 数据库已停止"
    else
        echo -e "${YELLOW}⚠${NC} 数据库未运行"
    fi
else
    echo -e "${BLUE}ℹ${NC} 数据库保持运行"
fi

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✓ 服务已停止！${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${BLUE}重新启动:${NC} /home/yukeen/aiw3-monitor/start-visualization.sh"
echo ""


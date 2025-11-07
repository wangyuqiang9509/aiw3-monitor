#!/bin/bash
# 邮件发送功能测试脚本
# 用于验证 SMTP 配置和邮件发送功能

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}AIWS Monitor - 邮件发送测试${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# 检查配置文件
echo -e "${YELLOW}[1/3] 检查 SMTP 配置...${NC}"

if [ ! -f "backend/config.toml" ]; then
    echo -e "${RED}✗ 配置文件不存在: backend/config.toml${NC}"
    echo -e "${YELLOW}提示: 请复制 config.example.toml 并配置 SMTP 设置${NC}"
    exit 1
fi

# 提取 SMTP 配置
SMTP_SERVER=$(grep -A 10 '\[smtp\]' backend/config.toml | grep 'server' | cut -d '=' -f2 | tr -d ' "')
SMTP_PORT=$(grep -A 10 '\[smtp\]' backend/config.toml | grep 'port' | cut -d '=' -f2 | tr -d ' ')
SMTP_FROM=$(grep -A 10 '\[smtp\]' backend/config.toml | grep 'from' | cut -d '=' -f2 | tr -d ' "')

if [ -z "$SMTP_SERVER" ]; then
    echo -e "${RED}✗ SMTP 服务器未配置${NC}"
    exit 1
fi

echo -e "${GREEN}✓ SMTP 配置已找到${NC}"
echo "  - 服务器: $SMTP_SERVER"
echo "  - 端口: $SMTP_PORT"
echo "  - 发件人: $SMTP_FROM"
echo ""

# 测试 SMTP 连接
echo -e "${YELLOW}[2/3] 测试 SMTP 服务器连接...${NC}"

if timeout 5 bash -c "echo > /dev/tcp/$SMTP_SERVER/$SMTP_PORT" 2>/dev/null; then
    echo -e "${GREEN}✓ SMTP 服务器可达${NC}"
else
    echo -e "${RED}✗ 无法连接到 SMTP 服务器${NC}"
    echo -e "${YELLOW}提示: 请检查网络连接和防火墙设置${NC}"
    exit 1
fi
echo ""

# 创建测试告警事件
echo -e "${YELLOW}[3/3] 创建测试告警事件...${NC}"

# 检查数据库连接
if ! psql -h localhost -U postgres -d aiw3_monitor -c "SELECT 1" > /dev/null 2>&1; then
    echo -e "${RED}✗ 数据库连接失败${NC}"
    exit 1
fi

# 创建或获取测试节点
NODE_ID=$(psql -h localhost -U postgres -d aiw3_monitor -t -c "
SELECT id FROM blockchain_nodes WHERE name = 'Test Node' LIMIT 1;
" | xargs)

if [ -z "$NODE_ID" ]; then
    NODE_ID=$(psql -h localhost -U postgres -d aiw3_monitor -t -c "
    INSERT INTO blockchain_nodes (name, rpc_url, environment, enabled) 
    VALUES ('Test Node', 'http://localhost:26657', 'test', true) 
    RETURNING id;
    " | xargs)
fi

# 创建或获取测试告警规则
RULE_ID=$(psql -h localhost -U postgres -d aiw3_monitor -t -c "
SELECT id FROM alert_rules WHERE name = 'Test Email Alert' LIMIT 1;
" | xargs)

if [ -z "$RULE_ID" ]; then
    RULE_ID=$(psql -h localhost -U postgres -d aiw3_monitor -t -c "
    INSERT INTO alert_rules (
        name, node_id, metric_name, condition_type, 
        threshold_value, comparison_operator, severity, 
        email_recipients, enabled
    ) 
    VALUES (
        'Test Email Alert', 
        $NODE_ID, 
        'test_metric', 
        'threshold', 
        10.0, 
        '>', 
        'warning', 
        ARRAY['test@example.com'], 
        true
    )
    RETURNING id;
    " | xargs)
fi

# 创建测试告警事件
EVENT_ID=$(psql -h localhost -U postgres -d aiw3_monitor -t -c "
INSERT INTO alert_events (
    rule_id, node_id, triggered_at, trigger_value, 
    status, notification_sent
) 
VALUES (
    $RULE_ID, 
    $NODE_ID, 
    NOW(), 
    15.0, 
    'active', 
    false
)
RETURNING id;
" | xargs)

echo -e "${GREEN}✓ 测试告警事件已创建 (ID: $EVENT_ID)${NC}"
echo ""

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}准备就绪！${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

echo -e "${YELLOW}下一步操作：${NC}"
echo "1. 确保 AIWS Monitor 服务正在运行"
echo "2. 告警调度器会在下次检查时发送测试邮件"
echo "3. 或者手动触发告警检查（如果实现了 API 端点）"
echo ""

echo -e "${YELLOW}检查邮件发送状态：${NC}"
echo "psql -d aiw3_monitor -c \"SELECT id, notification_sent, notification_error FROM alert_events WHERE id = $EVENT_ID;\""
echo ""

# 清理选项
read -p "是否清理测试数据? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}清理测试数据...${NC}"
    psql -h localhost -U postgres -d aiw3_monitor -c "
    DELETE FROM alert_events WHERE id = $EVENT_ID;
    " > /dev/null
    echo -e "${GREEN}✓ 测试数据已清理${NC}"
fi


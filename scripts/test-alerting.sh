#!/bin/bash
# 告警功能测试脚本
# 用于验证告警规则创建、触发和邮件发送功能

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}AIWS Monitor - 告警功能测试${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# 检查数据库连接
echo -e "${YELLOW}[1/5] 检查数据库连接...${NC}"
if psql -h localhost -U postgres -d aiw3_monitor -c "SELECT 1" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ 数据库连接成功${NC}"
else
    echo -e "${RED}✗ 数据库连接失败${NC}"
    exit 1
fi
echo ""

# 创建测试节点
echo -e "${YELLOW}[2/5] 创建测试节点...${NC}"
NODE_ID=$(psql -h localhost -U postgres -d aiw3_monitor -t -c "
INSERT INTO blockchain_nodes (name, rpc_url, environment, enabled) 
VALUES ('Test Node', 'http://localhost:26657', 'test', true) 
ON CONFLICT (name) DO UPDATE SET enabled = true
RETURNING id;
" | xargs)

if [ -n "$NODE_ID" ]; then
    echo -e "${GREEN}✓ 测试节点已创建 (ID: $NODE_ID)${NC}"
else
    echo -e "${RED}✗ 创建测试节点失败${NC}"
    exit 1
fi
echo ""

# 创建告警规则
echo -e "${YELLOW}[3/5] 创建告警规则...${NC}"
RULE_ID=$(psql -h localhost -U postgres -d aiw3_monitor -t -c "
INSERT INTO alert_rules (
    name, node_id, metric_name, condition_type, 
    threshold_value, comparison_operator, severity, 
    email_recipients, silence_period_seconds, enabled
) 
VALUES (
    'Test Alert - Low Peer Count', 
    $NODE_ID, 
    'peer_count', 
    'threshold', 
    2.0, 
    '<', 
    'warning', 
    ARRAY['test@example.com'], 
    300, 
    true
)
ON CONFLICT DO NOTHING
RETURNING id;
" | xargs)

if [ -n "$RULE_ID" ]; then
    echo -e "${GREEN}✓ 告警规则已创建 (ID: $RULE_ID)${NC}"
else
    # 尝试获取已存在的规则
    RULE_ID=$(psql -h localhost -U postgres -d aiw3_monitor -t -c "
    SELECT id FROM alert_rules WHERE name = 'Test Alert - Low Peer Count' LIMIT 1;
    " | xargs)
    echo -e "${GREEN}✓ 使用已存在的告警规则 (ID: $RULE_ID)${NC}"
fi
echo ""

# 插入触发告警的指标数据
echo -e "${YELLOW}[4/5] 插入触发告警的指标数据...${NC}"
psql -h localhost -U postgres -d aiw3_monitor -c "
INSERT INTO metric_data (node_id, metric_name, value, collected_at) 
VALUES ($NODE_ID, 'peer_count', 1.0, NOW());
" > /dev/null

echo -e "${GREEN}✓ 指标数据已插入 (peer_count = 1, 应触发告警)${NC}"
echo ""

# 检查告警规则
echo -e "${YELLOW}[5/5] 检查告警规则和事件...${NC}"

# 查询告警规则
RULE_COUNT=$(psql -h localhost -U postgres -d aiw3_monitor -t -c "
SELECT COUNT(*) FROM alert_rules WHERE enabled = true;
" | xargs)

echo -e "  - 活跃告警规则数: ${GREEN}$RULE_COUNT${NC}"

# 查询最近的指标数据
RECENT_METRIC=$(psql -h localhost -U postgres -d aiw3_monitor -t -c "
SELECT metric_name, value, collected_at 
FROM metric_data 
WHERE node_id = $NODE_ID 
ORDER BY collected_at DESC 
LIMIT 1;
")

echo -e "  - 最近的指标数据: ${GREEN}$RECENT_METRIC${NC}"

# 查询告警事件
EVENT_COUNT=$(psql -h localhost -U postgres -d aiw3_monitor -t -c "
SELECT COUNT(*) FROM alert_events WHERE rule_id = $RULE_ID;
" | xargs)

echo -e "  - 告警事件数: ${GREEN}$EVENT_COUNT${NC}"

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}测试完成！${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

echo -e "${YELLOW}注意事项：${NC}"
echo "1. 告警引擎需要在主程序中运行才能自动评估规则"
echo "2. 启动服务后，告警调度器会每 60 秒检查一次规则"
echo "3. 如果条件满足，会创建告警事件并发送邮件通知"
echo ""

echo -e "${YELLOW}下一步操作：${NC}"
echo "1. 启动 AIWS Monitor 服务: cargo run --release"
echo "2. 查看日志: tail -f logs/aiw3-monitor.log"
echo "3. 检查告警事件: psql -d aiw3_monitor -c 'SELECT * FROM alert_events ORDER BY triggered_at DESC LIMIT 10;'"
echo ""

# 清理选项
read -p "是否清理测试数据? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}清理测试数据...${NC}"
    psql -h localhost -U postgres -d aiw3_monitor -c "
    DELETE FROM alert_events WHERE rule_id = $RULE_ID;
    DELETE FROM alert_rules WHERE id = $RULE_ID;
    DELETE FROM metric_data WHERE node_id = $NODE_ID;
    DELETE FROM blockchain_nodes WHERE id = $NODE_ID;
    " > /dev/null
    echo -e "${GREEN}✓ 测试数据已清理${NC}"
fi


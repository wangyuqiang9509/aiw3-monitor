-- 创建告警级别枚举
CREATE TYPE alert_severity AS ENUM ('critical', 'warning', 'info');

-- 创建告警规则表
CREATE TABLE IF NOT EXISTS alert_rules (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    node_id INTEGER REFERENCES blockchain_nodes(id) ON DELETE CASCADE,
    metric_type metric_type NOT NULL,
    condition_type VARCHAR(50) NOT NULL, -- 'threshold', 'no_change', 'rate_of_change'
    threshold_value DOUBLE PRECISION,
    time_window_seconds INTEGER NOT NULL DEFAULT 300,
    severity alert_severity NOT NULL DEFAULT 'warning',
    enabled BOOLEAN NOT NULL DEFAULT true,
    silence_period_seconds INTEGER NOT NULL DEFAULT 1800,
    notification_channels JSONB DEFAULT '["email"]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX idx_alert_rules_node_id ON alert_rules(node_id);
CREATE INDEX idx_alert_rules_metric_type ON alert_rules(metric_type);
CREATE INDEX idx_alert_rules_enabled ON alert_rules(enabled) WHERE enabled = true;

-- 创建更新时间触发器
CREATE TRIGGER update_alert_rules_updated_at BEFORE UPDATE ON alert_rules
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 插入示例告警规则
INSERT INTO alert_rules (
    name, description, metric_type, condition_type, 
    threshold_value, time_window_seconds, severity
) VALUES 
(
    'Block Height No Change',
    '区块高度10分钟无变化',
    'blockheight',
    'no_change',
    NULL,
    600,
    'critical'
),
(
    'Low Node Count',
    '节点数量低于阈值',
    'nodecount',
    'threshold',
    3.0,
    300,
    'warning'
),
(
    'High Transaction Pool',
    '交易池积压超过限制',
    'txpoolsize',
    'threshold',
    1000.0,
    300,
    'warning'
);

-- 添加注释
COMMENT ON TABLE alert_rules IS '告警规则配置表';
COMMENT ON COLUMN alert_rules.condition_type IS '条件类型: threshold(阈值), no_change(无变化), rate_of_change(变化率)';
COMMENT ON COLUMN alert_rules.silence_period_seconds IS '告警静默期(秒),避免重复告警';


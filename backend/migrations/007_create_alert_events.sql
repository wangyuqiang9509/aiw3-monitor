-- 创建告警状态枚举
CREATE TYPE alert_status AS ENUM ('triggered', 'resolved', 'silenced');

-- 创建告警事件表
CREATE TABLE IF NOT EXISTS alert_events (
    id BIGSERIAL PRIMARY KEY,
    rule_id INTEGER NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    node_id INTEGER REFERENCES blockchain_nodes(id) ON DELETE SET NULL,
    status alert_status NOT NULL DEFAULT 'triggered',
    severity alert_severity NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    metric_value DOUBLE PRECISION,
    threshold_value DOUBLE PRECISION,
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    silenced_until TIMESTAMPTZ,
    notification_sent BOOLEAN NOT NULL DEFAULT false,
    notification_sent_at TIMESTAMPTZ,
    notes TEXT,
    metadata JSONB DEFAULT '{}'::jsonb
);

-- 创建索引
CREATE INDEX idx_alert_events_rule_id ON alert_events(rule_id);
CREATE INDEX idx_alert_events_node_id ON alert_events(node_id);
CREATE INDEX idx_alert_events_status ON alert_events(status);
CREATE INDEX idx_alert_events_triggered_at ON alert_events(triggered_at DESC);
CREATE INDEX idx_alert_events_severity ON alert_events(severity);

-- 创建分区(按时间分区,提高查询性能)
-- 注意: PostgreSQL 10+ 支持声明式分区
-- 这里我们使用简单的索引,如果数据量大可以考虑分区

-- 创建告警统计视图
CREATE OR REPLACE VIEW alert_statistics AS
SELECT
    rule_id,
    r.name AS rule_name,
    r.severity,
    COUNT(*) AS total_alerts,
    COUNT(*) FILTER (WHERE status = 'triggered') AS active_alerts,
    COUNT(*) FILTER (WHERE status = 'resolved') AS resolved_alerts,
    AVG(EXTRACT(EPOCH FROM (resolved_at - triggered_at))) FILTER (WHERE resolved_at IS NOT NULL) AS avg_resolution_time_seconds,
    MAX(triggered_at) AS last_triggered_at
FROM alert_events ae
JOIN alert_rules r ON ae.rule_id = r.id
GROUP BY rule_id, r.name, r.severity;

-- 添加注释
COMMENT ON TABLE alert_events IS '告警事件记录表';
COMMENT ON COLUMN alert_events.status IS '告警状态: triggered(已触发), resolved(已恢复), silenced(已静默)';
COMMENT ON VIEW alert_statistics IS '告警统计视图,用于分析告警频率和响应时间';


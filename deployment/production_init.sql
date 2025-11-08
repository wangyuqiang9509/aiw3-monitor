-- ============================================================================
-- AIW3 监控系统 - 生产环境数据库初始化脚本
-- 版本: 1.0.0
-- 生成时间: 2025-11-08
-- 说明: 此脚本包含所有必要的表结构、索引、触发器和初始数据
-- ============================================================================

-- 设置客户端编码
SET client_encoding = 'UTF8';

-- ============================================================================
-- 1. 创建区块链节点表
-- ============================================================================

CREATE TABLE IF NOT EXISTS blockchain_nodes (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    rpc_url VARCHAR(512) NOT NULL,
    rest_url VARCHAR(512),
    grpc_url VARCHAR(512),
    environment VARCHAR(50) NOT NULL DEFAULT 'mainnet',
    enabled BOOLEAN NOT NULL DEFAULT true,
    labels JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_blockchain_nodes_enabled ON blockchain_nodes(enabled);
CREATE INDEX IF NOT EXISTS idx_blockchain_nodes_environment ON blockchain_nodes(environment);
CREATE INDEX IF NOT EXISTS idx_blockchain_nodes_labels ON blockchain_nodes USING GIN(labels);

-- 创建更新时间触发器函数
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 创建触发器
DROP TRIGGER IF EXISTS update_blockchain_nodes_updated_at ON blockchain_nodes;
CREATE TRIGGER update_blockchain_nodes_updated_at
    BEFORE UPDATE ON blockchain_nodes
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- 插入示例数据（可选，生产环境可删除）
INSERT INTO blockchain_nodes (name, rpc_url, rest_url, grpc_url, environment, enabled, labels)
VALUES 
    ('AIW3-Mainnet-Node-1', 'http://localhost:26657', 'http://localhost:1317', 'http://localhost:9090', 'mainnet', true, '{"region": "us-east-1", "provider": "aws"}'),
    ('AIW3-Testnet-Node-1', 'http://localhost:26658', 'http://localhost:1318', 'http://localhost:9091', 'testnet', true, '{"region": "us-west-2", "provider": "aws"}')
ON CONFLICT (name) DO NOTHING;

-- ============================================================================
-- 2. 启用 TimescaleDB 并创建指标数据表
-- ============================================================================

-- 启用 TimescaleDB 扩展
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- 创建指标类型枚举
DO $$ BEGIN
    CREATE TYPE metric_type AS ENUM (
        'block_height',
        'block_time',
        'node_count',
        'tx_pool_size',
        'tps',
        'cpu_usage',
        'memory_usage',
        'disk_usage',
        'network_in',
        'network_out'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- 创建指标数据表（时序表）
CREATE TABLE IF NOT EXISTS metric_data (
    collected_at TIMESTAMPTZ NOT NULL,
    node_id INTEGER NOT NULL REFERENCES blockchain_nodes(id) ON DELETE CASCADE,
    metric_type metric_type NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    labels JSONB DEFAULT '{}'
);

-- 转换为 TimescaleDB 超表
SELECT create_hypertable('metric_data', 'collected_at', 
    if_not_exists => TRUE,
    chunk_time_interval => INTERVAL '1 day'
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_metric_data_node_id ON metric_data(node_id, collected_at DESC);
CREATE INDEX IF NOT EXISTS idx_metric_data_type ON metric_data(metric_type, collected_at DESC);
CREATE INDEX IF NOT EXISTS idx_metric_data_labels ON metric_data USING GIN(labels);

-- 设置数据保留策略（保留90天）
SELECT add_retention_policy('metric_data', INTERVAL '90 days', if_not_exists => TRUE);

-- 启用压缩（7天后压缩）
ALTER TABLE metric_data SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'node_id,metric_type'
);

SELECT add_compression_policy('metric_data', INTERVAL '7 days', if_not_exists => TRUE);

-- ============================================================================
-- 3. 创建小时级聚合视图
-- ============================================================================

CREATE MATERIALIZED VIEW IF NOT EXISTS hourly_metrics
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket('1 hour', collected_at) AS hour,
    node_id,
    metric_type,
    AVG(metric_value) AS avg_value,
    MIN(metric_value) AS min_value,
    MAX(metric_value) AS max_value,
    COUNT(*) AS sample_count
FROM metric_data
GROUP BY hour, node_id, metric_type
WITH NO DATA;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_hourly_metrics_hour ON hourly_metrics(hour DESC);
CREATE INDEX IF NOT EXISTS idx_hourly_metrics_node ON hourly_metrics(node_id, hour DESC);
CREATE INDEX IF NOT EXISTS idx_hourly_metrics_type ON hourly_metrics(metric_type, hour DESC);

-- 设置刷新策略（每小时刷新一次）
SELECT add_continuous_aggregate_policy('hourly_metrics',
    start_offset => INTERVAL '3 hours',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour',
    if_not_exists => TRUE
);

-- 设置保留策略（保留180天）
SELECT add_retention_policy('hourly_metrics', INTERVAL '180 days', if_not_exists => TRUE);

-- ============================================================================
-- 4. 创建日级聚合视图
-- ============================================================================

CREATE MATERIALIZED VIEW IF NOT EXISTS daily_metrics
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket('1 day', collected_at) AS day,
    node_id,
    metric_type,
    AVG(metric_value) AS avg_value,
    MIN(metric_value) AS min_value,
    MAX(metric_value) AS max_value,
    COUNT(*) AS sample_count
FROM metric_data
GROUP BY day, node_id, metric_type
WITH NO DATA;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_daily_metrics_day ON daily_metrics(day DESC);
CREATE INDEX IF NOT EXISTS idx_daily_metrics_node ON daily_metrics(node_id, day DESC);
CREATE INDEX IF NOT EXISTS idx_daily_metrics_type ON daily_metrics(metric_type, day DESC);

-- 设置刷新策略（每天刷新一次）
SELECT add_continuous_aggregate_policy('daily_metrics',
    start_offset => INTERVAL '3 days',
    end_offset => INTERVAL '1 day',
    schedule_interval => INTERVAL '1 day',
    if_not_exists => TRUE
);

-- 设置保留策略（保留365天）
SELECT add_retention_policy('daily_metrics', INTERVAL '365 days', if_not_exists => TRUE);

-- ============================================================================
-- 5. 创建月级聚合视图
-- ============================================================================

CREATE MATERIALIZED VIEW IF NOT EXISTS monthly_metrics
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket('1 month', collected_at) AS month,
    node_id,
    metric_type,
    AVG(metric_value) AS avg_value,
    MIN(metric_value) AS min_value,
    MAX(metric_value) AS max_value,
    COUNT(*) AS sample_count
FROM metric_data
GROUP BY month, node_id, metric_type
WITH NO DATA;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_monthly_metrics_month ON monthly_metrics(month DESC);
CREATE INDEX IF NOT EXISTS idx_monthly_metrics_node ON monthly_metrics(node_id, month DESC);
CREATE INDEX IF NOT EXISTS idx_monthly_metrics_type ON monthly_metrics(metric_type, month DESC);

-- 设置刷新策略（每周刷新一次）
SELECT add_continuous_aggregate_policy('monthly_metrics',
    start_offset => INTERVAL '3 months',
    end_offset => INTERVAL '1 month',
    schedule_interval => INTERVAL '1 week',
    if_not_exists => TRUE
);

-- 设置保留策略（永久保留）
-- SELECT add_retention_policy('monthly_metrics', INTERVAL '10 years', if_not_exists => TRUE);

-- ============================================================================
-- 6. 创建告警规则表
-- ============================================================================

-- 创建告警严重程度枚举
DO $$ BEGIN
    CREATE TYPE alert_severity AS ENUM ('critical', 'warning', 'info');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- 创建告警规则表
CREATE TABLE IF NOT EXISTS alert_rules (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    node_id INTEGER REFERENCES blockchain_nodes(id) ON DELETE CASCADE,
    metric_type VARCHAR(50) NOT NULL,
    condition_type VARCHAR(50) NOT NULL,
    threshold_value DOUBLE PRECISION,
    time_window_seconds INTEGER NOT NULL DEFAULT 300,
    severity alert_severity NOT NULL DEFAULT 'warning',
    enabled BOOLEAN NOT NULL DEFAULT true,
    silence_period_seconds INTEGER NOT NULL DEFAULT 1800,
    notification_channels JSONB DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_alert_rules_enabled ON alert_rules(enabled);
CREATE INDEX IF NOT EXISTS idx_alert_rules_node_id ON alert_rules(node_id);
CREATE INDEX IF NOT EXISTS idx_alert_rules_metric_type ON alert_rules(metric_type);
CREATE INDEX IF NOT EXISTS idx_alert_rules_severity ON alert_rules(severity);

-- 创建触发器
DROP TRIGGER IF EXISTS update_alert_rules_updated_at ON alert_rules;
CREATE TRIGGER update_alert_rules_updated_at
    BEFORE UPDATE ON alert_rules
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- 插入示例告警规则（可选）
INSERT INTO alert_rules (name, description, node_id, metric_type, condition_type, threshold_value, time_window_seconds, severity, enabled, notification_channels)
VALUES 
    ('Block Height Stall', '区块高度停滞告警', 1, 'block_height', 'rate_of_change', 0.1, 600, 'critical', true, '["admin@example.com"]'),
    ('High CPU Usage', 'CPU 使用率过高告警', 1, 'cpu_usage', 'threshold', 80.0, 300, 'warning', true, '["ops@example.com"]'),
    ('Low TPS', 'TPS 过低告警', 1, 'tps', 'threshold', 10.0, 300, 'warning', true, '["admin@example.com"]')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- 7. 创建告警事件表
-- ============================================================================

-- 创建告警状态枚举
DO $$ BEGIN
    CREATE TYPE alert_status AS ENUM ('triggered', 'resolved', 'silenced');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- 创建告警事件表
CREATE TABLE IF NOT EXISTS alert_events (
    id SERIAL PRIMARY KEY,
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
    metadata JSONB DEFAULT '{}'
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_alert_events_rule_id ON alert_events(rule_id, triggered_at DESC);
CREATE INDEX IF NOT EXISTS idx_alert_events_node_id ON alert_events(node_id, triggered_at DESC);
CREATE INDEX IF NOT EXISTS idx_alert_events_status ON alert_events(status);
CREATE INDEX IF NOT EXISTS idx_alert_events_severity ON alert_events(severity);
CREATE INDEX IF NOT EXISTS idx_alert_events_triggered_at ON alert_events(triggered_at DESC);
CREATE INDEX IF NOT EXISTS idx_alert_events_metadata ON alert_events USING GIN(metadata);

-- 创建告警统计视图
CREATE OR REPLACE VIEW alert_statistics AS
SELECT 
    COUNT(*) FILTER (WHERE status = 'triggered') AS active_alerts,
    COUNT(*) FILTER (WHERE status = 'resolved') AS resolved_alerts,
    COUNT(*) FILTER (WHERE status = 'silenced') AS silenced_alerts,
    COUNT(*) FILTER (WHERE severity = 'critical') AS critical_alerts,
    COUNT(*) FILTER (WHERE severity = 'warning') AS warning_alerts,
    COUNT(*) FILTER (WHERE severity = 'info') AS info_alerts,
    AVG(EXTRACT(EPOCH FROM (resolved_at - triggered_at))) FILTER (WHERE resolved_at IS NOT NULL) AS avg_resolution_time_seconds
FROM alert_events
WHERE triggered_at >= NOW() - INTERVAL '24 hours';

-- ============================================================================
-- 8. 创建 SMTP 配置表
-- ============================================================================

CREATE TABLE IF NOT EXISTS smtp_configs (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    server VARCHAR(255) NOT NULL,
    port INTEGER NOT NULL,
    username VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL,
    from_address VARCHAR(255) NOT NULL,
    use_tls BOOLEAN NOT NULL DEFAULT true,
    is_default BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_smtp_configs_is_default ON smtp_configs(is_default);

-- 创建触发器
DROP TRIGGER IF EXISTS update_smtp_configs_updated_at ON smtp_configs;
CREATE TRIGGER update_smtp_configs_updated_at
    BEFORE UPDATE ON smtp_configs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- 创建告警接收人表
CREATE TABLE IF NOT EXISTS alert_recipients (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_alert_recipients_enabled ON alert_recipients(enabled);
CREATE INDEX IF NOT EXISTS idx_alert_recipients_email ON alert_recipients(email);

-- 创建触发器
DROP TRIGGER IF EXISTS update_alert_recipients_updated_at ON alert_recipients;
CREATE TRIGGER update_alert_recipients_updated_at
    BEFORE UPDATE ON alert_recipients
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- 创建告警规则与接收人的关联表
CREATE TABLE IF NOT EXISTS alert_rule_recipients (
    rule_id INTEGER NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    recipient_id INTEGER NOT NULL REFERENCES alert_recipients(id) ON DELETE CASCADE,
    PRIMARY KEY (rule_id, recipient_id)
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_alert_rule_recipients_rule ON alert_rule_recipients(rule_id);
CREATE INDEX IF NOT EXISTS idx_alert_rule_recipients_recipient ON alert_rule_recipients(recipient_id);

-- 插入示例 SMTP 配置（请修改为实际配置）
INSERT INTO smtp_configs (name, server, port, username, password, from_address, use_tls, is_default)
VALUES 
    ('Default SMTP', 'smtp.gmail.com', 587, 'alerts@example.com', 'your-password-here', 'AIW3 Alerts <alerts@example.com>', true, true)
ON CONFLICT (name) DO NOTHING;

-- 插入示例接收人
INSERT INTO alert_recipients (name, email, enabled)
VALUES 
    ('Admin', 'admin@example.com', true),
    ('Operations Team', 'ops@example.com', true)
ON CONFLICT (email) DO NOTHING;

-- ============================================================================
-- 完成
-- ============================================================================

-- 刷新所有连续聚合视图
CALL refresh_continuous_aggregate('hourly_metrics', NULL, NULL);
CALL refresh_continuous_aggregate('daily_metrics', NULL, NULL);
CALL refresh_continuous_aggregate('monthly_metrics', NULL, NULL);

-- 输出统计信息
SELECT 'Database initialization completed successfully!' AS status;

SELECT 
    'blockchain_nodes' AS table_name,
    COUNT(*) AS row_count
FROM blockchain_nodes
UNION ALL
SELECT 
    'alert_rules' AS table_name,
    COUNT(*) AS row_count
FROM alert_rules
UNION ALL
SELECT 
    'smtp_configs' AS table_name,
    COUNT(*) AS row_count
FROM smtp_configs
UNION ALL
SELECT 
    'alert_recipients' AS table_name,
    COUNT(*) AS row_count
FROM alert_recipients;



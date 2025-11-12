-- ========================================
-- AIW3 监控系统 - 完整数据库架构
-- 版本: 1.0.0
-- 日期: 2025-11-08
-- 说明: 用于生产环境部署的完整 SQL 脚本
-- ========================================

-- ========================================
-- 第一部分: 扩展和基础配置
-- ========================================

-- 启用 TimescaleDB 扩展
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- ========================================
-- 第二部分: 自定义枚举类型
-- ========================================

-- 创建指标类型枚举
DO $$ BEGIN
    CREATE TYPE metric_type AS ENUM (
        'blockheight',
        'blocktime',
        'nodecount',
        'txpoolsize',
        'networklatency',
        'validatorcount',
        'tps',
        'memiavlheight',
        'blockstmconflicts'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- 创建告警级别枚举
DO $$ BEGIN
    CREATE TYPE alert_severity AS ENUM ('critical', 'warning', 'info');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- 创建告警状态枚举
DO $$ BEGIN
    CREATE TYPE alert_status AS ENUM ('triggered', 'resolved', 'silenced');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- ========================================
-- 第三部分: 核心表结构
-- ========================================

-- 1. 区块链节点表
CREATE TABLE IF NOT EXISTS blockchain_nodes (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    rpc_url VARCHAR(512) NOT NULL,
    rest_url VARCHAR(512) NOT NULL,
    grpc_url VARCHAR(512) NOT NULL,
    environment VARCHAR(50) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    labels JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_nodes_environment ON blockchain_nodes(environment);
CREATE INDEX IF NOT EXISTS idx_nodes_enabled ON blockchain_nodes(enabled);

-- 创建更新时间触发器函数
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 创建触发器
DROP TRIGGER IF EXISTS update_nodes_updated_at ON blockchain_nodes;
CREATE TRIGGER update_nodes_updated_at BEFORE UPDATE ON blockchain_nodes
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 2. 原始监控数据表
CREATE TABLE IF NOT EXISTS metric_data (
    id BIGSERIAL,
    node_id INTEGER NOT NULL REFERENCES blockchain_nodes(id) ON DELETE CASCADE,
    metric_type metric_type NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    collected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT NULL,
    PRIMARY KEY (id, collected_at)
);

-- 转换为 TimescaleDB hypertable (按时间分区)
SELECT create_hypertable('metric_data', 'collected_at', if_not_exists => TRUE);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_metric_data_node_id ON metric_data(node_id, collected_at DESC);
CREATE INDEX IF NOT EXISTS idx_metric_data_type ON metric_data(metric_type, collected_at DESC);

-- 设置数据保留策略: 原始数据保留1小时
SELECT add_retention_policy('metric_data', INTERVAL '1 hour', if_not_exists => TRUE);

-- 创建压缩策略(可选,用于节省存储空间)
ALTER TABLE metric_data SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'node_id, metric_type'
);

SELECT add_compression_policy('metric_data', INTERVAL '30 minutes', if_not_exists => TRUE);

-- 3. 告警规则表
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
CREATE INDEX IF NOT EXISTS idx_alert_rules_node_id ON alert_rules(node_id);
CREATE INDEX IF NOT EXISTS idx_alert_rules_metric_type ON alert_rules(metric_type);
CREATE INDEX IF NOT EXISTS idx_alert_rules_enabled ON alert_rules(enabled) WHERE enabled = true;

-- 创建触发器
DROP TRIGGER IF EXISTS update_alert_rules_updated_at ON alert_rules;
CREATE TRIGGER update_alert_rules_updated_at BEFORE UPDATE ON alert_rules
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 4. 告警事件表
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
CREATE INDEX IF NOT EXISTS idx_alert_events_rule_id ON alert_events(rule_id);
CREATE INDEX IF NOT EXISTS idx_alert_events_node_id ON alert_events(node_id);
CREATE INDEX IF NOT EXISTS idx_alert_events_status ON alert_events(status);
CREATE INDEX IF NOT EXISTS idx_alert_events_triggered_at ON alert_events(triggered_at DESC);
CREATE INDEX IF NOT EXISTS idx_alert_events_severity ON alert_events(severity);

-- 5. SMTP 配置表
CREATE TABLE IF NOT EXISTS smtp_configs (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    server VARCHAR(255) NOT NULL,
    port INTEGER NOT NULL DEFAULT 587,
    username VARCHAR(255) NOT NULL,
    password_encrypted TEXT NOT NULL, -- 使用 AES-256 加密存储
    from_address VARCHAR(255) NOT NULL,
    from_name VARCHAR(255),
    use_tls BOOLEAN NOT NULL DEFAULT true,
    use_starttls BOOLEAN NOT NULL DEFAULT true,
    timeout_seconds INTEGER NOT NULL DEFAULT 30,
    is_default BOOLEAN NOT NULL DEFAULT false,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_smtp_configs_enabled ON smtp_configs(enabled) WHERE enabled = true;
CREATE INDEX IF NOT EXISTS idx_smtp_configs_default ON smtp_configs(is_default) WHERE is_default = true;

-- 创建触发器
DROP TRIGGER IF EXISTS update_smtp_configs_updated_at ON smtp_configs;
CREATE TRIGGER update_smtp_configs_updated_at BEFORE UPDATE ON smtp_configs
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 确保只有一个默认 SMTP 配置
CREATE UNIQUE INDEX IF NOT EXISTS idx_smtp_configs_single_default ON smtp_configs(is_default) 
WHERE is_default = true;

-- 6. 邮件接收人表
CREATE TABLE IF NOT EXISTS alert_recipients (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_alert_recipients_email ON alert_recipients(email);
CREATE INDEX IF NOT EXISTS idx_alert_recipients_enabled ON alert_recipients(enabled) WHERE enabled = true;

-- 7. 告警规则与接收人的关联表
CREATE TABLE IF NOT EXISTS alert_rule_recipients (
    rule_id INTEGER NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    recipient_id INTEGER NOT NULL REFERENCES alert_recipients(id) ON DELETE CASCADE,
    PRIMARY KEY (rule_id, recipient_id)
);

-- ========================================
-- 第四部分: 聚合视图
-- ========================================

-- 1. 小时级聚合视图(保留7天)
DROP MATERIALIZED VIEW IF EXISTS hourly_metrics CASCADE;

CREATE MATERIALIZED VIEW hourly_metrics
WITH (timescaledb.continuous) AS
SELECT
    node_id,
    metric_type,
    time_bucket('1 hour', collected_at) AS time_bucket,
    AVG(metric_value) AS avg_value,
    MIN(metric_value) AS min_value,
    MAX(metric_value) AS max_value,
    STDDEV(metric_value) AS stddev_value,
    COUNT(*) AS sample_count
FROM metric_data
GROUP BY node_id, metric_type, time_bucket('1 hour', collected_at)
WITH NO DATA;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_hourly_metrics_node ON hourly_metrics(node_id, time_bucket DESC);
CREATE INDEX IF NOT EXISTS idx_hourly_metrics_type ON hourly_metrics(metric_type, time_bucket DESC);

-- 添加持续聚合刷新策略
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM timescaledb_information.jobs
        WHERE proc_name = 'policy_refresh_continuous_aggregate'
        AND hypertable_name = 'hourly_metrics'
    ) THEN
        PERFORM add_continuous_aggregate_policy('hourly_metrics',
            start_offset => INTERVAL '3 hours',
            end_offset => INTERVAL '1 hour',
            schedule_interval => INTERVAL '1 hour');
    END IF;
END $$;

-- 2. 天级聚合视图(保留90天)
DROP MATERIALIZED VIEW IF EXISTS daily_metrics CASCADE;

CREATE MATERIALIZED VIEW daily_metrics
WITH (timescaledb.continuous) AS
SELECT
    node_id,
    metric_type,
    time_bucket('1 day', collected_at) AS time_bucket,
    AVG(metric_value) AS avg_value,
    MIN(metric_value) AS min_value,
    MAX(metric_value) AS max_value,
    STDDEV(metric_value) AS stddev_value,
    COUNT(*) AS sample_count
FROM metric_data
GROUP BY node_id, metric_type, time_bucket('1 day', collected_at)
WITH NO DATA;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_daily_metrics_node ON daily_metrics(node_id, time_bucket DESC);
CREATE INDEX IF NOT EXISTS idx_daily_metrics_type ON daily_metrics(metric_type, time_bucket DESC);

-- 添加持续聚合刷新策略
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM timescaledb_information.jobs
        WHERE proc_name = 'policy_refresh_continuous_aggregate'
        AND hypertable_name = 'daily_metrics'
    ) THEN
        PERFORM add_continuous_aggregate_policy('daily_metrics',
            start_offset => INTERVAL '3 days',
            end_offset => INTERVAL '1 day',
            schedule_interval => INTERVAL '1 day');
    END IF;
END $$;

-- 3. 月级聚合视图(保留1年)
DROP MATERIALIZED VIEW IF EXISTS monthly_metrics CASCADE;

CREATE MATERIALIZED VIEW monthly_metrics
WITH (timescaledb.continuous) AS
SELECT
    node_id,
    metric_type,
    time_bucket('1 month', collected_at) AS time_bucket,
    AVG(metric_value) AS avg_value,
    MIN(metric_value) AS min_value,
    MAX(metric_value) AS max_value,
    STDDEV(metric_value) AS stddev_value,
    COUNT(*) AS sample_count
FROM metric_data
GROUP BY node_id, metric_type, time_bucket('1 month', collected_at)
WITH NO DATA;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_monthly_metrics_node ON monthly_metrics(node_id, time_bucket DESC);
CREATE INDEX IF NOT EXISTS idx_monthly_metrics_type ON monthly_metrics(metric_type, time_bucket DESC);

-- 注意: 月级视图的策略可能需要调整时间窗口
-- 如果出现 "policy refresh window too small" 错误，可以增大 start_offset

-- ========================================
-- 第五部分: 统计视图
-- ========================================

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

-- ========================================
-- 第六部分: 初始数据
-- ========================================

-- 插入示例节点
INSERT INTO blockchain_nodes (name, rpc_url, rest_url, grpc_url, environment, labels)
VALUES (
    'AIWS DevNet 3',
    'https://devnet-rpc3.aiw3.io',
    'https://devnet-api3.aiw3.io',
    'https://devnet-grpc3.aiw3.io:443',
    'devnet',
    '{"region": "us-west", "chain_id": "aiw3chain-devnet"}'::jsonb
)
ON CONFLICT (name) DO NOTHING;

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
)
ON CONFLICT (name) DO NOTHING;

-- 插入示例 SMTP 配置
INSERT INTO smtp_configs (
    name, server, port, username, password_encrypted, 
    from_address, from_name, use_tls, is_default
) VALUES (
    'Default SMTP',
    'smtp.gmail.com',
    587,
    'alerts@example.com',
    'ENCRYPTED_PASSWORD_PLACEHOLDER', -- 需要在应用中替换为真实加密密码
    'aiw3-monitor@example.com',
    'AIWS Monitor',
    true,
    true
) ON CONFLICT (name) DO NOTHING;

-- 插入示例接收人
INSERT INTO alert_recipients (email, name) VALUES
('admin@example.com', 'System Administrator'),
('ops@example.com', 'Operations Team')
ON CONFLICT DO NOTHING;

-- ========================================
-- 第七部分: 注释说明
-- ========================================

COMMENT ON TABLE blockchain_nodes IS '区块链节点配置表';
COMMENT ON TABLE metric_data IS '原始监控数据表，使用 TimescaleDB hypertable';
COMMENT ON TABLE alert_rules IS '告警规则配置表';
COMMENT ON COLUMN alert_rules.condition_type IS '条件类型: threshold(阈值), no_change(无变化), rate_of_change(变化率)';
COMMENT ON COLUMN alert_rules.silence_period_seconds IS '告警静默期(秒),避免重复告警';
COMMENT ON TABLE alert_events IS '告警事件记录表';
COMMENT ON COLUMN alert_events.status IS '告警状态: triggered(已触发), resolved(已恢复), silenced(已静默)';
COMMENT ON TABLE smtp_configs IS 'SMTP 邮件服务器配置表';
COMMENT ON COLUMN smtp_configs.password_encrypted IS '密码使用 AES-256 加密存储';
COMMENT ON TABLE alert_recipients IS '告警通知接收人表';
COMMENT ON TABLE alert_rule_recipients IS '告警规则与接收人的多对多关联表';
COMMENT ON MATERIALIZED VIEW hourly_metrics IS '小时级聚合指标数据,保留7天';
COMMENT ON MATERIALIZED VIEW daily_metrics IS '天级聚合指标数据,保留90天,用于月度和季度报告';
COMMENT ON MATERIALIZED VIEW monthly_metrics IS '月级聚合指标数据,保留1年,用于年度趋势分析';
COMMENT ON VIEW alert_statistics IS '告警统计视图,用于分析告警频率和响应时间';

-- ========================================
-- 部署完成
-- ========================================
-- 执行此脚本后，数据库架构将完全就绪
-- 请确保：
-- 1. TimescaleDB 扩展已安装
-- 2. 数据库用户具有足够的权限
-- 3. 根据实际需求调整保留策略和聚合策略
-- 4. 更新 SMTP 配置中的真实密码
-- 5. 添加实际的节点和告警规则
-- ========================================


-- 创建天级聚合视图(保留90天)
CREATE MATERIALIZED VIEW IF NOT EXISTS daily_metrics
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
GROUP BY node_id, metric_type, time_bucket('1 day', collected_at);

-- 创建持续聚合刷新策略
SELECT add_continuous_aggregate_policy('daily_metrics',
    start_offset => INTERVAL '3 days',
    end_offset => INTERVAL '1 day',
    schedule_interval => INTERVAL '1 day',
    if_not_exists => TRUE);

-- 设置数据保留策略: 天级数据保留90天
SELECT add_retention_policy('daily_metrics', INTERVAL '90 days', if_not_exists => TRUE);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_daily_metrics_node ON daily_metrics(node_id, time_bucket DESC);
CREATE INDEX IF NOT EXISTS idx_daily_metrics_type ON daily_metrics(metric_type, time_bucket DESC);

-- 添加注释
COMMENT ON MATERIALIZED VIEW daily_metrics IS '天级聚合指标数据,保留90天,用于月度和季度报告';


-- 创建月级聚合视图(保留1年)
CREATE MATERIALIZED VIEW IF NOT EXISTS monthly_metrics
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
GROUP BY node_id, metric_type, time_bucket('1 month', collected_at);

-- 创建持续聚合刷新策略
SELECT add_continuous_aggregate_policy('monthly_metrics',
    start_offset => INTERVAL '40 days',
    end_offset => INTERVAL '1 day',
    schedule_interval => INTERVAL '1 day',
    if_not_exists => TRUE);

-- 设置数据保留策略: 月级数据保留365天
SELECT add_retention_policy('monthly_metrics', INTERVAL '365 days', if_not_exists => TRUE);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_monthly_metrics_node ON monthly_metrics(node_id, time_bucket DESC);
CREATE INDEX IF NOT EXISTS idx_monthly_metrics_type ON monthly_metrics(metric_type, time_bucket DESC);

-- 添加注释
COMMENT ON MATERIALIZED VIEW monthly_metrics IS '月级聚合指标数据,保留1年,用于年度趋势分析';


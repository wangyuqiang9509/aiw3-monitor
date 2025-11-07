-- 创建小时级聚合视图(保留7天)
CREATE MATERIALIZED VIEW IF NOT EXISTS hourly_metrics
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
GROUP BY node_id, metric_type, time_bucket('1 hour', collected_at);

-- 创建持续聚合刷新策略
SELECT add_continuous_aggregate_policy('hourly_metrics',
    start_offset => INTERVAL '3 hours',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);

-- 设置数据保留策略: 小时数据保留7天
SELECT add_retention_policy('hourly_metrics', INTERVAL '7 days', if_not_exists => TRUE);

-- 创建索引
CREATE INDEX idx_hourly_metrics_node ON hourly_metrics(node_id, time_bucket DESC);
CREATE INDEX idx_hourly_metrics_type ON hourly_metrics(metric_type, time_bucket DESC);


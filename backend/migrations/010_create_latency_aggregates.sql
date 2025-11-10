-- 创建延迟指标的 TimescaleDB 连续聚合视图
-- 用于自动计算区块时间和 API 延迟的统计数据

-- 1分钟聚合视图（区块时间和 API 延迟）
CREATE MATERIALIZED VIEW IF NOT EXISTS metrics_latency_1min
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket('1 minute', collected_at) AS bucket,
    node_id,
    metric_type,
    AVG(metric_value) as avg_value,
    MIN(metric_value) as min_value,
    MAX(metric_value) as max_value,
    STDDEV(metric_value) as std_dev,
    COUNT(*) as sample_count
FROM metrics
WHERE metric_type IN ('blocktime', 'apilatency')
GROUP BY bucket, node_id, metric_type
WITH NO DATA;

-- 添加刷新策略：每分钟刷新一次，处理最近 1 小时到 1 分钟前的数据
SELECT add_continuous_aggregate_policy('metrics_latency_1min',
    start_offset => INTERVAL '1 hour',
    end_offset => INTERVAL '1 minute',
    schedule_interval => INTERVAL '1 minute');

-- 5分钟聚合视图（基于 1 分钟聚合）
CREATE MATERIALIZED VIEW IF NOT EXISTS metrics_latency_5min
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket('5 minutes', bucket) AS bucket,
    node_id,
    metric_type,
    AVG(avg_value) as avg_value,
    MIN(min_value) as min_value,
    MAX(max_value) as max_value,
    AVG(std_dev) as avg_std_dev,
    SUM(sample_count) as total_samples
FROM metrics_latency_1min
GROUP BY time_bucket('5 minutes', bucket), node_id, metric_type
WITH NO DATA;

-- 添加刷新策略：每 5 分钟刷新一次
SELECT add_continuous_aggregate_policy('metrics_latency_5min',
    start_offset => INTERVAL '6 hours',
    end_offset => INTERVAL '5 minutes',
    schedule_interval => INTERVAL '5 minutes');

-- 1小时聚合视图（基于 5 分钟聚合）
CREATE MATERIALIZED VIEW IF NOT EXISTS metrics_latency_1hour
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket('1 hour', bucket) AS bucket,
    node_id,
    metric_type,
    AVG(avg_value) as avg_value,
    MIN(min_value) as min_value,
    MAX(max_value) as max_value,
    AVG(avg_std_dev) as avg_std_dev,
    SUM(total_samples) as total_samples
FROM metrics_latency_5min
GROUP BY time_bucket('1 hour', bucket), node_id, metric_type
WITH NO DATA;

-- 添加刷新策略：每小时刷新一次
SELECT add_continuous_aggregate_policy('metrics_latency_1hour',
    start_offset => INTERVAL '1 day',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour');

-- 创建索引以优化查询性能
CREATE INDEX IF NOT EXISTS idx_metrics_latency_1min_node_type 
    ON metrics_latency_1min (node_id, metric_type, bucket DESC);

CREATE INDEX IF NOT EXISTS idx_metrics_latency_5min_node_type 
    ON metrics_latency_5min (node_id, metric_type, bucket DESC);

CREATE INDEX IF NOT EXISTS idx_metrics_latency_1hour_node_type 
    ON metrics_latency_1hour (node_id, metric_type, bucket DESC);

-- 添加注释
COMMENT ON MATERIALIZED VIEW metrics_latency_1min IS 
    '延迟指标 1 分钟聚合视图，包含区块时间和 API 延迟的统计数据';

COMMENT ON MATERIALIZED VIEW metrics_latency_5min IS 
    '延迟指标 5 分钟聚合视图，用于中期趋势分析';

COMMENT ON MATERIALIZED VIEW metrics_latency_1hour IS 
    '延迟指标 1 小时聚合视图，用于长期趋势分析';


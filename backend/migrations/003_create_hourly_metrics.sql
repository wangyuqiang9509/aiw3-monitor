-- 创建小时级聚合视图(保留7天)
-- 注意: TimescaleDB 连续聚合视图的创建和策略添加需要分开执行

-- 步骤1: 删除已存在的视图（如果存在）
DROP MATERIALIZED VIEW IF EXISTS hourly_metrics CASCADE;

-- 步骤2: 创建连续聚合视图
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
GROUP BY node_id, metric_type, time_bucket('1 hour', collected_at);

-- 步骤3: 创建索引
CREATE INDEX IF NOT EXISTS idx_hourly_metrics_node ON hourly_metrics(node_id, time_bucket DESC);
CREATE INDEX IF NOT EXISTS idx_hourly_metrics_type ON hourly_metrics(metric_type, time_bucket DESC);

-- 步骤4: 添加持续聚合刷新策略
-- 注意: 这个需要在视图创建后单独执行
DO $$
BEGIN
    -- 检查策略是否已存在
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

-- 步骤5: 设置数据保留策略
-- 注意: 这个也需要单独执行
DO $$
BEGIN
    -- 检查保留策略是否已存在
    IF NOT EXISTS (
        SELECT 1 FROM timescaledb_information.jobs
        WHERE proc_name = 'policy_retention'
        AND config::text LIKE '%hourly_metrics%'
    ) THEN
        PERFORM add_retention_policy('hourly_metrics', INTERVAL '7 days');
    END IF;
END $$;


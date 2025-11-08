-- 创建天级聚合视图(保留90天)
-- 注意: TimescaleDB 连续聚合视图的创建和策略添加需要分开执行

-- 步骤1: 删除已存在的视图（如果存在）
DROP MATERIALIZED VIEW IF EXISTS daily_metrics CASCADE;

-- 步骤2: 创建连续聚合视图
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
GROUP BY node_id, metric_type, time_bucket('1 day', collected_at);

-- 步骤3: 创建索引
CREATE INDEX IF NOT EXISTS idx_daily_metrics_node ON daily_metrics(node_id, time_bucket DESC);
CREATE INDEX IF NOT EXISTS idx_daily_metrics_type ON daily_metrics(metric_type, time_bucket DESC);

-- 步骤4: 添加持续聚合刷新策略
DO $$
BEGIN
    -- 检查策略是否已存在
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

-- 步骤5: 设置数据保留策略
DO $$
BEGIN
    -- 检查保留策略是否已存在
    IF NOT EXISTS (
        SELECT 1 FROM timescaledb_information.jobs
        WHERE proc_name = 'policy_retention'
        AND config::text LIKE '%daily_metrics%'
    ) THEN
        PERFORM add_retention_policy('daily_metrics', INTERVAL '90 days');
    END IF;
END $$;

-- 添加注释
COMMENT ON MATERIALIZED VIEW daily_metrics IS '天级聚合指标数据,保留90天,用于月度和季度报告';


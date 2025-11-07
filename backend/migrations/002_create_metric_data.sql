-- 启用 TimescaleDB 扩展
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- 创建指标类型枚举
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

-- 创建原始监控数据表
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
CREATE INDEX idx_metric_data_node_id ON metric_data(node_id, collected_at DESC);
CREATE INDEX idx_metric_data_type ON metric_data(metric_type, collected_at DESC);

-- 设置数据保留策略: 原始数据保留1小时
SELECT add_retention_policy('metric_data', INTERVAL '1 hour', if_not_exists => TRUE);

-- 创建压缩策略(可选,用于节省存储空间)
ALTER TABLE metric_data SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'node_id, metric_type'
);

SELECT add_compression_policy('metric_data', INTERVAL '30 minutes', if_not_exists => TRUE);


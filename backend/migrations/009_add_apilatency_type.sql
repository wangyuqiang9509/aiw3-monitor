-- 添加 apilatency 到 metric_type 枚举
-- 用于记录 API 响应延迟指标

ALTER TYPE metric_type ADD VALUE IF NOT EXISTS 'apilatency';

-- 注释：此迁移添加了 API 延迟指标类型
-- 用于监控 RPC/REST API 端点的响应时间


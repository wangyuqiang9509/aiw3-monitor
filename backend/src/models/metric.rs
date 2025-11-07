use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 监控指标类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "metric_type", rename_all = "lowercase")]
pub enum MetricType {
    BlockHeight,
    BlockTime,
    NodeCount,
    TxPoolSize,
    NetworkLatency,
    ValidatorCount,
    Tps,
    MemIavlHeight,
    BlockStmConflicts,
}

/// 原始监控数据点
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MetricData {
    pub id: i64,
    pub node_id: i32,
    pub metric_type: MetricType,
    pub metric_value: f64,
    pub collected_at: chrono::DateTime<chrono::Utc>,
    pub metadata: Option<serde_json::Value>,
}

impl MetricData {
    pub fn new(
        node_id: i32,
        metric_type: MetricType,
        metric_value: f64,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: 0, // 数据库生成
            node_id,
            metric_type,
            metric_value,
            collected_at: chrono::Utc::now(),
            metadata,
        }
    }
}

/// 聚合监控数据
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AggregatedMetric {
    pub node_id: i32,
    pub metric_type: MetricType,
    pub time_bucket: chrono::DateTime<chrono::Utc>,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub p95_value: Option<f64>,
    pub p99_value: Option<f64>,
    pub sample_count: i64,
}

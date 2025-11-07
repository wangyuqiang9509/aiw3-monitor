// 指标数据存储
use sqlx::PgPool;
use tracing::{debug, error, info};

use crate::error::Result;
use crate::models::metric::{MetricData, MetricType};

/// 指标存储
pub struct MetricsStore {
    pool: PgPool,
}

impl MetricsStore {
    /// 创建新的指标存储
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 插入单个指标
    pub async fn insert(&self, metric: &MetricData) -> Result<i64> {
        debug!(
            "Inserting metric: node_id={}, type={:?}, value={}",
            metric.node_id, metric.metric_type, metric.metric_value
        );

        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO metric_data (node_id, metric_type, metric_value, collected_at, metadata)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
            metric.node_id,
            metric.metric_type as MetricType,
            metric.metric_value,
            metric.collected_at,
            metric.metadata
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    /// 批量插入指标
    pub async fn insert_batch(&self, metrics: &[MetricData]) -> Result<u64> {
        if metrics.is_empty() {
            return Ok(0);
        }

        info!("Inserting {} metrics in batch", metrics.len());

        let mut tx = self.pool.begin().await?;
        let mut inserted = 0u64;

        for metric in metrics {
            let result = sqlx::query!(
                r#"
                INSERT INTO metric_data (node_id, metric_type, metric_value, collected_at, metadata)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                metric.node_id,
                metric.metric_type as MetricType,
                metric.metric_value,
                metric.collected_at,
                metric.metadata
            )
            .execute(&mut *tx)
            .await;

            match result {
                Ok(_) => inserted += 1,
                Err(e) => {
                    error!("Failed to insert metric: {}", e);
                    // 继续插入其他指标
                }
            }
        }

        tx.commit().await?;
        info!("Successfully inserted {}/{} metrics", inserted, metrics.len());

        Ok(inserted)
    }

    /// 获取最新指标
    pub async fn get_latest(
        &self,
        node_id: i32,
        metric_type: MetricType,
        limit: i64,
    ) -> Result<Vec<MetricData>> {
        let metrics = sqlx::query_as!(
            MetricData,
            r#"
            SELECT id, node_id, metric_type as "metric_type: MetricType", 
                   metric_value, collected_at, metadata
            FROM metric_data
            WHERE node_id = $1 AND metric_type = $2
            ORDER BY collected_at DESC
            LIMIT $3
            "#,
            node_id,
            metric_type as MetricType,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(metrics)
    }

    /// 获取指定时间范围的指标
    pub async fn get_range(
        &self,
        node_id: i32,
        metric_type: MetricType,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<MetricData>> {
        let metrics = sqlx::query_as!(
            MetricData,
            r#"
            SELECT id, node_id, metric_type as "metric_type: MetricType", 
                   metric_value, collected_at, metadata
            FROM metric_data
            WHERE node_id = $1 
              AND metric_type = $2
              AND collected_at BETWEEN $3 AND $4
            ORDER BY collected_at ASC
            "#,
            node_id,
            metric_type as MetricType,
            start_time,
            end_time
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(metrics)
    }

    /// 删除过期数据(由 TimescaleDB 自动处理,此方法用于手动清理)
    pub async fn cleanup_old_data(&self, before: chrono::DateTime<chrono::Utc>) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM metric_data
            WHERE collected_at < $1
            "#,
            before
        )
        .execute(&self.pool)
        .await?;

        let deleted = result.rows_affected();
        info!("Cleaned up {} old metric records", deleted);

        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // 辅助函数: 获取测试数据库连接
    async fn get_test_pool() -> Option<sqlx::PgPool> {
        let database_url = std::env::var("DATABASE_URL").ok()?;
        sqlx::PgPool::connect(&database_url).await.ok()
    }

    #[tokio::test]
    #[ignore] // 需要数据库连接
    async fn test_insert_single_metric() {
        let pool = get_test_pool().await.expect("DATABASE_URL not set");
        let store = MetricsStore::new(pool);

        let metric = MetricData {
            id: 0,
            node_id: 1,
            metric_type: MetricType::BlockHeight,
            metric_value: 12345.0,
            collected_at: Utc::now(),
            metadata: None,
        };

        let result = store.insert(&metric).await;
        assert!(result.is_ok());
        let id = result.unwrap();
        assert!(id > 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_insert_batch_metrics() {
        let pool = get_test_pool().await.expect("DATABASE_URL not set");
        let store = MetricsStore::new(pool);

        let metrics = vec![
            MetricData {
                id: 0,
                node_id: 1,
                metric_type: MetricType::BlockHeight,
                metric_value: 100.0,
                collected_at: Utc::now(),
                metadata: None,
            },
            MetricData {
                id: 0,
                node_id: 1,
                metric_type: MetricType::NodeCount,
                metric_value: 5.0,
                collected_at: Utc::now(),
                metadata: None,
            },
            MetricData {
                id: 0,
                node_id: 1,
                metric_type: MetricType::TxPoolSize,
                metric_value: 0.0,
                collected_at: Utc::now(),
                metadata: None,
            },
        ];

        let result = store.insert_batch(&metrics).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_latest_metrics() {
        let pool = get_test_pool().await.expect("DATABASE_URL not set");
        let store = MetricsStore::new(pool);

        // 先插入一些测试数据
        let metrics = vec![
            MetricData {
                id: 0,
                node_id: 1,
                metric_type: MetricType::BlockHeight,
                metric_value: 100.0,
                collected_at: Utc::now(),
                metadata: None,
            },
            MetricData {
                id: 0,
                node_id: 1,
                metric_type: MetricType::BlockHeight,
                metric_value: 101.0,
                collected_at: Utc::now(),
                metadata: None,
            },
        ];
        store.insert_batch(&metrics).await.unwrap();

        // 获取最新的指标
        let result = store
            .get_latest(1, MetricType::BlockHeight, 10)
            .await;

        assert!(result.is_ok());
        let fetched = result.unwrap();
        assert!(fetched.len() <= 10);
        assert!(fetched.len() >= 2);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_range_metrics() {
        let pool = get_test_pool().await.expect("DATABASE_URL not set");
        let store = MetricsStore::new(pool);

        let now = Utc::now();
        let start = now - chrono::Duration::hours(1);
        let end = now;

        let result = store
            .get_range(1, MetricType::BlockHeight, start, end)
            .await;

        assert!(result.is_ok());
        let _metrics = result.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_cleanup_old_data() {
        let pool = get_test_pool().await.expect("DATABASE_URL not set");
        let store = MetricsStore::new(pool);

        // 插入一些旧数据
        let old_time = Utc::now() - chrono::Duration::days(2);
        let old_metric = MetricData {
            id: 0,
            node_id: 1,
            metric_type: MetricType::BlockHeight,
            metric_value: 50.0,
            collected_at: old_time,
            metadata: None,
        };
        store.insert(&old_metric).await.unwrap();

        // 清理 1 天前的数据
        let cutoff = Utc::now() - chrono::Duration::days(1);
        let result = store.cleanup_old_data(cutoff).await;

        assert!(result.is_ok());
        let deleted = result.unwrap();
        assert!(deleted >= 1);
    }

    #[tokio::test]
    async fn test_metrics_store_creation() {
        // 测试 MetricsStore 可以被创建(不需要数据库连接)
        use sqlx::postgres::PgPoolOptions;
        
        // 创建一个未连接的 pool (仅用于测试结构)
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://localhost/test")
            .unwrap();
        
        let _store = MetricsStore::new(pool);
        // 如果能创建,测试通过
    }
}

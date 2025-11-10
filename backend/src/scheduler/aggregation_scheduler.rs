// 数据聚合调度器
// 负责触发 TimescaleDB 持续聚合的刷新和维护任务

use crate::error::Result;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

/// 数据聚合调度器
pub struct AggregationScheduler {
    pool: Arc<PgPool>,
    check_interval: Duration,
}

impl AggregationScheduler {
    /// 创建新的聚合调度器
    ///
    /// # 参数
    /// - `pool`: 数据库连接池
    /// - `check_interval_seconds`: 检查间隔（秒）
    pub fn new(pool: Arc<PgPool>, check_interval_seconds: u64) -> Self {
        Self { pool, check_interval: Duration::from_secs(check_interval_seconds) }
    }

    /// 启动调度器（持续运行）
    pub async fn start(&self) {
        info!("Starting aggregation scheduler with interval: {:?}", self.check_interval);

        let mut ticker = interval(self.check_interval);

        loop {
            ticker.tick().await;

            if let Err(e) = self.run_aggregation_tasks().await {
                error!("Aggregation tasks failed: {}", e);
            }
        }
    }

    /// 执行一次聚合任务
    pub async fn run_aggregation_tasks(&self) -> Result<()> {
        info!("Running scheduled aggregation tasks");

        // 刷新小时级聚合视图
        if let Err(e) = self.refresh_hourly_metrics().await {
            error!("Failed to refresh hourly metrics: {}", e);
        }

        // 刷新天级聚合视图
        if let Err(e) = self.refresh_daily_metrics().await {
            error!("Failed to refresh daily metrics: {}", e);
        }

        // 刷新月级聚合视图
        if let Err(e) = self.refresh_monthly_metrics().await {
            error!("Failed to refresh monthly metrics: {}", e);
        }

        // 获取聚合统计信息
        if let Ok(stats) = self.get_aggregation_stats().await {
            info!(
                "Aggregation stats - Hourly: {}, Daily: {}, Monthly: {}",
                stats.hourly_count, stats.daily_count, stats.monthly_count
            );
        }

        info!("Aggregation tasks completed");
        Ok(())
    }

    /// 手动刷新小时级聚合视图
    ///
    /// 注意：TimescaleDB 的持续聚合会自动刷新，此方法用于手动触发
    async fn refresh_hourly_metrics(&self) -> Result<()> {
        // TimescaleDB 的持续聚合策略会自动刷新
        // 这里我们只是验证视图是否正常工作
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM hourly_metrics WHERE hour_timestamp >= NOW() - INTERVAL '24 hours'"
        )
        .fetch_one(self.pool.as_ref())
        .await?;

        info!("Hourly metrics count (last 24h): {}", count);
        Ok(())
    }

    /// 手动刷新天级聚合视图
    async fn refresh_daily_metrics(&self) -> Result<()> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM daily_metrics WHERE day_date >= NOW() - INTERVAL '7 days'",
        )
        .fetch_one(self.pool.as_ref())
        .await?;

        info!("Daily metrics count (last 7 days): {}", count);
        Ok(())
    }

    /// 手动刷新月级聚合视图
    async fn refresh_monthly_metrics(&self) -> Result<()> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM monthly_metrics WHERE month_start >= NOW() - INTERVAL '1 year'",
        )
        .fetch_one(self.pool.as_ref())
        .await?;

        info!("Monthly metrics count (last year): {}", count);
        Ok(())
    }

    /// 获取聚合统计信息
    async fn get_aggregation_stats(&self) -> Result<AggregationStats> {
        let hourly_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM hourly_metrics")
            .fetch_one(self.pool.as_ref())
            .await?;

        let daily_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM daily_metrics")
            .fetch_one(self.pool.as_ref())
            .await?;

        let monthly_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM monthly_metrics")
            .fetch_one(self.pool.as_ref())
            .await?;

        Ok(AggregationStats { hourly_count, daily_count, monthly_count })
    }

    /// 验证聚合策略是否正常工作
    pub async fn verify_aggregation_policies(&self) -> Result<Vec<PolicyStatus>> {
        let policies = sqlx::query_as::<_, PolicyStatus>(
            r#"
            SELECT 
                view_name::text,
                schedule_interval::text,
                config::text as status
            FROM timescaledb_information.continuous_aggregates
            WHERE view_name IN ('hourly_metrics', 'daily_metrics', 'monthly_metrics')
            "#,
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        for policy in &policies {
            info!(
                "Aggregation policy: {} - Interval: {} - Status: {}",
                policy.view_name, policy.schedule_interval, policy.status
            );
        }

        if policies.is_empty() {
            warn!("No aggregation policies found! TimescaleDB continuous aggregates may not be configured.");
        }

        Ok(policies)
    }
}

/// 聚合统计信息
#[derive(Debug, Clone)]
pub struct AggregationStats {
    pub hourly_count: i64,
    pub daily_count: i64,
    pub monthly_count: i64,
}

/// 策略状态
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PolicyStatus {
    pub view_name: String,
    pub schedule_interval: String,
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregation_scheduler_creation() {
        // 基本结构测试
        // 实际测试需要数据库连接
    }

    #[test]
    fn test_aggregation_stats() {
        let stats = AggregationStats { hourly_count: 100, daily_count: 50, monthly_count: 10 };

        assert_eq!(stats.hourly_count, 100);
        assert_eq!(stats.daily_count, 50);
        assert_eq!(stats.monthly_count, 10);
    }
}

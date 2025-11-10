// 数据清理调度器
// 负责清理过期的原始数据和告警事件，执行数据保留策略

use crate::error::Result;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::interval;
use tracing::{error, info, warn};

/// 数据清理调度器
pub struct CleanupScheduler {
    pool: Arc<PgPool>,
    check_interval: std::time::Duration,
}

impl CleanupScheduler {
    /// 创建新的清理调度器
    ///
    /// # 参数
    /// - `pool`: 数据库连接池
    /// - `check_interval_seconds`: 检查间隔（秒），建议每天运行一次
    pub fn new(pool: Arc<PgPool>, check_interval_seconds: u64) -> Self {
        Self { pool, check_interval: std::time::Duration::from_secs(check_interval_seconds) }
    }

    /// 启动调度器（持续运行）
    pub async fn start(&self) {
        info!("Starting cleanup scheduler with interval: {:?}", self.check_interval);

        let mut ticker = interval(self.check_interval);

        loop {
            ticker.tick().await;

            if let Err(e) = self.run_cleanup_tasks().await {
                error!("Cleanup tasks failed: {}", e);
            }
        }
    }

    /// 执行一次清理任务
    pub async fn run_cleanup_tasks(&self) -> Result<CleanupStats> {
        info!("Running scheduled cleanup tasks");

        let mut stats = CleanupStats::default();

        // 清理过期的原始指标数据（保留 1 小时）
        // 注意：TimescaleDB 的保留策略会自动清理，这里是备用清理
        match self.cleanup_old_metrics().await {
            Ok(count) => {
                stats.metrics_deleted = count;
                if count > 0 {
                    info!("Cleaned up {} old metric data points", count);
                }
            }
            Err(e) => {
                error!("Failed to cleanup old metrics: {}", e);
                stats.errors.push(format!("Metrics cleanup: {}", e));
            }
        }

        // 清理已恢复的旧告警事件（保留 90 天）
        match self.cleanup_old_alert_events().await {
            Ok(count) => {
                stats.alert_events_deleted = count;
                if count > 0 {
                    info!("Cleaned up {} old alert events", count);
                }
            }
            Err(e) => {
                error!("Failed to cleanup old alert events: {}", e);
                stats.errors.push(format!("Alert events cleanup: {}", e));
            }
        }

        // 清理孤立的活跃告警（超过 24 小时未恢复的异常告警）
        match self.cleanup_stale_active_alerts().await {
            Ok(count) => {
                stats.stale_alerts_resolved = count;
                if count > 0 {
                    warn!("Auto-resolved {} stale active alerts", count);
                }
            }
            Err(e) => {
                error!("Failed to cleanup stale alerts: {}", e);
                stats.errors.push(format!("Stale alerts cleanup: {}", e));
            }
        }

        // 验证保留策略
        if let Err(e) = self.verify_retention_policies().await {
            error!("Failed to verify retention policies: {}", e);
            stats.errors.push(format!("Retention policy verification: {}", e));
        }

        info!(
            "Cleanup tasks completed - Metrics: {}, Alerts: {}, Stale: {}, Errors: {}",
            stats.metrics_deleted,
            stats.alert_events_deleted,
            stats.stale_alerts_resolved,
            stats.errors.len()
        );

        Ok(stats)
    }

    /// 清理过期的原始指标数据
    ///
    /// TimescaleDB 的保留策略会自动清理，此方法作为备用
    async fn cleanup_old_metrics(&self) -> Result<u64> {
        let cutoff_time = Utc::now() - Duration::hours(1);

        let result = sqlx::query(
            r#"
            DELETE FROM metric_data
            WHERE collected_at < $1
            "#,
        )
        .bind(cutoff_time)
        .execute(self.pool.as_ref())
        .await?;

        Ok(result.rows_affected())
    }

    /// 清理已恢复的旧告警事件
    ///
    /// 保留 90 天的已恢复告警记录
    async fn cleanup_old_alert_events(&self) -> Result<u64> {
        let cutoff_time = Utc::now() - Duration::days(90);

        let result = sqlx::query(
            r#"
            DELETE FROM alert_events
            WHERE status = 'resolved'
              AND resolved_at < $1
            "#,
        )
        .bind(cutoff_time)
        .execute(self.pool.as_ref())
        .await?;

        Ok(result.rows_affected())
    }

    /// 清理孤立的活跃告警
    ///
    /// 将超过 24 小时未恢复的活跃告警标记为已恢复
    async fn cleanup_stale_active_alerts(&self) -> Result<u64> {
        let cutoff_time = Utc::now() - Duration::hours(24);

        let result = sqlx::query(
            r#"
            UPDATE alert_events
            SET status = 'resolved',
                resolved_at = NOW(),
                updated_at = NOW(),
                user_notes = COALESCE(user_notes, '') || ' [Auto-resolved: stale alert]'
            WHERE status = 'active'
              AND triggered_at < $1
            "#,
        )
        .bind(cutoff_time)
        .execute(self.pool.as_ref())
        .await?;

        Ok(result.rows_affected())
    }

    /// 验证 TimescaleDB 保留策略是否正常工作
    async fn verify_retention_policies(&self) -> Result<Vec<RetentionPolicy>> {
        let policies = sqlx::query_as::<_, RetentionPolicy>(
            r#"
            SELECT 
                hypertable_name::text,
                drop_after::text as retention_period
            FROM timescaledb_information.jobs
            WHERE proc_name = 'policy_retention'
            "#,
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        for policy in &policies {
            info!(
                "Retention policy: {} - Period: {}",
                policy.hypertable_name, policy.retention_period
            );
        }

        if policies.is_empty() {
            warn!("No retention policies found! Data may accumulate indefinitely.");
        }

        Ok(policies)
    }

    /// 获取数据库大小统计
    pub async fn get_database_size_stats(&self) -> Result<DatabaseSizeStats> {
        let total_size: i64 = sqlx::query_scalar("SELECT pg_database_size(current_database())")
            .fetch_one(self.pool.as_ref())
            .await?;

        let metrics_size: i64 = sqlx::query_scalar("SELECT pg_total_relation_size('metric_data')")
            .fetch_one(self.pool.as_ref())
            .await?;

        let hourly_size: i64 =
            sqlx::query_scalar("SELECT pg_total_relation_size('hourly_metrics')")
                .fetch_one(self.pool.as_ref())
                .await?;

        let daily_size: i64 = sqlx::query_scalar("SELECT pg_total_relation_size('daily_metrics')")
            .fetch_one(self.pool.as_ref())
            .await?;

        let monthly_size: i64 =
            sqlx::query_scalar("SELECT pg_total_relation_size('monthly_metrics')")
                .fetch_one(self.pool.as_ref())
                .await?;

        let alerts_size: i64 = sqlx::query_scalar("SELECT pg_total_relation_size('alert_events')")
            .fetch_one(self.pool.as_ref())
            .await?;

        Ok(DatabaseSizeStats {
            total_size_bytes: total_size,
            metrics_size_bytes: metrics_size,
            hourly_size_bytes: hourly_size,
            daily_size_bytes: daily_size,
            monthly_size_bytes: monthly_size,
            alerts_size_bytes: alerts_size,
        })
    }
}

/// 清理统计信息
#[derive(Debug, Clone, Default)]
pub struct CleanupStats {
    pub metrics_deleted: u64,
    pub alert_events_deleted: u64,
    pub stale_alerts_resolved: u64,
    pub errors: Vec<String>,
}

/// 保留策略信息
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RetentionPolicy {
    pub hypertable_name: String,
    pub retention_period: String,
}

/// 数据库大小统计
#[derive(Debug, Clone)]
pub struct DatabaseSizeStats {
    pub total_size_bytes: i64,
    pub metrics_size_bytes: i64,
    pub hourly_size_bytes: i64,
    pub daily_size_bytes: i64,
    pub monthly_size_bytes: i64,
    pub alerts_size_bytes: i64,
}

impl DatabaseSizeStats {
    /// 转换为人类可读的格式
    pub fn to_human_readable(&self) -> String {
        format!(
            "Total: {}, Metrics: {}, Hourly: {}, Daily: {}, Monthly: {}, Alerts: {}",
            bytes_to_human(self.total_size_bytes),
            bytes_to_human(self.metrics_size_bytes),
            bytes_to_human(self.hourly_size_bytes),
            bytes_to_human(self.daily_size_bytes),
            bytes_to_human(self.monthly_size_bytes),
            bytes_to_human(self.alerts_size_bytes),
        )
    }
}

/// 将字节数转换为人类可读的格式
fn bytes_to_human(bytes: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_human() {
        assert_eq!(bytes_to_human(1024), "1.00 KB");
        assert_eq!(bytes_to_human(1048576), "1.00 MB");
        assert_eq!(bytes_to_human(1073741824), "1.00 GB");
    }

    #[test]
    fn test_cleanup_stats_default() {
        let stats = CleanupStats::default();
        assert_eq!(stats.metrics_deleted, 0);
        assert_eq!(stats.alert_events_deleted, 0);
        assert_eq!(stats.stale_alerts_resolved, 0);
        assert_eq!(stats.errors.len(), 0);
    }

    #[test]
    fn test_database_size_stats_human_readable() {
        let stats = DatabaseSizeStats {
            total_size_bytes: 10737418240,  // 10 GB
            metrics_size_bytes: 1073741824, // 1 GB
            hourly_size_bytes: 104857600,   // 100 MB
            daily_size_bytes: 10485760,     // 10 MB
            monthly_size_bytes: 1048576,    // 1 MB
            alerts_size_bytes: 102400,      // 100 KB
        };

        let readable = stats.to_human_readable();
        assert!(readable.contains("10.00 GB"));
        assert!(readable.contains("1.00 GB"));
    }
}

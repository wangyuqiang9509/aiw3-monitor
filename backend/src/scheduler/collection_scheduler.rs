// 采集调度器
use std::time::Duration;
use tokio::time;
use tracing::{error, info};

use crate::collectors::metrics_collector::MetricsCollector;

/// 采集调度器
pub struct CollectionScheduler {
    collector: MetricsCollector,
    interval_seconds: u64,
}

impl CollectionScheduler {
    /// 创建新的调度器
    pub fn new(collector: MetricsCollector, interval_seconds: u64) -> Self {
        Self {
            collector,
            interval_seconds,
        }
    }

    /// 运行调度器(无限循环)
    pub async fn run(&self) {
        info!(
            "Starting collection scheduler with interval: {}s",
            self.interval_seconds
        );

        let mut interval = time::interval(Duration::from_secs(self.interval_seconds));

        loop {
            interval.tick().await;

            info!("Triggering metrics collection");
            match self.collector.collect_all().await {
                Ok(count) => {
                    info!("Successfully collected {} metrics", count);
                }
                Err(e) => {
                    error!("Metrics collection failed: {}", e);
                }
            }
        }
    }

    /// 运行一次采集(用于测试)
    pub async fn run_once(&self) -> Result<usize, crate::error::AppError> {
        self.collector.collect_all().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exporter::metrics_registry::MetricsRegistry;
    use sqlx::PgPool;

    #[tokio::test]
    #[ignore] // 需要数据库
    async fn test_run_once() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| {
                "postgres://postgres:secret@localhost:5432/aiw3_monitor_test".to_string()
            });

        let pool = PgPool::connect(&database_url).await.unwrap();
        let registry = MetricsRegistry::new().unwrap();
        let collector = MetricsCollector::new(pool, registry);
        let scheduler = CollectionScheduler::new(collector, 30);

        let result = scheduler.run_once().await;
        assert!(result.is_ok());
    }
}

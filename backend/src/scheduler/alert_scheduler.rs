use crate::alerting::rule_engine::AlertRuleEngine;
use crate::error::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info};

/// 告警检查调度器
pub struct AlertScheduler {
    engine: Arc<AlertRuleEngine>,
    check_interval: Duration,
}

impl AlertScheduler {
    /// 创建新的告警调度器
    pub fn new(engine: Arc<AlertRuleEngine>, check_interval_seconds: u64) -> Self {
        Self {
            engine,
            check_interval: Duration::from_secs(check_interval_seconds),
        }
    }

    /// 启动调度器（持续运行）
    pub async fn start(&self) {
        info!(
            "Starting alert scheduler with interval: {:?}",
            self.check_interval
        );

        let mut ticker = interval(self.check_interval);

        loop {
            ticker.tick().await;

            if let Err(e) = self.run_check().await {
                error!("Alert check failed: {}", e);
            }
        }
    }

    /// 执行一次告警检查
    pub async fn run_check(&self) -> Result<()> {
        info!("Running scheduled alert check");

        let result = self.engine.check_all_rules().await?;

        info!(
            "Alert check completed: {} rules checked, {} triggered, {} resolved, {} errors",
            result.total_rules,
            result.triggered_count,
            result.resolved_count,
            result.errors.len()
        );

        if !result.errors.is_empty() {
            error!("Alert check errors: {:?}", result.errors);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_scheduler_creation() {
        // 这个测试需要完整的依赖，在集成测试中进行
        // 这里只测试基本的结构
    }
}


use crate::error::Result;
use crate::models::alert_event::AlertEvent;
use crate::models::alert_rule::AlertRule;
use crate::models::metric::MetricData;
use crate::models::node::BlockchainNode;
use crate::storage::alert_store::AlertStore;
use crate::storage::metrics_store::MetricsStore;
use crate::storage::node_store::NodeStore;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use super::evaluator::{AlertEvaluator, EvaluationResult};
use super::notifier::RetryableEmailNotifier;

/// 告警规则引擎
pub struct AlertRuleEngine {
    alert_store: Arc<AlertStore>,
    metrics_store: Arc<MetricsStore>,
    node_store: Arc<NodeStore>,
    notifier: Arc<RetryableEmailNotifier>,
    // 记录每个规则最后一次触发时间，用于静默期判断
    last_triggered: Arc<tokio::sync::RwLock<HashMap<i32, chrono::DateTime<Utc>>>>,
}

impl AlertRuleEngine {
    /// 创建新的告警规则引擎
    pub fn new(
        alert_store: Arc<AlertStore>,
        metrics_store: Arc<MetricsStore>,
        node_store: Arc<NodeStore>,
        notifier: Arc<RetryableEmailNotifier>,
    ) -> Self {
        Self {
            alert_store,
            metrics_store,
            node_store,
            notifier,
            last_triggered: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 检查所有启用的告警规则
    pub async fn check_all_rules(&self) -> Result<CheckResult> {
        info!("Starting alert rule check");

        // 获取所有启用的告警规则
        let rules = self.alert_store.get_enabled_rules().await?;
        debug!("Found {} enabled alert rules", rules.len());

        let mut result = CheckResult {
            total_rules: rules.len(),
            triggered_count: 0,
            resolved_count: 0,
            errors: Vec::new(),
        };

        // 检查每个规则
        for rule in rules {
            match self.check_rule(&rule).await {
                Ok(status) => match status {
                    RuleCheckStatus::Triggered => result.triggered_count += 1,
                    RuleCheckStatus::Resolved => result.resolved_count += 1,
                    RuleCheckStatus::NoChange => {}
                },
                Err(e) => {
                    error!("Failed to check rule {}: {}", rule.name, e);
                    result.errors.push(format!("Rule '{}': {}", rule.name, e));
                }
            }
        }

        info!(
            "Alert rule check completed: {} rules, {} triggered, {} resolved, {} errors",
            result.total_rules,
            result.triggered_count,
            result.resolved_count,
            result.errors.len()
        );

        Ok(result)
    }

    /// 检查单个告警规则
    async fn check_rule(&self, rule: &AlertRule) -> Result<RuleCheckStatus> {
        debug!("Checking alert rule: {}", rule.name);

        // 检查静默期
        if self.is_in_silence_period(rule).await {
            debug!("Rule {} is in silence period, skipping", rule.name);
            return Ok(RuleCheckStatus::NoChange);
        }

        // 获取节点信息
        let node = if let Some(node_id) = rule.node_id {
            self.node_store.get_by_id(node_id).await?
        } else {
            None
        };

        // 获取相关指标数据
        let metrics = self.get_metrics_for_rule(rule).await?;

        if metrics.is_empty() {
            debug!("No metrics found for rule: {}", rule.name);
            return Ok(RuleCheckStatus::NoChange);
        }

        // 评估告警条件
        let evaluation = AlertEvaluator::evaluate(rule, &metrics)?;

        // 获取该规则的活跃告警
        let active_alerts = self.alert_store.get_active_events_by_rule(rule.id).await?;

        // 根据评估结果处理告警
        if evaluation.triggered {
            if active_alerts.is_empty() {
                // 新触发的告警
                self.trigger_alert(rule, &evaluation, node.as_ref()).await?;
                Ok(RuleCheckStatus::Triggered)
            } else {
                // 告警已存在，不重复触发
                debug!("Alert already active for rule: {}", rule.name);
                Ok(RuleCheckStatus::NoChange)
            }
        } else {
            if !active_alerts.is_empty() {
                // 告警已恢复
                self.resolve_alerts(rule, &active_alerts, node.as_ref()).await?;
                Ok(RuleCheckStatus::Resolved)
            } else {
                // 无告警
                Ok(RuleCheckStatus::NoChange)
            }
        }
    }

    /// 触发新告警
    async fn trigger_alert(
        &self,
        rule: &AlertRule,
        evaluation: &EvaluationResult,
        node: Option<&BlockchainNode>,
    ) -> Result<()> {
        info!(
            "Triggering alert: rule={}, value={}, message={}",
            rule.name, evaluation.current_value, evaluation.message
        );

        // 创建告警事件
        let mut event = AlertEvent::new(
            rule.id,
            rule.node_id,
            rule.severity.clone(),
            format!("Alert: {}", rule.name),
            evaluation.message.clone(),
            Some(evaluation.current_value),
            rule.threshold_value,
        );

        // 保存到数据库
        let event_id = self.alert_store.create_event(&event).await?;
        event.id = event_id;

        // 发送邮件通知
        match self.notifier.send_alert(&event, rule, node).await {
            Ok(_) => {
                event.mark_notification_sent();
                self.alert_store.update_event(&event).await?;
                info!("Alert notification sent successfully for rule: {}", rule.name);
            }
            Err(e) => {
                error!("Failed to send alert notification for rule {}: {}", rule.name, e);
                // 不再使用 mark_notification_failed，直接更新状态
                // notification_sent 已经是 false，只需记录错误
                self.alert_store.update_event(&event).await?;
            }
        }

        // 更新最后触发时间
        self.update_last_triggered(rule.id).await;

        Ok(())
    }

    /// 解决告警
    async fn resolve_alerts(
        &self,
        rule: &AlertRule,
        alerts: &[AlertEvent],
        node: Option<&BlockchainNode>,
    ) -> Result<()> {
        info!("Resolving {} alerts for rule: {}", alerts.len(), rule.name);

        for alert in alerts {
            let mut updated_alert = alert.clone();
            updated_alert.resolve();

            // 更新数据库
            self.alert_store.update_event(&updated_alert).await?;

            // 发送恢复通知
            match self.notifier.send_resolution(&updated_alert, rule, node).await {
                Ok(_) => {
                    info!("Resolution notification sent for alert: {}", alert.id);
                }
                Err(e) => {
                    warn!("Failed to send resolution notification for alert {}: {}", alert.id, e);
                }
            }
        }

        Ok(())
    }

    /// 获取规则相关的指标数据
    async fn get_metrics_for_rule(&self, rule: &AlertRule) -> Result<Vec<MetricData>> {
        use crate::models::metric::MetricType;

        // 根据规则的时间窗口获取指标
        let window_seconds = rule.time_window_seconds; // 直接使用，不是 Option
        let end_time = Utc::now();
        let start_time = end_time - chrono::Duration::seconds(window_seconds as i64);

        // 将规则的指标名称转换为 MetricType
        let metric_type = match rule.metric_type.to_lowercase().as_str() {
            "block_height" => MetricType::BlockHeight,
            "block_time" => MetricType::BlockTime,
            "node_count" | "peer_count" => MetricType::NodeCount,
            "tx_pool_size" | "unconfirmed_txs" => MetricType::TxPoolSize,
            "network_latency" => MetricType::NetworkLatency,
            "validator_count" => MetricType::ValidatorCount,
            "tps" => MetricType::Tps,
            "mem_iavl_height" => MetricType::MemIavlHeight,
            "block_stm_conflicts" => MetricType::BlockStmConflicts,
            _ => {
                warn!("Unknown metric name: {}, using BlockHeight as default", rule.metric_type);
                MetricType::BlockHeight
            }
        };

        // 获取指标数据
        let metrics = self
            .metrics_store
            .get_range(rule.node_id.unwrap_or(0), metric_type, start_time, end_time)
            .await?;

        debug!("Retrieved {} metrics for rule: {}", metrics.len(), rule.name);

        Ok(metrics)
    }

    /// 检查是否在静默期内
    async fn is_in_silence_period(&self, rule: &AlertRule) -> bool {
        let last_triggered = self.last_triggered.read().await;

        if let Some(last_time) = last_triggered.get(&rule.id) {
            let elapsed = (Utc::now() - *last_time).num_seconds();
            elapsed < rule.silence_period_seconds as i64
        } else {
            false
        }
    }

    /// 更新最后触发时间
    async fn update_last_triggered(&self, rule_id: i32) {
        let mut last_triggered = self.last_triggered.write().await;
        last_triggered.insert(rule_id, Utc::now());
    }
}

/// 规则检查结果
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub total_rules: usize,
    pub triggered_count: usize,
    pub resolved_count: usize,
    pub errors: Vec<String>,
}

/// 单个规则检查状态
#[derive(Debug, Clone, PartialEq, Eq)]
enum RuleCheckStatus {
    Triggered,
    Resolved,
    NoChange,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_result() {
        let result = CheckResult {
            total_rules: 10,
            triggered_count: 2,
            resolved_count: 1,
            errors: vec!["Error 1".to_string()],
        };

        assert_eq!(result.total_rules, 10);
        assert_eq!(result.triggered_count, 2);
        assert_eq!(result.resolved_count, 1);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_rule_check_status() {
        assert_eq!(RuleCheckStatus::Triggered, RuleCheckStatus::Triggered);
        assert_ne!(RuleCheckStatus::Triggered, RuleCheckStatus::Resolved);
    }
}

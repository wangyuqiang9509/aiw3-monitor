use crate::error::{AppError, Result};
use crate::models::alert_rule::{AlertRule, ComparisonOperator, ConditionType};
use crate::models::metric::MetricData;
use chrono::Utc;
use tracing::{debug, info};

/// 告警条件评估器
pub struct AlertEvaluator;

/// 评估结果
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub triggered: bool,
    pub current_value: f64,
    #[allow(dead_code)]
    pub threshold_value: Option<f64>,
    pub message: String,
}

impl AlertEvaluator {
    /// 评估告警规则
    pub fn evaluate(rule: &AlertRule, metrics: &[MetricData]) -> Result<EvaluationResult> {
        debug!("Evaluating alert rule: {} (type: {})", rule.name, rule.condition_type);

        let condition_type = ConditionType::from_str(&rule.condition_type).ok_or_else(|| {
            AppError::validation(format!("Invalid condition type: {}", rule.condition_type))
        })?;

        match condition_type {
            ConditionType::Threshold => Self::evaluate_threshold(rule, metrics),
            ConditionType::TimeWindow => Self::evaluate_time_window(rule, metrics),
            ConditionType::RateOfChange => Self::evaluate_rate_of_change(rule, metrics),
        }
    }

    /// 评估阈值条件
    fn evaluate_threshold(rule: &AlertRule, metrics: &[MetricData]) -> Result<EvaluationResult> {
        if metrics.is_empty() {
            return Ok(EvaluationResult {
                triggered: false,
                current_value: 0.0,
                threshold_value: rule.threshold_value,
                message: "No metrics available".to_string(),
            });
        }

        // 获取最新的指标值
        let latest_metric = metrics
            .iter()
            .max_by_key(|m| m.collected_at)
            .ok_or_else(|| AppError::validation("No metrics found"))?;

        let current_value = latest_metric.metric_value;
        let threshold =
            rule.threshold_value.ok_or_else(|| AppError::validation("Threshold value not set"))?;

        // 使用默认的比较运算符（大于）
        // 因为 comparison_operator 字段已被删除
        let operator = ComparisonOperator::GreaterThan;

        let triggered = operator.evaluate(current_value, threshold);

        let message = if triggered {
            format!(
                "{} {} {} (current: {})",
                rule.metric_type,
                operator.as_str(),
                threshold,
                current_value
            )
        } else {
            format!("Condition not met: {} = {}", rule.metric_type, current_value)
        };

        info!(
            "Threshold evaluation: rule={}, triggered={}, value={}, threshold={}",
            rule.name, triggered, current_value, threshold
        );

        Ok(EvaluationResult { triggered, current_value, threshold_value: Some(threshold), message })
    }

    /// 评估时间窗口条件（指标在指定时间内无变化）
    fn evaluate_time_window(rule: &AlertRule, metrics: &[MetricData]) -> Result<EvaluationResult> {
        if metrics.is_empty() {
            return Ok(EvaluationResult {
                triggered: false,
                current_value: 0.0,
                threshold_value: None,
                message: "No metrics available".to_string(),
            });
        }

        let window_seconds = rule.time_window_seconds; // 直接使用，不是 Option

        let now = Utc::now();
        let window_start = now - chrono::Duration::seconds(window_seconds as i64);

        // 获取时间窗口内的指标
        let window_metrics: Vec<_> =
            metrics.iter().filter(|m| m.collected_at >= window_start).collect();

        if window_metrics.is_empty() {
            return Ok(EvaluationResult {
                triggered: false,
                current_value: 0.0,
                threshold_value: None,
                message: "No metrics in time window".to_string(),
            });
        }

        // 检查指标是否有变化
        let first_value = window_metrics.first().unwrap().metric_value;
        let last_value = window_metrics.last().unwrap().metric_value;
        let has_changed = (last_value - first_value).abs() > f64::EPSILON;

        let triggered = !has_changed;
        let message = if triggered {
            format!(
                "{} unchanged for {} seconds (value: {})",
                rule.metric_type, window_seconds, last_value
            )
        } else {
            format!(
                "{} changed from {} to {} in {} seconds",
                rule.metric_type, first_value, last_value, window_seconds
            )
        };

        info!(
            "Time window evaluation: rule={}, triggered={}, first={}, last={}",
            rule.name, triggered, first_value, last_value
        );

        Ok(EvaluationResult {
            triggered,
            current_value: last_value,
            threshold_value: None,
            message,
        })
    }

    /// 评估变化率条件
    fn evaluate_rate_of_change(
        rule: &AlertRule,
        metrics: &[MetricData],
    ) -> Result<EvaluationResult> {
        if metrics.len() < 2 {
            return Ok(EvaluationResult {
                triggered: false,
                current_value: 0.0,
                threshold_value: rule.threshold_value,
                message: "Not enough metrics to calculate rate of change".to_string(),
            });
        }

        let threshold =
            rule.threshold_value.ok_or_else(|| AppError::validation("Threshold value not set"))?;

        let window_seconds = rule.time_window_seconds; // 直接使用，不是 Option
        let now = Utc::now();
        let window_start = now - chrono::Duration::seconds(window_seconds as i64);

        // 获取时间窗口内的指标
        let window_metrics: Vec<_> =
            metrics.iter().filter(|m| m.collected_at >= window_start).collect();

        if window_metrics.len() < 2 {
            return Ok(EvaluationResult {
                triggered: false,
                current_value: 0.0,
                threshold_value: Some(threshold),
                message: "Not enough metrics in time window".to_string(),
            });
        }

        let first_value = window_metrics.first().unwrap().metric_value;
        let last_value = window_metrics.last().unwrap().metric_value;

        // 计算变化率（百分比）
        let rate_of_change = if first_value.abs() > f64::EPSILON {
            ((last_value - first_value) / first_value) * 100.0
        } else {
            0.0
        };

        // 使用默认的比较运算符（大于）
        // 因为 comparison_operator 字段已被删除
        let operator = ComparisonOperator::GreaterThan;

        let triggered = operator.evaluate(rate_of_change.abs(), threshold);

        let message = if triggered {
            format!(
                "{} changed by {:.2}% in {} seconds (from {} to {})",
                rule.metric_type, rate_of_change, window_seconds, first_value, last_value
            )
        } else {
            format!(
                "{} change rate {:.2}% is within threshold {}%",
                rule.metric_type, rate_of_change, threshold
            )
        };

        info!(
            "Rate of change evaluation: rule={}, triggered={}, rate={:.2}%, threshold={}%",
            rule.name, triggered, rate_of_change, threshold
        );

        Ok(EvaluationResult {
            triggered,
            current_value: rate_of_change,
            threshold_value: Some(threshold),
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::alert_rule::Severity;
    use crate::models::metric::MetricType;
    use chrono::DateTime;

    fn create_test_metrics(values: Vec<f64>, timestamps: Vec<DateTime<Utc>>) -> Vec<MetricData> {
        values
            .into_iter()
            .zip(timestamps)
            .map(|(value, timestamp)| MetricData {
                id: 0,
                node_id: 1,
                metric_type: MetricType::BlockHeight,
                metric_value: value,
                collected_at: timestamp,
                metadata: None,
            })
            .collect()
    }

    #[test]
    fn test_evaluate_threshold_triggered() {
        let rule = AlertRule::new(
            "Block height too low".to_string(),
            Some(1),
            "block_height".to_string(),
            ConditionType::Threshold,
            Severity::Critical,
            vec!["ops@example.com".to_string()],
        )
        .with_threshold(100000.0);

        let metrics = create_test_metrics(vec![150000.0], vec![Utc::now()]);

        let result = AlertEvaluator::evaluate(&rule, &metrics).unwrap();
        assert!(result.triggered);
        assert_eq!(result.current_value, 150000.0);
        assert_eq!(result.threshold_value, Some(100000.0));
    }

    #[test]
    fn test_evaluate_threshold_not_triggered() {
        let rule = AlertRule::new(
            "Block height too low".to_string(),
            Some(1),
            "block_height".to_string(),
            ConditionType::Threshold,
            Severity::Critical,
            vec!["ops@example.com".to_string()],
        )
        .with_threshold(100000.0);

        let metrics = create_test_metrics(vec![50000.0], vec![Utc::now()]);

        let result = AlertEvaluator::evaluate(&rule, &metrics).unwrap();
        assert!(!result.triggered);
        assert_eq!(result.current_value, 50000.0);
    }

    #[test]
    fn test_evaluate_time_window_no_change() {
        let now = Utc::now();
        let rule = AlertRule::new(
            "Block height not changing".to_string(),
            Some(1),
            "block_height".to_string(),
            ConditionType::TimeWindow,
            Severity::Critical,
            vec!["ops@example.com".to_string()],
        )
        .with_time_window(300); // 5 minutes

        // 创建5分钟内值不变的指标
        let metrics = create_test_metrics(
            vec![100000.0, 100000.0, 100000.0],
            vec![now - chrono::Duration::seconds(300), now - chrono::Duration::seconds(150), now],
        );

        let result = AlertEvaluator::evaluate(&rule, &metrics).unwrap();
        assert!(result.triggered);
    }

    #[test]
    fn test_evaluate_time_window_with_change() {
        let now = Utc::now();
        let rule = AlertRule::new(
            "Block height not changing".to_string(),
            Some(1),
            "block_height".to_string(),
            ConditionType::TimeWindow,
            Severity::Critical,
            vec!["ops@example.com".to_string()],
        )
        .with_time_window(300);

        // 创建5分钟内值有变化的指标
        let metrics = create_test_metrics(
            vec![100000.0, 100100.0, 100200.0],
            vec![now - chrono::Duration::seconds(300), now - chrono::Duration::seconds(150), now],
        );

        let result = AlertEvaluator::evaluate(&rule, &metrics).unwrap();
        assert!(!result.triggered);
    }

    #[test]
    fn test_evaluate_rate_of_change_triggered() {
        let now = Utc::now();
        let rule = AlertRule::new(
            "Block height dropping fast".to_string(),
            Some(1),
            "block_height".to_string(),
            ConditionType::RateOfChange,
            Severity::Warning,
            vec!["ops@example.com".to_string()],
        )
        .with_threshold(10.0) // 变化率 > 10%
        .with_time_window(600); // 10 分钟窗口

        // 创建变化率 > 10% 的指标（从100000降到80000，变化20%）
        let metrics = create_test_metrics(
            vec![100000.0, 80000.0],
            vec![
                now - chrono::Duration::seconds(500), // 在窗口内
                now - chrono::Duration::seconds(10),  // 在窗口内
            ],
        );

        let result = AlertEvaluator::evaluate(&rule, &metrics).unwrap();
        assert!(
            result.triggered,
            "Expected alert to be triggered, but it wasn't. Result: {:?}",
            result
        );
        assert!(result.current_value.abs() > 10.0); // 变化率的绝对值 > 10%
    }

    #[test]
    fn test_evaluate_empty_metrics() {
        let rule = AlertRule::new(
            "Test rule".to_string(),
            Some(1),
            "block_height".to_string(),
            ConditionType::Threshold,
            Severity::Critical,
            vec!["ops@example.com".to_string()],
        )
        .with_threshold(100000.0);

        let metrics = vec![];
        let result = AlertEvaluator::evaluate(&rule, &metrics).unwrap();
        assert!(!result.triggered);
    }
}

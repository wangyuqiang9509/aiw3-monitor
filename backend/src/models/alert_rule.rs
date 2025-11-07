use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 告警规则
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: i32,
    pub name: String,
    pub node_id: Option<i32>,
    pub metric_name: String,
    pub condition_type: String,
    pub threshold_value: Option<f64>,
    pub window_seconds: Option<i32>,
    pub comparison_operator: Option<String>,
    pub severity: String,
    pub email_recipients: Vec<String>,
    pub silence_period_seconds: i32,
    pub enabled: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 告警条件类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionType {
    /// 阈值条件 (如: block_height < 100000)
    Threshold,
    /// 时间窗口条件 (如: 10分钟内无变化)
    TimeWindow,
    /// 变化率条件 (如: 指标下降 > 10%)
    RateOfChange,
}

impl ConditionType {
    pub fn as_str(&self) -> &str {
        match self {
            ConditionType::Threshold => "threshold",
            ConditionType::TimeWindow => "time_window",
            ConditionType::RateOfChange => "rate_of_change",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "threshold" => Some(ConditionType::Threshold),
            "time_window" => Some(ConditionType::TimeWindow),
            "rate_of_change" => Some(ConditionType::RateOfChange),
            _ => None,
        }
    }
}

/// 告警级别
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

impl Severity {
    pub fn as_str(&self) -> &str {
        match self {
            Severity::Critical => "critical",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "critical" => Some(Severity::Critical),
            "warning" => Some(Severity::Warning),
            "info" => Some(Severity::Info),
            _ => None,
        }
    }
}

/// 比较运算符
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    Equal,
    GreaterThanOrEqual,
    LessThanOrEqual,
    NotEqual,
}

impl ComparisonOperator {
    pub fn as_str(&self) -> &str {
        match self {
            ComparisonOperator::GreaterThan => ">",
            ComparisonOperator::LessThan => "<",
            ComparisonOperator::Equal => "=",
            ComparisonOperator::GreaterThanOrEqual => ">=",
            ComparisonOperator::LessThanOrEqual => "<=",
            ComparisonOperator::NotEqual => "!=",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            ">" => Some(ComparisonOperator::GreaterThan),
            "<" => Some(ComparisonOperator::LessThan),
            "=" => Some(ComparisonOperator::Equal),
            ">=" => Some(ComparisonOperator::GreaterThanOrEqual),
            "<=" => Some(ComparisonOperator::LessThanOrEqual),
            "!=" => Some(ComparisonOperator::NotEqual),
            _ => None,
        }
    }

    /// 评估比较运算
    pub fn evaluate(&self, left: f64, right: f64) -> bool {
        match self {
            ComparisonOperator::GreaterThan => left > right,
            ComparisonOperator::LessThan => left < right,
            ComparisonOperator::Equal => (left - right).abs() < f64::EPSILON,
            ComparisonOperator::GreaterThanOrEqual => left >= right,
            ComparisonOperator::LessThanOrEqual => left <= right,
            ComparisonOperator::NotEqual => (left - right).abs() >= f64::EPSILON,
        }
    }
}

impl AlertRule {
    /// 创建新的告警规则
    pub fn new(
        name: String,
        node_id: Option<i32>,
        metric_name: String,
        condition_type: ConditionType,
        severity: Severity,
        email_recipients: Vec<String>,
    ) -> Self {
        Self {
            id: 0,
            name,
            node_id,
            metric_name,
            condition_type: condition_type.as_str().to_string(),
            threshold_value: None,
            window_seconds: None,
            comparison_operator: None,
            severity: severity.as_str().to_string(),
            email_recipients,
            silence_period_seconds: 1800, // 默认30分钟
            enabled: true,
            description: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// 设置阈值条件
    pub fn with_threshold(mut self, operator: ComparisonOperator, value: f64) -> Self {
        self.comparison_operator = Some(operator.as_str().to_string());
        self.threshold_value = Some(value);
        self
    }

    /// 设置时间窗口
    pub fn with_time_window(mut self, window_seconds: i32) -> Self {
        self.window_seconds = Some(window_seconds);
        self
    }

    /// 设置静默期
    pub fn with_silence_period(mut self, seconds: i32) -> Self {
        self.silence_period_seconds = seconds;
        self
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_type_conversion() {
        assert_eq!(ConditionType::Threshold.as_str(), "threshold");
        assert_eq!(ConditionType::TimeWindow.as_str(), "time_window");
        assert_eq!(ConditionType::RateOfChange.as_str(), "rate_of_change");

        assert_eq!(
            ConditionType::from_str("threshold"),
            Some(ConditionType::Threshold)
        );
        assert_eq!(
            ConditionType::from_str("time_window"),
            Some(ConditionType::TimeWindow)
        );
        assert_eq!(ConditionType::from_str("invalid"), None);
    }

    #[test]
    fn test_severity_conversion() {
        assert_eq!(Severity::Critical.as_str(), "critical");
        assert_eq!(Severity::Warning.as_str(), "warning");
        assert_eq!(Severity::Info.as_str(), "info");

        assert_eq!(Severity::from_str("critical"), Some(Severity::Critical));
        assert_eq!(Severity::from_str("warning"), Some(Severity::Warning));
        assert_eq!(Severity::from_str("invalid"), None);
    }

    #[test]
    fn test_comparison_operator_evaluate() {
        let gt = ComparisonOperator::GreaterThan;
        assert!(gt.evaluate(10.0, 5.0));
        assert!(!gt.evaluate(5.0, 10.0));

        let lt = ComparisonOperator::LessThan;
        assert!(lt.evaluate(5.0, 10.0));
        assert!(!lt.evaluate(10.0, 5.0));

        let eq = ComparisonOperator::Equal;
        assert!(eq.evaluate(10.0, 10.0));
        assert!(!eq.evaluate(10.0, 11.0));

        let gte = ComparisonOperator::GreaterThanOrEqual;
        assert!(gte.evaluate(10.0, 10.0));
        assert!(gte.evaluate(11.0, 10.0));
        assert!(!gte.evaluate(9.0, 10.0));
    }

    #[test]
    fn test_alert_rule_builder() {
        let rule = AlertRule::new(
            "Test Rule".to_string(),
            Some(1),
            "block_height".to_string(),
            ConditionType::Threshold,
            Severity::Critical,
            vec!["ops@example.com".to_string()],
        )
        .with_threshold(ComparisonOperator::LessThan, 100000.0)
        .with_silence_period(3600)
        .with_description("Test description".to_string());

        assert_eq!(rule.name, "Test Rule");
        assert_eq!(rule.node_id, Some(1));
        assert_eq!(rule.metric_name, "block_height");
        assert_eq!(rule.condition_type, "threshold");
        assert_eq!(rule.severity, "critical");
        assert_eq!(rule.threshold_value, Some(100000.0));
        assert_eq!(rule.comparison_operator, Some("<".to_string()));
        assert_eq!(rule.silence_period_seconds, 3600);
        assert_eq!(rule.description, Some("Test description".to_string()));
    }
}

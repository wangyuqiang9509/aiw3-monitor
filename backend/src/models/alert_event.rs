use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 告警事件记录
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AlertEvent {
    pub id: i64,
    pub rule_id: i32,
    pub node_id: Option<i32>,  // ✅ 修改为 Option (匹配数据库)
    pub status: String,
    pub severity: String,  // ✅ 添加 severity 字段 (数据库中存在)
    // ✅ 添加数据库中存在的字段
    pub title: String,
    pub message: String,
    // ✅ 修改: trigger_value -> metric_value (匹配数据库)
    pub metric_value: Option<f64>,
    pub threshold_value: Option<f64>,  // ✅ 添加 threshold_value (数据库中存在)
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub silenced_until: Option<DateTime<Utc>>,  // ✅ 添加 silenced_until (数据库中存在)
    pub notification_sent: bool,
    pub notification_sent_at: Option<DateTime<Utc>>,  // ✅ 添加 notification_sent_at (数据库中存在)
    // ✅ 修改: user_notes -> notes (匹配数据库)
    pub notes: Option<String>,
    // ✅ 添加 metadata 字段 (数据库中存在)
    #[sqlx(json)]
    pub metadata: serde_json::Value,
}

/// 告警状态 (匹配数据库 alert_status 枚举)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertStatus {
    Triggered,  // ✅ 修改: Active -> Triggered (匹配数据库)
    Resolved,
    Silenced,   // ✅ 添加 Silenced 状态 (数据库中存在)
}

impl AlertStatus {
    pub fn as_str(&self) -> &str {
        match self {
            AlertStatus::Triggered => "triggered",  // ✅ 修改
            AlertStatus::Resolved => "resolved",
            AlertStatus::Silenced => "silenced",    // ✅ 添加
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "triggered" => Some(AlertStatus::Triggered),  // ✅ 修改
            "resolved" => Some(AlertStatus::Resolved),
            "silenced" => Some(AlertStatus::Silenced),    // ✅ 添加
            _ => None,
        }
    }
}

impl AlertEvent {
    /// 创建新的告警事件
    pub fn new(
        rule_id: i32,
        node_id: Option<i32>,  // ✅ 修改为 Option
        severity: String,
        title: String,
        message: String,
        metric_value: Option<f64>,  // ✅ 使用新字段名
        threshold_value: Option<f64>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            rule_id,
            node_id,
            status: AlertStatus::Triggered.as_str().to_string(),  // ✅ 使用 Triggered
            severity,
            title,
            message,
            metric_value,  // ✅ 使用新字段名
            threshold_value,
            triggered_at: now,
            resolved_at: None,
            silenced_until: None,
            notification_sent: false,
            notification_sent_at: None,
            notes: None,  // ✅ 使用新字段名
            metadata: serde_json::json!({}),
        }
    }

    /// 标记为已发送通知
    pub fn mark_notification_sent(&mut self) {
        self.notification_sent = true;
        self.notification_sent_at = Some(Utc::now());  // ✅ 设置发送时间
    }

    /// 解决告警
    pub fn resolve(&mut self) {
        self.status = AlertStatus::Resolved.as_str().to_string();
        self.resolved_at = Some(Utc::now());
    }

    /// 静默告警
    pub fn silence(&mut self, until: DateTime<Utc>) {
        self.status = AlertStatus::Silenced.as_str().to_string();
        self.silenced_until = Some(until);
    }

    /// 添加备注
    pub fn add_note(&mut self, note: String) {
        self.notes = Some(note);  // ✅ 使用新字段名
    }

    /// 检查是否已解决
    pub fn is_resolved(&self) -> bool {
        self.status == AlertStatus::Resolved.as_str()
    }

    /// 检查是否已触发
    pub fn is_triggered(&self) -> bool {
        self.status == AlertStatus::Triggered.as_str()  // ✅ 使用 Triggered
    }

    /// 检查是否已静默
    pub fn is_silenced(&self) -> bool {
        self.status == AlertStatus::Silenced.as_str()
    }

    /// 获取告警持续时间（秒）
    pub fn duration_seconds(&self) -> i64 {
        let end_time = self.resolved_at.unwrap_or_else(Utc::now);
        (end_time - self.triggered_at).num_seconds()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_status_conversion() {
        assert_eq!(AlertStatus::Triggered.as_str(), "triggered");  // ✅ 修改
        assert_eq!(AlertStatus::Resolved.as_str(), "resolved");
        assert_eq!(AlertStatus::Silenced.as_str(), "silenced");  // ✅ 添加

        assert_eq!(AlertStatus::from_str("triggered"), Some(AlertStatus::Triggered));  // ✅ 修改
        assert_eq!(
            AlertStatus::from_str("resolved"),
            Some(AlertStatus::Resolved)
        );
        assert_eq!(AlertStatus::from_str("silenced"), Some(AlertStatus::Silenced));  // ✅ 添加
        assert_eq!(AlertStatus::from_str("invalid"), None);
    }

    #[test]
    fn test_alert_event_creation() {
        let event = AlertEvent::new(
            1,
            Some(2),  // ✅ 使用 Option
            "critical".to_string(),
            "Test Alert".to_string(),
            "Test message".to_string(),
            Some(100.0),  // ✅ 使用新字段名
            Some(50.0),
        );

        assert_eq!(event.rule_id, 1);
        assert_eq!(event.node_id, Some(2));
        assert_eq!(event.severity, "critical");
        assert_eq!(event.title, "Test Alert");
        assert_eq!(event.message, "Test message");
        assert_eq!(event.metric_value, Some(100.0));  // ✅ 使用新字段名
        assert_eq!(event.threshold_value, Some(50.0));
        assert_eq!(event.status, "triggered");  // ✅ 修改
        assert!(!event.notification_sent);
        assert!(event.resolved_at.is_none());
    }

    #[test]
    fn test_mark_notification_sent() {
        let mut event = AlertEvent::new(
            1,
            Some(2),
            "warning".to_string(),
            "Test".to_string(),
            "Message".to_string(),
            Some(100.0),
            Some(50.0),
        );
        assert!(!event.notification_sent);
        assert!(event.notification_sent_at.is_none());  // ✅ 添加

        event.mark_notification_sent();
        assert!(event.notification_sent);
        assert!(event.notification_sent_at.is_some());  // ✅ 添加
    }

    #[test]
    fn test_resolve_alert() {
        let mut event = AlertEvent::new(
            1,
            Some(2),
            "warning".to_string(),
            "Test".to_string(),
            "Message".to_string(),
            Some(100.0),
            Some(50.0),
        );
        assert!(event.is_triggered());  // ✅ 修改方法名
        assert!(!event.is_resolved());
        assert!(event.resolved_at.is_none());

        event.resolve();
        assert!(!event.is_triggered());  // ✅ 修改方法名
        assert!(event.is_resolved());
        assert!(event.resolved_at.is_some());
    }

    #[test]
    fn test_silence_alert() {  // ✅ 新增测试
        let mut event = AlertEvent::new(
            1,
            Some(2),
            "warning".to_string(),
            "Test".to_string(),
            "Message".to_string(),
            Some(100.0),
            Some(50.0),
        );
        assert!(!event.is_silenced());

        let until = Utc::now() + chrono::Duration::hours(1);
        event.silence(until);
        assert!(event.is_silenced());
        assert_eq!(event.silenced_until, Some(until));
    }

    #[test]
    fn test_add_note() {
        let mut event = AlertEvent::new(
            1,
            Some(2),
            "info".to_string(),
            "Test".to_string(),
            "Message".to_string(),
            Some(100.0),
            Some(50.0),
        );
        assert!(event.notes.is_none());  // ✅ 使用新字段名

        event.add_note("Investigated and fixed".to_string());
        assert_eq!(
            event.notes,  // ✅ 使用新字段名
            Some("Investigated and fixed".to_string())
        );
    }

    #[test]
    fn test_duration_seconds() {
        let mut event = AlertEvent::new(
            1,
            Some(2),
            "warning".to_string(),
            "Test".to_string(),
            "Message".to_string(),
            Some(100.0),
            Some(50.0),
        );
        
        // 触发状态的持续时间应该是从触发到现在
        let duration = event.duration_seconds();
        assert!(duration >= 0);

        // 解决后的持续时间应该是固定的
        event.resolve();
        let resolved_duration = event.duration_seconds();
        assert!(resolved_duration >= 0);
    }
}

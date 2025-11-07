use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 告警事件记录
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AlertEvent {
    pub id: i64,
    pub rule_id: i32,
    pub node_id: i32,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub trigger_value: f64,
    pub status: String,
    pub notification_sent: bool,
    pub notification_error: Option<String>,
    pub user_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 告警状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertStatus {
    Active,
    Resolved,
}

impl AlertStatus {
    pub fn as_str(&self) -> &str {
        match self {
            AlertStatus::Active => "active",
            AlertStatus::Resolved => "resolved",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "active" => Some(AlertStatus::Active),
            "resolved" => Some(AlertStatus::Resolved),
            _ => None,
        }
    }
}

impl AlertEvent {
    /// 创建新的告警事件
    pub fn new(rule_id: i32, node_id: i32, trigger_value: f64) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            rule_id,
            node_id,
            triggered_at: now,
            resolved_at: None,
            trigger_value,
            status: AlertStatus::Active.as_str().to_string(),
            notification_sent: false,
            notification_error: None,
            user_notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 标记为已发送通知
    pub fn mark_notification_sent(&mut self) {
        self.notification_sent = true;
        self.updated_at = Utc::now();
    }

    /// 标记通知发送失败
    pub fn mark_notification_failed(&mut self, error: String) {
        self.notification_error = Some(error);
        self.updated_at = Utc::now();
    }

    /// 解决告警
    pub fn resolve(&mut self) {
        self.status = AlertStatus::Resolved.as_str().to_string();
        self.resolved_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// 添加用户备注
    pub fn add_note(&mut self, note: String) {
        self.user_notes = Some(note);
        self.updated_at = Utc::now();
    }

    /// 检查是否已解决
    pub fn is_resolved(&self) -> bool {
        self.status == AlertStatus::Resolved.as_str()
    }

    /// 检查是否活跃
    pub fn is_active(&self) -> bool {
        self.status == AlertStatus::Active.as_str()
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
        assert_eq!(AlertStatus::Active.as_str(), "active");
        assert_eq!(AlertStatus::Resolved.as_str(), "resolved");

        assert_eq!(AlertStatus::from_str("active"), Some(AlertStatus::Active));
        assert_eq!(
            AlertStatus::from_str("resolved"),
            Some(AlertStatus::Resolved)
        );
        assert_eq!(AlertStatus::from_str("invalid"), None);
    }

    #[test]
    fn test_alert_event_creation() {
        let event = AlertEvent::new(1, 2, 100.0);

        assert_eq!(event.rule_id, 1);
        assert_eq!(event.node_id, 2);
        assert_eq!(event.trigger_value, 100.0);
        assert_eq!(event.status, "active");
        assert!(!event.notification_sent);
        assert!(event.notification_error.is_none());
        assert!(event.resolved_at.is_none());
    }

    #[test]
    fn test_mark_notification_sent() {
        let mut event = AlertEvent::new(1, 2, 100.0);
        assert!(!event.notification_sent);

        event.mark_notification_sent();
        assert!(event.notification_sent);
    }

    #[test]
    fn test_mark_notification_failed() {
        let mut event = AlertEvent::new(1, 2, 100.0);
        assert!(event.notification_error.is_none());

        event.mark_notification_failed("SMTP connection failed".to_string());
        assert_eq!(
            event.notification_error,
            Some("SMTP connection failed".to_string())
        );
    }

    #[test]
    fn test_resolve_alert() {
        let mut event = AlertEvent::new(1, 2, 100.0);
        assert!(event.is_active());
        assert!(!event.is_resolved());
        assert!(event.resolved_at.is_none());

        event.resolve();
        assert!(!event.is_active());
        assert!(event.is_resolved());
        assert!(event.resolved_at.is_some());
    }

    #[test]
    fn test_add_note() {
        let mut event = AlertEvent::new(1, 2, 100.0);
        assert!(event.user_notes.is_none());

        event.add_note("Investigated and fixed".to_string());
        assert_eq!(
            event.user_notes,
            Some("Investigated and fixed".to_string())
        );
    }

    #[test]
    fn test_duration_seconds() {
        let mut event = AlertEvent::new(1, 2, 100.0);
        
        // 活跃告警的持续时间应该是从触发到现在
        let duration = event.duration_seconds();
        assert!(duration >= 0);

        // 解决后的持续时间应该是固定的
        event.resolve();
        let resolved_duration = event.duration_seconds();
        assert!(resolved_duration >= 0);
    }
}

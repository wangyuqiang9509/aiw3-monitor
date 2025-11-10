use crate::models::alert_event::AlertEvent;
use crate::models::alert_rule::AlertRule;
use crate::models::node::BlockchainNode;

/// 邮件模板生成器
pub struct EmailTemplates;

impl EmailTemplates {
    /// 生成纯文本邮件内容
    pub fn generate_text(
        alert: &AlertEvent,
        rule: &AlertRule,
        node: Option<&BlockchainNode>,
    ) -> String {
        let node_name = node.map(|n| n.name.as_str()).unwrap_or("Unknown");
        let severity_emoji = match rule.severity.as_str() {
            "critical" => "🚨",
            "warning" => "⚠️",
            "info" => "ℹ️",
            _ => "📢",
        };

        format!(
            r#"{} 区块链监控告警

告警规则: {}
告警级别: {}
节点名称: {}
触发时间: {}
触发值: {}

描述:
{}

---
此邮件由 AIWS 区块链监控系统自动发送
"#,
            severity_emoji,
            rule.name,
            rule.severity.to_uppercase(),
            node_name,
            alert.triggered_at.format("%Y-%m-%d %H:%M:%S UTC"),
            alert.metric_value.unwrap_or(0.0),
            rule.description.as_deref().unwrap_or("无描述")
        )
    }

    /// 生成 HTML 邮件内容
    pub fn generate_html(
        alert: &AlertEvent,
        rule: &AlertRule,
        node: Option<&BlockchainNode>,
    ) -> String {
        let node_name = node.map(|n| n.name.as_str()).unwrap_or("Unknown");
        let (severity_color, severity_emoji) = match rule.severity.as_str() {
            "critical" => ("#d32f2f", "🚨"),
            "warning" => ("#f57c00", "⚠️"),
            "info" => ("#1976d2", "ℹ️"),
            _ => ("#757575", "📢"),
        };

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
        }}
        .header {{
            background-color: {severity_color};
            color: white;
            padding: 20px;
            border-radius: 8px 8px 0 0;
            text-align: center;
        }}
        .header h1 {{
            margin: 0;
            font-size: 24px;
        }}
        .content {{
            background-color: #f5f5f5;
            padding: 20px;
            border-radius: 0 0 8px 8px;
        }}
        .info-row {{
            margin: 10px 0;
            padding: 10px;
            background-color: white;
            border-radius: 4px;
        }}
        .info-label {{
            font-weight: bold;
            color: #666;
            display: inline-block;
            width: 120px;
        }}
        .info-value {{
            color: #333;
        }}
        .description {{
            margin-top: 20px;
            padding: 15px;
            background-color: white;
            border-left: 4px solid {severity_color};
            border-radius: 4px;
        }}
        .footer {{
            margin-top: 20px;
            padding-top: 20px;
            border-top: 1px solid #ddd;
            text-align: center;
            color: #999;
            font-size: 12px;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>{severity_emoji} 区块链监控告警</h1>
    </div>
    <div class="content">
        <div class="info-row">
            <span class="info-label">告警规则:</span>
            <span class="info-value">{rule_name}</span>
        </div>
        <div class="info-row">
            <span class="info-label">告警级别:</span>
            <span class="info-value" style="color: {severity_color}; font-weight: bold;">{severity}</span>
        </div>
        <div class="info-row">
            <span class="info-label">节点名称:</span>
            <span class="info-value">{node_name}</span>
        </div>
        <div class="info-row">
            <span class="info-label">触发时间:</span>
            <span class="info-value">{triggered_at}</span>
        </div>
        <div class="info-row">
            <span class="info-label">触发值:</span>
            <span class="info-value">{trigger_value}</span>
        </div>
        <div class="description">
            <strong>描述:</strong><br>
            {description}
        </div>
    </div>
    <div class="footer">
        此邮件由 AIWS 区块链监控系统自动发送<br>
        请勿回复此邮件
    </div>
</body>
</html>"#,
            severity_color = severity_color,
            severity_emoji = severity_emoji,
            rule_name = rule.name,
            severity = rule.severity.to_uppercase(),
            node_name = node_name,
            triggered_at = alert.triggered_at.format("%Y-%m-%d %H:%M:%S UTC"),
            trigger_value = alert.metric_value.unwrap_or(0.0),
            description = rule.description.as_deref().unwrap_or("无描述")
        )
    }

    /// 生成告警恢复邮件（纯文本）
    pub fn generate_resolution_text(
        alert: &AlertEvent,
        rule: &AlertRule,
        node: Option<&BlockchainNode>,
    ) -> String {
        let node_name = node.map(|n| n.name.as_str()).unwrap_or("Unknown");
        let duration = alert.duration_seconds();

        format!(
            r#"✅ 告警已恢复

告警规则: {}
节点名称: {}
触发时间: {}
恢复时间: {}
持续时间: {} 秒

---
此邮件由 AIWS 区块链监控系统自动发送
"#,
            rule.name,
            node_name,
            alert.triggered_at.format("%Y-%m-%d %H:%M:%S UTC"),
            alert.resolved_at.unwrap_or(alert.triggered_at).format("%Y-%m-%d %H:%M:%S UTC"),
            duration
        )
    }

    /// 生成告警恢复邮件（HTML）
    pub fn generate_resolution_html(
        alert: &AlertEvent,
        rule: &AlertRule,
        node: Option<&BlockchainNode>,
    ) -> String {
        let node_name = node.map(|n| n.name.as_str()).unwrap_or("Unknown");
        let duration = alert.duration_seconds();

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
        }}
        .header {{
            background-color: #4caf50;
            color: white;
            padding: 20px;
            border-radius: 8px 8px 0 0;
            text-align: center;
        }}
        .header h1 {{
            margin: 0;
            font-size: 24px;
        }}
        .content {{
            background-color: #f5f5f5;
            padding: 20px;
            border-radius: 0 0 8px 8px;
        }}
        .info-row {{
            margin: 10px 0;
            padding: 10px;
            background-color: white;
            border-radius: 4px;
        }}
        .info-label {{
            font-weight: bold;
            color: #666;
            display: inline-block;
            width: 120px;
        }}
        .info-value {{
            color: #333;
        }}
        .footer {{
            margin-top: 20px;
            padding-top: 20px;
            border-top: 1px solid #ddd;
            text-align: center;
            color: #999;
            font-size: 12px;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>✅ 告警已恢复</h1>
    </div>
    <div class="content">
        <div class="info-row">
            <span class="info-label">告警规则:</span>
            <span class="info-value">{rule_name}</span>
        </div>
        <div class="info-row">
            <span class="info-label">节点名称:</span>
            <span class="info-value">{node_name}</span>
        </div>
        <div class="info-row">
            <span class="info-label">触发时间:</span>
            <span class="info-value">{triggered_at}</span>
        </div>
        <div class="info-row">
            <span class="info-label">恢复时间:</span>
            <span class="info-value">{resolved_at}</span>
        </div>
        <div class="info-row">
            <span class="info-label">持续时间:</span>
            <span class="info-value">{duration} 秒</span>
        </div>
    </div>
    <div class="footer">
        此邮件由 AIWS 区块链监控系统自动发送<br>
        请勿回复此邮件
    </div>
</body>
</html>"#,
            rule_name = rule.name,
            node_name = node_name,
            triggered_at = alert.triggered_at.format("%Y-%m-%d %H:%M:%S UTC"),
            resolved_at =
                alert.resolved_at.unwrap_or(alert.triggered_at).format("%Y-%m-%d %H:%M:%S UTC"),
            duration = duration
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::alert_rule::{ConditionType, Severity};
    use chrono::Utc;

    fn create_test_alert() -> AlertEvent {
        AlertEvent::new(
            1,                                             // rule_id
            Some(1),                                       // node_id
            "critical".to_string(),                        // severity
            "Block height too low".to_string(),            // title
            "Block height is below threshold".to_string(), // message
            Some(99000.0),                                 // metric_value
            Some(100000.0),                                // threshold_value
        )
    }

    fn create_test_rule() -> AlertRule {
        AlertRule::new(
            "Block height too low".to_string(),
            Some(1),
            "block_height".to_string(),
            ConditionType::Threshold,
            Severity::Critical,
            vec!["ops@example.com".to_string()],
        )
        .with_description("Block height is below threshold".to_string())
    }

    fn create_test_node() -> BlockchainNode {
        use std::collections::HashMap;
        BlockchainNode {
            id: 1,
            name: "AIWS DevNet 3".to_string(),
            rpc_url: "https://devnet-rpc3.aiw3.io".to_string(),
            rest_url: "https://devnet-api3.aiw3.io".to_string(),
            grpc_url: "https://devnet-grpc3.aiw3.io:443".to_string(),
            environment: "devnet".to_string(),
            enabled: true,
            labels: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_generate_text_email() {
        let alert = create_test_alert();
        let rule = create_test_rule();
        let node = create_test_node();

        let text = EmailTemplates::generate_text(&alert, &rule, Some(&node));

        assert!(text.contains("区块链监控告警"));
        assert!(text.contains("Block height too low"));
        assert!(text.contains("CRITICAL"));
        assert!(text.contains("AIWS DevNet 3"));
        assert!(text.contains("99000"));
    }

    #[test]
    fn test_generate_html_email() {
        let alert = create_test_alert();
        let rule = create_test_rule();
        let node = create_test_node();

        let html = EmailTemplates::generate_html(&alert, &rule, Some(&node));

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("区块链监控告警"));
        assert!(html.contains("Block height too low"));
        assert!(html.contains("CRITICAL"));
        assert!(html.contains("AIWS DevNet 3"));
        assert!(html.contains("99000"));
        assert!(html.contains("#d32f2f")); // Critical color
    }

    #[test]
    fn test_generate_text_email_without_node() {
        let alert = create_test_alert();
        let rule = create_test_rule();

        let text = EmailTemplates::generate_text(&alert, &rule, None);

        assert!(text.contains("Unknown"));
    }

    #[test]
    fn test_severity_colors() {
        let alert = create_test_alert();
        let node = create_test_node();

        // Test critical
        let mut rule = create_test_rule();
        let html = EmailTemplates::generate_html(&alert, &rule, Some(&node));
        assert!(html.contains("#d32f2f"));

        // Test warning
        rule.severity = "warning".to_string();
        let html = EmailTemplates::generate_html(&alert, &rule, Some(&node));
        assert!(html.contains("#f57c00"));

        // Test info
        rule.severity = "info".to_string();
        let html = EmailTemplates::generate_html(&alert, &rule, Some(&node));
        assert!(html.contains("#1976d2"));
    }

    #[test]
    fn test_generate_resolution_text() {
        let mut alert = create_test_alert();
        alert.resolve();
        let rule = create_test_rule();
        let node = create_test_node();

        let text = EmailTemplates::generate_resolution_text(&alert, &rule, Some(&node));

        assert!(text.contains("告警已恢复"));
        assert!(text.contains("Block height too low"));
        assert!(text.contains("AIWS DevNet 3"));
        assert!(text.contains("持续时间"));
    }

    #[test]
    fn test_generate_resolution_html() {
        let mut alert = create_test_alert();
        alert.resolve();
        let rule = create_test_rule();
        let node = create_test_node();

        let html = EmailTemplates::generate_resolution_html(&alert, &rule, Some(&node));

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("告警已恢复"));
        assert!(html.contains("Block height too low"));
        assert!(html.contains("AIWS DevNet 3"));
        assert!(html.contains("#4caf50")); // Success color
    }
}

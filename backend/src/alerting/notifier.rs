use crate::error::{AppError, Result};
use crate::models::alert_event::AlertEvent;
use crate::models::alert_rule::AlertRule;
use crate::models::node::BlockchainNode;
use crate::models::smtp_config::SmtpConfig;
use lettre::message::{header, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use tracing::{debug, error, info, warn};

use super::templates::EmailTemplates;

/// 邮件通知发送器
pub struct EmailNotifier {
    smtp_config: SmtpConfig,
}

impl EmailNotifier {
    /// 创建新的邮件通知发送器
    pub fn new(smtp_config: SmtpConfig) -> Result<Self> {
        // 验证 SMTP 配置
        smtp_config.validate().map_err(AppError::validation)?;
        
        Ok(Self { smtp_config })
    }

    /// 发送告警通知邮件
    pub async fn send_alert(
        &self,
        alert: &AlertEvent,
        rule: &AlertRule,
        node: Option<&BlockchainNode>,
    ) -> Result<()> {
        info!(
            "Sending alert notification: rule={}, node_id={:?}",
            rule.name, alert.node_id
        );

        // 生成邮件内容
        let text_body = EmailTemplates::generate_text(alert, rule, node);
        let html_body = EmailTemplates::generate_html(alert, rule, node);

        // 构建邮件
        let subject = format!(
            "[{}] {} - {}",
            rule.severity.to_uppercase(),
            rule.name,
            node.map(|n| n.name.as_str()).unwrap_or("Unknown Node")
        );

        // 发送给所有收件人
        for recipient in &rule.notification_channels.0 {
            match self
                .send_email(recipient, &subject, &text_body, &html_body)
                .await
            {
                Ok(_) => {
                    info!("Alert notification sent to: {}", recipient);
                }
                Err(e) => {
                    error!("Failed to send alert notification to {}: {}", recipient, e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// 发送告警恢复通知
    pub async fn send_resolution(
        &self,
        alert: &AlertEvent,
        rule: &AlertRule,
        node: Option<&BlockchainNode>,
    ) -> Result<()> {
        info!(
            "Sending resolution notification: rule={}, node_id={:?}",
            rule.name, alert.node_id
        );

        // 生成邮件内容
        let text_body = EmailTemplates::generate_resolution_text(alert, rule, node);
        let html_body = EmailTemplates::generate_resolution_html(alert, rule, node);

        // 构建邮件
        let subject = format!(
            "[RESOLVED] {} - {}",
            rule.name,
            node.map(|n| n.name.as_str()).unwrap_or("Unknown Node")
        );

        // 发送给所有收件人
        for recipient in &rule.notification_channels.0 {
            match self
                .send_email(recipient, &subject, &text_body, &html_body)
                .await
            {
                Ok(_) => {
                    info!("Resolution notification sent to: {}", recipient);
                }
                Err(e) => {
                    error!("Failed to send resolution notification to {}: {}", recipient, e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// 发送测试邮件
    pub async fn send_test_email(&self, recipient: &str) -> Result<()> {
        info!("Sending test email to: {}", recipient);

        let subject = "AIWS Monitor - Test Email";
        let text_body = r#"
This is a test email from AIWS Blockchain Monitor.

If you received this email, your SMTP configuration is working correctly.

---
AIWS Blockchain Monitor
"#;

        let html_body = r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {
            font-family: Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 600px;
            margin: 0 auto;
            padding: 20px;
        }
        .header {
            background-color: #1976d2;
            color: white;
            padding: 20px;
            border-radius: 8px;
            text-align: center;
        }
        .content {
            padding: 20px;
            background-color: #f5f5f5;
            border-radius: 8px;
            margin-top: 20px;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>✅ Test Email</h1>
    </div>
    <div class="content">
        <p>This is a test email from <strong>AIWS Blockchain Monitor</strong>.</p>
        <p>If you received this email, your SMTP configuration is working correctly.</p>
    </div>
</body>
</html>
"#;

        self.send_email(recipient, subject, text_body, html_body)
            .await
    }

    /// 发送单个邮件
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        text_body: &str,
        html_body: &str,
    ) -> Result<()> {
        debug!(
            "Building email: to={}, subject={}, smtp={}",
            to,
            subject,
            self.smtp_config.connection_string()
        );

        // 构建邮件
        let email = Message::builder()
            .from(
                self.smtp_config
                    .from_address
                    .parse()
                    .map_err(|e| AppError::validation(format!("Invalid from address: {}", e)))?,
            )
            .to(to
                .parse()
                .map_err(|e| AppError::validation(format!("Invalid recipient address: {}", e)))?)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN)
                            .body(text_body.to_string()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML)
                            .body(html_body.to_string()),
                    ),
            )
            .map_err(|e| AppError::validation(format!("Failed to build email: {}", e)))?;

        // 创建 SMTP 传输
        let creds = Credentials::new(
            self.smtp_config.username.clone(),
            self.smtp_config.password.clone(),
        );

        let mailer = if self.smtp_config.use_tls {
            SmtpTransport::relay(&self.smtp_config.server)
                .map_err(|e| AppError::validation(format!("Invalid SMTP server: {}", e)))?
                .credentials(creds)
                .port(self.smtp_config.port as u16)
                .build()
        } else {
            SmtpTransport::builder_dangerous(&self.smtp_config.server)
                .credentials(creds)
                .port(self.smtp_config.port as u16)
                .build()
        };

        // 发送邮件
        mailer
            .send(&email)
            .map_err(|e| AppError::validation(format!("Failed to send email: {}", e)))?;

        debug!("Email sent successfully to: {}", to);
        Ok(())
    }
}

/// 带重试的邮件发送器
pub struct RetryableEmailNotifier {
    notifier: EmailNotifier,
    max_retries: u32,
}

impl RetryableEmailNotifier {
    /// 创建带重试功能的邮件发送器
    pub fn new(smtp_config: SmtpConfig, max_retries: u32) -> Result<Self> {
        let notifier = EmailNotifier::new(smtp_config)?;
        Ok(Self {
            notifier,
            max_retries,
        })
    }

    /// 发送告警通知（带重试）
    pub async fn send_alert(
        &self,
        alert: &AlertEvent,
        rule: &AlertRule,
        node: Option<&BlockchainNode>,
    ) -> Result<()> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.max_retries {
            attempts += 1;

            match self.notifier.send_alert(alert, rule, node).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    warn!(
                        "Failed to send alert notification (attempt {}/{}): {}",
                        attempts, self.max_retries, e
                    );
                    last_error = Some(e);

                    if attempts < self.max_retries {
                        // 指数退避
                        let delay = std::time::Duration::from_secs(2u64.pow(attempts));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AppError::validation("Failed to send email after retries")))
    }

    /// 发送恢复通知（带重试）
    pub async fn send_resolution(
        &self,
        alert: &AlertEvent,
        rule: &AlertRule,
        node: Option<&BlockchainNode>,
    ) -> Result<()> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.max_retries {
            attempts += 1;

            match self.notifier.send_resolution(alert, rule, node).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    warn!(
                        "Failed to send resolution notification (attempt {}/{}): {}",
                        attempts, self.max_retries, e
                    );
                    last_error = Some(e);

                    if attempts < self.max_retries {
                        let delay = std::time::Duration::from_secs(2u64.pow(attempts));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AppError::validation("Failed to send email after retries")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_smtp_config() -> SmtpConfig {
        SmtpConfig::new(
            "Test SMTP".to_string(),
            "smtp.example.com".to_string(),
            587,
            "test@example.com".to_string(),
            "password".to_string(),
            "alerts@example.com".to_string(),
        )
    }

    #[test]
    fn test_email_notifier_creation() {
        let config = create_test_smtp_config();
        let notifier = EmailNotifier::new(config);
        assert!(notifier.is_ok());
    }

    #[test]
    fn test_email_notifier_invalid_config() {
        let mut config = create_test_smtp_config();
        config.server = "".to_string();
        let notifier = EmailNotifier::new(config);
        assert!(notifier.is_err());
    }

    #[test]
    fn test_retryable_notifier_creation() {
        let config = create_test_smtp_config();
        let notifier = RetryableEmailNotifier::new(config, 3);
        assert!(notifier.is_ok());
    }

    // 注意：实际的邮件发送测试需要真实的 SMTP 服务器
    // 这些测试在集成测试中使用 mock SMTP 服务器进行
}

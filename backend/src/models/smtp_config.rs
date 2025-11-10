use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// SMTP 配置
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub id: i32,
    pub name: String,
    pub server: String,
    pub port: i32,
    pub username: String,
    #[serde(skip_serializing)] // 不序列化密码
    pub password: String,
    pub from_address: String,
    pub use_tls: bool,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SmtpConfig {
    /// 创建新的 SMTP 配置
    pub fn new(
        name: String,
        server: String,
        port: i32,
        username: String,
        password: String,
        from_address: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            name,
            server,
            port,
            username,
            password,
            from_address,
            use_tls: true,
            is_default: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// 设置为默认配置
    pub fn set_as_default(mut self) -> Self {
        self.is_default = true;
        self
    }

    /// 禁用 TLS
    pub fn without_tls(mut self) -> Self {
        self.use_tls = false;
        self
    }

    /// 验证配置是否完整
    pub fn validate(&self) -> Result<(), String> {
        if self.server.is_empty() {
            return Err("SMTP server cannot be empty".to_string());
        }
        if self.port <= 0 || self.port > 65535 {
            return Err(format!("Invalid port: {}", self.port));
        }
        if self.username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if self.password.is_empty() {
            return Err("Password cannot be empty".to_string());
        }
        if self.from_address.is_empty() {
            return Err("From address cannot be empty".to_string());
        }
        if !self.from_address.contains('@') {
            return Err("Invalid from address format".to_string());
        }
        Ok(())
    }

    /// 获取连接字符串（用于日志，不包含密码）
    pub fn connection_string(&self) -> String {
        format!("{}@{}:{} (TLS: {})", self.username, self.server, self.port, self.use_tls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smtp_config_creation() {
        let config = SmtpConfig::new(
            "Gmail".to_string(),
            "smtp.gmail.com".to_string(),
            587,
            "user@gmail.com".to_string(),
            "password".to_string(),
            "alerts@example.com".to_string(),
        );

        assert_eq!(config.name, "Gmail");
        assert_eq!(config.server, "smtp.gmail.com");
        assert_eq!(config.port, 587);
        assert_eq!(config.username, "user@gmail.com");
        assert_eq!(config.password, "password");
        assert_eq!(config.from_address, "alerts@example.com");
        assert!(config.use_tls);
        assert!(!config.is_default);
    }

    #[test]
    fn test_set_as_default() {
        let config = SmtpConfig::new(
            "Gmail".to_string(),
            "smtp.gmail.com".to_string(),
            587,
            "user@gmail.com".to_string(),
            "password".to_string(),
            "alerts@example.com".to_string(),
        )
        .set_as_default();

        assert!(config.is_default);
    }

    #[test]
    fn test_without_tls() {
        let config = SmtpConfig::new(
            "Local".to_string(),
            "localhost".to_string(),
            25,
            "user".to_string(),
            "password".to_string(),
            "alerts@localhost".to_string(),
        )
        .without_tls();

        assert!(!config.use_tls);
    }

    #[test]
    fn test_validate_success() {
        let config = SmtpConfig::new(
            "Gmail".to_string(),
            "smtp.gmail.com".to_string(),
            587,
            "user@gmail.com".to_string(),
            "password".to_string(),
            "alerts@example.com".to_string(),
        );

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_server() {
        let config = SmtpConfig::new(
            "Test".to_string(),
            "".to_string(),
            587,
            "user".to_string(),
            "password".to_string(),
            "alerts@example.com".to_string(),
        );

        assert!(config.validate().is_err());
        assert_eq!(config.validate().unwrap_err(), "SMTP server cannot be empty");
    }

    #[test]
    fn test_validate_invalid_port() {
        let config = SmtpConfig::new(
            "Test".to_string(),
            "smtp.example.com".to_string(),
            70000,
            "user".to_string(),
            "password".to_string(),
            "alerts@example.com".to_string(),
        );

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_empty_username() {
        let config = SmtpConfig::new(
            "Test".to_string(),
            "smtp.example.com".to_string(),
            587,
            "".to_string(),
            "password".to_string(),
            "alerts@example.com".to_string(),
        );

        assert!(config.validate().is_err());
        assert_eq!(config.validate().unwrap_err(), "Username cannot be empty");
    }

    #[test]
    fn test_validate_invalid_from_address() {
        let config = SmtpConfig::new(
            "Test".to_string(),
            "smtp.example.com".to_string(),
            587,
            "user".to_string(),
            "password".to_string(),
            "invalid-email".to_string(),
        );

        assert!(config.validate().is_err());
        assert_eq!(config.validate().unwrap_err(), "Invalid from address format");
    }

    #[test]
    fn test_connection_string() {
        let config = SmtpConfig::new(
            "Gmail".to_string(),
            "smtp.gmail.com".to_string(),
            587,
            "user@gmail.com".to_string(),
            "password".to_string(),
            "alerts@example.com".to_string(),
        );

        let conn_str = config.connection_string();
        assert!(conn_str.contains("user@gmail.com"));
        assert!(conn_str.contains("smtp.gmail.com"));
        assert!(conn_str.contains("587"));
        assert!(!conn_str.contains("password")); // 密码不应该出现在连接字符串中
    }

    #[test]
    fn test_password_not_serialized() {
        let config = SmtpConfig::new(
            "Gmail".to_string(),
            "smtp.gmail.com".to_string(),
            587,
            "user@gmail.com".to_string(),
            "secret_password".to_string(),
            "alerts@example.com".to_string(),
        );

        let json = serde_json::to_string(&config).unwrap();
        assert!(!json.contains("secret_password")); // 密码不应该被序列化
    }
}

use thiserror::Error;

/// 应用程序错误类型
#[derive(Error, Debug)]
pub enum AppError {
    /// 数据库错误
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// HTTP 请求错误
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    /// 配置错误
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 序列化/反序列化错误
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Prometheus 错误
    #[error("Prometheus error: {0}")]
    Prometheus(#[from] prometheus::Error),

    /// SMTP 邮件错误
    #[error("SMTP error: {0}")]
    Smtp(#[from] lettre::error::Error),

    /// 地址解析错误
    #[error("Address parse error: {0}")]
    AddressParse(#[from] lettre::address::AddressError),

    /// RPC 错误
    #[error("RPC error: {0}")]
    Rpc(String),

    /// 数据验证错误
    #[error("Validation error: {0}")]
    Validation(String),

    /// 资源未找到
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// 超时错误
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// 告警引擎错误
    #[error("Alert engine error: {0}")]
    Alert(String),

    /// 通用错误
    #[error("{0}")]
    Generic(String),
}

/// Result 类型别名
pub type Result<T> = std::result::Result<T, AppError>;

impl AppError {
    /// 创建 RPC 错误
    pub fn rpc<S: Into<String>>(msg: S) -> Self {
        Self::Rpc(msg.into())
    }

    /// 创建验证错误
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    /// 创建未找到错误
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Self::NotFound(msg.into())
    }

    /// 创建超时错误
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        Self::Timeout(msg.into())
    }

    /// 创建告警错误
    pub fn alert<S: Into<String>>(msg: S) -> Self {
        Self::Alert(msg.into())
    }

    /// 创建通用错误
    pub fn generic<S: Into<String>>(msg: S) -> Self {
        Self::Generic(msg.into())
    }
}

/// 从 anyhow::Error 转换
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::Generic(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = AppError::rpc("Connection failed");
        assert_eq!(err.to_string(), "RPC error: Connection failed");

        let err = AppError::validation("Invalid input");
        assert_eq!(err.to_string(), "Validation error: Invalid input");

        let err = AppError::not_found("Node not found");
        assert_eq!(err.to_string(), "Resource not found: Node not found");
    }
}

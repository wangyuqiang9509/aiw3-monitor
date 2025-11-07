use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::collections::HashMap;

/// 应用程序配置
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub service: ServiceConfig,
    pub database: DatabaseConfig,
    pub nodes: Vec<NodeConfig>,
    pub collection: CollectionConfig,
    pub aggregation: AggregationConfig,
    pub retention: RetentionConfig,
    pub smtp: SmtpConfig,
    pub alerting: AlertingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub log_level: String,
    pub prometheus_port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NodeConfig {
    pub name: String,
    pub rpc_url: String,
    pub rest_url: String,
    pub grpc_url: String,
    pub environment: String,
    pub enabled: bool,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CollectionConfig {
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
    pub retry_count: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AggregationConfig {
    pub hourly_cron: String,
    pub daily_cron: String,
    pub monthly_cron: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RetentionConfig {
    pub raw_metrics_hours: i32,
    pub hourly_metrics_days: i32,
    pub daily_metrics_days: i32,
    pub monthly_metrics_days: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SmtpConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    pub use_tls: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AlertingConfig {
    pub check_interval_seconds: u64,
    pub silence_period_seconds: i64,
    pub max_retries: u32,
}

impl Settings {
    /// 从配置文件加载配置
    pub fn new() -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name("config").required(false))
            .add_source(File::with_name("config.toml").required(false))
            .build()?;

        config.try_deserialize()
    }
}

impl DatabaseConfig {
    /// 获取数据库连接字符串
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}

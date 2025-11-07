use crate::config::settings::DatabaseConfig;
use crate::error::{AppError, Result};
use sqlx::{postgres::PgPoolOptions, PgPool, Pool, Postgres};
use tracing::{error, info};

/// 创建数据库连接池
pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool> {
    info!(
        "Creating database connection pool: {}@{}:{}/{}",
        config.username, config.host, config.port, config.database
    );

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&config.connection_string())
        .await
        .map_err(|e| {
            error!("Failed to create database pool: {}", e);
            AppError::Database(e)
        })?;

    // 验证连接
    sqlx::query("SELECT 1").fetch_one(&pool).await.map_err(|e| {
        error!("Failed to verify database connection: {}", e);
        AppError::Database(e)
    })?;

    info!("Database connection pool created successfully");
    Ok(pool)
}

/// 数据库连接池包装器
pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    /// 创建新的数据库实例
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let pool = create_pool(config).await?;
        Ok(Self { pool })
    }

    /// 获取连接池引用
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// 测试连接是否正常
    pub async fn ping(&self) -> Result<()> {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await.map_err(AppError::Database)?;
        Ok(())
    }

    /// 获取连接池状态
    pub fn status(&self) -> DatabaseStatus {
        DatabaseStatus { size: self.pool.size(), idle: self.pool.num_idle() }
    }
}

/// 数据库连接池状态
#[derive(Debug, Clone)]
pub struct DatabaseStatus {
    pub size: u32,
    pub idle: usize,
}

impl std::fmt::Display for DatabaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pool[size={}, idle={}]", self.size, self.idle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::DatabaseConfig;

    #[tokio::test]
    #[ignore] // 需要真实数据库连接
    async fn test_create_pool() {
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "aiw3_monitor_test".to_string(),
            username: "postgres".to_string(),
            password: "secret".to_string(),
            max_connections: 5,
        };

        let result = create_pool(&config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_database_ping() {
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "aiw3_monitor_test".to_string(),
            username: "postgres".to_string(),
            password: "secret".to_string(),
            max_connections: 5,
        };

        let db = Database::new(&config).await.unwrap();
        let result = db.ping().await;
        assert!(result.is_ok());
    }
}

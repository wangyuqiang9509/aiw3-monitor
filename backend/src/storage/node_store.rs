// 节点配置存储
use sqlx::PgPool;
use tracing::{debug, info};

use crate::error::Result;
use crate::models::node::BlockchainNode;

/// 节点存储
pub struct NodeStore {
    pool: PgPool,
}

impl NodeStore {
    /// 创建新的节点存储
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 获取所有启用的节点
    pub async fn get_enabled_nodes(&self) -> Result<Vec<BlockchainNode>> {
        let nodes = sqlx::query_as::<_, BlockchainNode>(
            r#"
            SELECT id, name, rpc_url, rest_url, grpc_url, environment, 
                   enabled, labels, created_at, updated_at
            FROM blockchain_nodes
            WHERE enabled = true
            ORDER BY id
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        info!("Found {} enabled nodes", nodes.len());
        Ok(nodes)
    }

    /// 根据 ID 获取节点
    pub async fn get_by_id(&self, id: i32) -> Result<Option<BlockchainNode>> {
        let node = sqlx::query_as::<_, BlockchainNode>(
            r#"
            SELECT id, name, rpc_url, rest_url, grpc_url, environment, 
                   enabled, labels, created_at, updated_at
            FROM blockchain_nodes
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(node)
    }

    /// 根据名称获取节点
    pub async fn get_by_name(&self, name: &str) -> Result<Option<BlockchainNode>> {
        let node = sqlx::query_as::<_, BlockchainNode>(
            r#"
            SELECT id, name, rpc_url, rest_url, grpc_url, environment, 
                   enabled, labels, created_at, updated_at
            FROM blockchain_nodes
            WHERE name = $1
            "#
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(node)
    }

    /// 获取所有节点
    pub async fn get_all(&self) -> Result<Vec<BlockchainNode>> {
        let nodes = sqlx::query_as::<_, BlockchainNode>(
            r#"
            SELECT id, name, rpc_url, rest_url, grpc_url, environment, 
                   enabled, labels, created_at, updated_at
            FROM blockchain_nodes
            ORDER BY id
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(nodes)
    }

    /// 更新节点启用状态
    pub async fn update_enabled(&self, id: i32, enabled: bool) -> Result<()> {
        debug!("Updating node {} enabled status to {}", id, enabled);

        sqlx::query!(
            r#"
            UPDATE blockchain_nodes
            SET enabled = $1, updated_at = NOW()
            WHERE id = $2
            "#,
            enabled,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 插入新节点
    pub async fn insert(&self, node: &BlockchainNode) -> Result<i32> {
        let labels_json = serde_json::to_value(&node.labels)?;
        let id = sqlx::query_scalar::<_, i32>(
            r#"
            INSERT INTO blockchain_nodes 
                (name, rpc_url, rest_url, grpc_url, environment, enabled, labels)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#
        )
        .bind(&node.name)
        .bind(&node.rpc_url)
        .bind(&node.rest_url)
        .bind(&node.grpc_url)
        .bind(&node.environment)
        .bind(node.enabled)
        .bind(labels_json)
        .fetch_one(&self.pool)
        .await?;

        info!("Inserted new node: {} (id={})", node.name, id);
        Ok(id)
    }

    /// 删除节点
    pub async fn delete(&self, id: i32) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM blockchain_nodes
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        info!("Deleted node with id={}", id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要数据库连接
    async fn test_get_enabled_nodes() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| {
                "postgres://postgres:secret@localhost:5432/aiw3_monitor_test".to_string()
            });

        let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
        let store = NodeStore::new(pool);

        let nodes = store.get_enabled_nodes().await.unwrap();
        assert!(nodes.iter().all(|n| n.enabled));
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_by_name() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| {
                "postgres://postgres:secret@localhost:5432/aiw3_monitor_test".to_string()
            });

        let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
        let store = NodeStore::new(pool);

        let node = store.get_by_name("AIWS DevNet 3").await.unwrap();
        assert!(node.is_some());
    }
}

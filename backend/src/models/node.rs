use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

/// 区块链节点配置
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BlockchainNode {
    pub id: i32,
    pub name: String,
    pub rpc_url: String,
    pub rest_url: String,
    pub grpc_url: String,
    pub environment: String,
    pub enabled: bool,
    #[sqlx(json)]
    pub labels: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl BlockchainNode {
    pub fn new(
        name: String,
        rpc_url: String,
        rest_url: String,
        grpc_url: String,
        environment: String,
        labels: HashMap<String, String>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: 0, // 数据库生成
            name,
            rpc_url,
            rest_url,
            grpc_url,
            environment,
            enabled: true,
            labels,
            created_at: now,
            updated_at: now,
        }
    }
}

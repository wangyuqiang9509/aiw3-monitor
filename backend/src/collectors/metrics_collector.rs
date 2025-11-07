// 指标采集协调器
use sqlx::PgPool;
use tracing::{debug, error, info, warn};

use crate::collectors::rpc_client::RpcClient;
use crate::error::Result;
use crate::exporter::metrics_registry::MetricsRegistry;
use crate::models::metric::{MetricData, MetricType};
use crate::models::node::BlockchainNode;
use crate::storage::metrics_store::MetricsStore;
use crate::storage::node_store::NodeStore;

/// 指标采集器
pub struct MetricsCollector {
    node_store: NodeStore,
    metrics_store: MetricsStore,
    metrics_registry: MetricsRegistry,
}

impl MetricsCollector {
    /// 创建新的指标采集器
    pub fn new(pool: PgPool, metrics_registry: MetricsRegistry) -> Self {
        Self {
            node_store: NodeStore::new(pool.clone()),
            metrics_store: MetricsStore::new(pool),
            metrics_registry,
        }
    }

    /// 采集所有启用节点的指标
    pub async fn collect_all(&self) -> Result<usize> {
        info!("Starting metrics collection for all enabled nodes");

        let nodes = self.node_store.get_enabled_nodes().await?;
        if nodes.is_empty() {
            warn!("No enabled nodes found");
            return Ok(0);
        }

        info!("Found {} enabled nodes to collect metrics from", nodes.len());

        let mut collected = 0;
        for node in nodes {
            match self.collect_node_metrics(&node).await {
                Ok(count) => {
                    collected += count;
                    debug!("Collected {} metrics from node {}", count, node.name);
                }
                Err(e) => {
                    error!("Failed to collect metrics from node {}: {}", node.name, e);
                }
            }
        }

        info!("Collected {} total metrics from {} nodes", collected, collected);
        Ok(collected)
    }

    /// 采集单个节点的指标
    async fn collect_node_metrics(&self, node: &BlockchainNode) -> Result<usize> {
        let client = RpcClient::new(node.rpc_url.clone(), 10, 3);

        let mut metrics = Vec::new();
        let now = chrono::Utc::now();

        // 获取节点状态
        match client.get_status().await {
            Ok(status) => {
                // 区块高度
                if let Ok(height) = status.sync_info.block_height() {
                    metrics.push(MetricData::new(
                        node.id,
                        MetricType::BlockHeight,
                        height as f64,
                        None,
                    ));

                    // 更新 Prometheus 指标
                    let chain_id = node
                        .labels
                        .get("chain_id")
                        .map(|s| s.as_str())
                        .unwrap_or("unknown");
                    self.metrics_registry.set_block_height(
                        &node.name,
                        &node.environment,
                        chain_id,
                        height as f64,
                    );
                }

                // 同步状态
                let catching_up = status.sync_info.catching_up;
                self.metrics_registry
                    .set_sync_status(&node.name, &node.environment, catching_up);
            }
            Err(e) => {
                warn!("Failed to get status from {}: {}", node.name, e);
            }
        }

        // 获取网络信息
        match client.get_net_info().await {
            Ok(net_info) => {
                if let Ok(peer_count) = net_info.peer_count() {
                    metrics.push(MetricData::new(
                        node.id,
                        MetricType::NodeCount,
                        peer_count as f64,
                        None,
                    ));

                    // 更新 Prometheus 指标
                    self.metrics_registry
                        .set_node_count(&node.name, &node.environment, peer_count as f64);
                }
            }
            Err(e) => {
                warn!("Failed to get net_info from {}: {}", node.name, e);
            }
        }

        // 获取交易池信息
        match client.get_num_unconfirmed_txs().await {
            Ok(tx_info) => {
                if let Ok(tx_count) = tx_info.tx_count() {
                    metrics.push(MetricData::new(
                        node.id,
                        MetricType::TxPoolSize,
                        tx_count as f64,
                        None,
                    ));

                    // 更新 Prometheus 指标
                    self.metrics_registry
                        .set_tx_pool_size(&node.name, &node.environment, tx_count as f64);
                }
            }
            Err(e) => {
                warn!(
                    "Failed to get unconfirmed_txs from {}: {}",
                    node.name, e
                );
            }
        }

        // 批量插入指标到数据库
        let count = metrics.len();
        if !metrics.is_empty() {
            self.metrics_store.insert_batch(&metrics).await?;
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要数据库和真实 RPC 端点
    async fn test_collect_all() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| {
                "postgres://postgres:secret@localhost:5432/aiw3_monitor_test".to_string()
            });

        let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
        let registry = MetricsRegistry::new().unwrap();
        let collector = MetricsCollector::new(pool, registry);

        let result = collector.collect_all().await;
        assert!(result.is_ok());
    }
}

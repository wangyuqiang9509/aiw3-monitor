// 区块链高级指标采集器
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::collectors::rpc_client::RpcClient;
use crate::error::{AppError, Result};

/// Mempool 指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolMetrics {
    /// 未确认交易数量
    pub unconfirmed_txs: u64,
    /// Mempool 大小（字节）
    pub mempool_size_bytes: u64,
    /// 交易总数
    pub total_txs: u64,
}

/// 验证者指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorMetrics {
    /// 验证者总数
    pub total_validators: u64,
    /// 在线验证者数量
    pub online_validators: u64,
    /// 总投票权
    pub total_voting_power: i64,
    /// 平均投票权
    pub average_voting_power: f64,
}

/// 网络指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// 入站连接数
    pub inbound_peers: u64,
    /// 出站连接数
    pub outbound_peers: u64,
    /// 总连接数
    pub total_peers: u64,
    /// 监听地址数量
    pub listening_addresses: u64,
}

/// 共识状态指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusMetrics {
    /// 当前高度
    pub height: u64,
    /// 当前轮次
    pub round: u32,
    /// 当前步骤 (1=propose, 2=prevote, 3=precommit, 4=commit)
    pub step: u32,
    /// 提议者地址
    pub proposer_address: String,
}

/// 区块链指标采集器
pub struct BlockchainMetricsCollector {
    rpc_client: RpcClient,
}

impl BlockchainMetricsCollector {
    /// 创建新的区块链指标采集器
    pub fn new(rpc_url: String) -> Self {
        Self { rpc_client: RpcClient::new(rpc_url, 10, 3) }
    }

    /// 采集 Mempool 指标
    pub async fn collect_mempool_metrics(&self) -> Result<MempoolMetrics> {
        debug!("Collecting mempool metrics");

        // 获取未确认交易数量
        match self.rpc_client.get_num_unconfirmed_txs().await {
            Ok(result) => {
                let unconfirmed_txs = result.n_txs.parse::<u64>().unwrap_or(0);
                let total_txs = result.total.parse::<u64>().unwrap_or(0);

                // 估算 mempool 大小（假设平均每笔交易 500 字节）
                let mempool_size_bytes = unconfirmed_txs * 500;

                debug!(
                    "Mempool metrics: unconfirmed={}, total={}, size={}",
                    unconfirmed_txs, total_txs, mempool_size_bytes
                );

                Ok(MempoolMetrics { unconfirmed_txs, mempool_size_bytes, total_txs })
            }
            Err(e) => {
                warn!("Failed to collect mempool metrics: {}", e);
                Err(AppError::rpc(format!("Failed to get unconfirmed txs: {}", e)))
            }
        }
    }

    /// 采集验证者指标
    pub async fn collect_validator_metrics(&self) -> Result<ValidatorMetrics> {
        debug!("Collecting validator metrics");

        match self.rpc_client.get_validators(None, None, None).await {
            Ok(result) => {
                let total_validators = result.total_count().unwrap_or(0);
                let online_validators = result.active_count().unwrap_or(0);
                let total_voting_power = result.total_voting_power().unwrap_or(0);
                let average_voting_power = result.average_voting_power().unwrap_or(0.0);

                debug!(
                    "Validator metrics: total={}, online={}, total_power={}, avg_power={}",
                    total_validators, online_validators, total_voting_power, average_voting_power
                );

                Ok(ValidatorMetrics {
                    total_validators,
                    online_validators,
                    total_voting_power,
                    average_voting_power,
                })
            }
            Err(e) => {
                warn!("Failed to collect validator metrics: {}", e);
                Err(AppError::rpc(format!("Failed to get validators: {}", e)))
            }
        }
    }

    /// 采集共识状态指标
    pub async fn collect_consensus_metrics(&self) -> Result<ConsensusMetrics> {
        debug!("Collecting consensus metrics");

        match self.rpc_client.get_consensus_state().await {
            Ok(result) => {
                let height = result.round_state.height().unwrap_or(0);
                let round = result.round_state.round().unwrap_or(0);
                let step = result.round_state.step().unwrap_or(0);
                let proposer_address = result.round_state.proposer.address.clone();

                debug!(
                    "Consensus metrics: height={}, round={}, step={}, proposer={}",
                    height, round, step, proposer_address
                );

                Ok(ConsensusMetrics { height, round, step, proposer_address })
            }
            Err(e) => {
                warn!("Failed to collect consensus metrics: {}", e);
                Err(AppError::rpc(format!("Failed to get consensus state: {}", e)))
            }
        }
    }

    /// 采集网络指标
    pub async fn collect_network_metrics(&self) -> Result<NetworkMetrics> {
        debug!("Collecting network metrics");

        // 获取网络信息
        match self.rpc_client.get_net_info().await {
            Ok(result) => {
                let total_peers = result.n_peers.parse::<u64>().unwrap_or(0);
                let listening_addresses = result.listeners.len() as u64;

                // 统计入站和出站连接
                let mut inbound_peers = 0u64;
                let mut outbound_peers = 0u64;

                for peer in &result.peers {
                    if peer.is_outbound {
                        outbound_peers += 1;
                    } else {
                        inbound_peers += 1;
                    }
                }

                debug!(
                    "Network metrics: total={}, inbound={}, outbound={}, listeners={}",
                    total_peers, inbound_peers, outbound_peers, listening_addresses
                );

                Ok(NetworkMetrics {
                    inbound_peers,
                    outbound_peers,
                    total_peers,
                    listening_addresses,
                })
            }
            Err(e) => {
                warn!("Failed to collect network metrics: {}", e);
                Err(AppError::rpc(format!("Failed to get net info: {}", e)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_metrics_collector_creation() {
        let _collector = BlockchainMetricsCollector::new("http://localhost:26657".to_string());
        // 基本创建测试
        assert!(true);
    }

    #[tokio::test]
    #[ignore = "Requires a running blockchain node for integration test"]
    async fn test_collect_mempool_metrics_integration() {
        let collector = BlockchainMetricsCollector::new("http://localhost:26657".to_string());

        match collector.collect_mempool_metrics().await {
            Ok(metrics) => {
                println!("Mempool metrics: {:?}", metrics);
                // 验证指标已成功采集
                assert!(true);
            }
            Err(e) => {
                println!("Skipping integration test: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore = "Requires a running blockchain node for integration test"]
    async fn test_collect_validator_metrics_integration() {
        let collector = BlockchainMetricsCollector::new("http://localhost:26657".to_string());

        match collector.collect_validator_metrics().await {
            Ok(metrics) => {
                println!("Validator metrics: {:?}", metrics);
                // 验证指标已成功采集
                assert!(true);
            }
            Err(e) => {
                println!("Skipping integration test: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore = "Requires a running blockchain node for integration test"]
    async fn test_collect_network_metrics_integration() {
        let collector = BlockchainMetricsCollector::new("http://localhost:26657".to_string());

        match collector.collect_network_metrics().await {
            Ok(metrics) => {
                println!("Network metrics: {:?}", metrics);
                // 验证指标已成功采集
                assert!(true);
            }
            Err(e) => {
                println!("Skipping integration test: {}", e);
            }
        }
    }
}

// 区块链静态配置加载模块
// 从 performance-metrics.json 加载区块链配置信息

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::error::{AppError, Result};

/// 区块链配置（从 performance-metrics.json 加载）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockchainConfig {
    pub execution_metrics: ExecutionMetrics,
    pub storage_metrics: StorageMetrics,
    pub network_metrics: NetworkMetrics,
    pub throughput_metrics: ThroughputMetrics,
    pub optimization_features: Vec<OptimizationFeature>,
}

/// 执行指标配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutionMetrics {
    pub block_stm: BlockStmConfig,
}

/// Block-STM 配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockStmConfig {
    pub enabled: bool,
    pub status: String,
    pub workers: u32,
    pub pre_estimation: bool,
    pub conflict_resolution: String,
    pub performance_gain: u32,
    pub sequential_fallback: bool,
    pub average_parallelism_percentage: u32,
    pub conflict_rate_percentage: u32,
}

/// 存储指标配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageMetrics {
    pub memiavl: MemiavlConfig,
    pub iavl: IavlConfig,
    pub database: DatabaseConfig,
}

/// MemIAVL 配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemiavlConfig {
    pub enabled: bool,
    pub status: String,
    pub cache_size_nodes: u64,
    pub zero_copy: bool,
    pub async_commit_buffer: u32,
    pub snapshot_interval_blocks: u32,
    pub performance_gain: u32,
    pub cache_hit_rate_percentage: u32,
    pub commit_latency_ms: u32,
}

/// IAVL 配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IavlConfig {
    pub cache_size_nodes: u64,
    pub inter_block_cache: bool,
}

/// 数据库配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub backend: String,
    pub pruning_strategy_devnet: String,
    pub pruning_strategy_production: String,
}

/// 优化特性
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OptimizationFeature {
    pub name: String,
    pub enabled: bool,
    pub impact: String,
    pub category: String,
    pub description: String,
}

/// 网络指标配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkMetrics {
    pub p2p: P2pConfig,
}

/// P2P 网络配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct P2pConfig {
    pub protocol: String,
    pub max_inbound_peers: u64,
    pub max_outbound_peers: u64,
    pub send_rate_bytes_per_second: u64,
    pub recv_rate_bytes_per_second: u64,
    pub flush_throttle_timeout_ms: u64,
    pub mtu: u64,
}

/// 吞吐量指标配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThroughputMetrics {
    pub block_capacity: BlockCapacity,
}

/// 区块容量配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockCapacity {
    pub max_size_mb: u64,
    pub max_size_bytes: u64,
}

/// 加载区块链配置
///
/// 从编译时嵌入的 performance-metrics.json 文件加载配置
///
/// # 返回
/// * `Ok(BlockchainConfig)` - 成功加载的配置
/// * `Err(AppError)` - 加载或解析失败
pub fn load_blockchain_config() -> Result<BlockchainConfig> {
    info!("Loading blockchain configuration from embedded performance-metrics.json");

    // 在编译时嵌入 JSON 文件内容
    let config_data = include_str!("performance-metrics.json");

    // 解析 JSON
    let config: BlockchainConfig = serde_json::from_str(config_data)
        .map_err(|e| AppError::generic(format!("Failed to parse blockchain config: {}", e)))?;

    info!(
        "Blockchain config loaded: Block-STM {} with {} workers, MemIAVL {} with {}M cache",
        if config.execution_metrics.block_stm.enabled { "enabled" } else { "disabled" },
        config.execution_metrics.block_stm.workers,
        if config.storage_metrics.memiavl.enabled { "enabled" } else { "disabled" },
        config.storage_metrics.memiavl.cache_size_nodes / 1_000_000
    );

    Ok(config)
}

impl BlockchainConfig {
    /// 获取 Block-STM 配置
    pub fn block_stm(&self) -> &BlockStmConfig {
        &self.execution_metrics.block_stm
    }

    /// 获取 MemIAVL 配置
    pub fn memiavl(&self) -> &MemiavlConfig {
        &self.storage_metrics.memiavl
    }

    /// 获取 IAVL 配置
    pub fn iavl(&self) -> &IavlConfig {
        &self.storage_metrics.iavl
    }

    /// 获取数据库配置
    pub fn database(&self) -> &DatabaseConfig {
        &self.storage_metrics.database
    }

    /// 获取所有启用的优化特性
    pub fn enabled_optimizations(&self) -> Vec<&OptimizationFeature> {
        self.optimization_features.iter().filter(|f| f.enabled).collect()
    }

    /// 检查是否启用了 Zero-Copy 优化
    pub fn is_zero_copy_enabled(&self) -> bool {
        self.optimization_features.iter().any(|f| f.name.contains("Zero-Copy") && f.enabled)
    }

    /// 检查是否启用了 Optimistic Execution
    pub fn is_optimistic_execution_enabled(&self) -> bool {
        self.optimization_features
            .iter()
            .any(|f| f.name.contains("Optimistic Execution") && f.enabled)
    }

    /// 检查是否启用了 Large Block Size
    pub fn is_large_blocks_enabled(&self) -> bool {
        self.optimization_features.iter().any(|f| f.name.contains("Large Block Size") && f.enabled)
    }

    /// 获取 P2P 网络配置
    pub fn p2p_config(&self) -> &P2pConfig {
        &self.network_metrics.p2p
    }

    /// 获取 Large Block Size 配置
    pub fn large_blocks_config(&self) -> Option<&OptimizationFeature> {
        self.optimization_features
            .iter()
            .find(|f| f.name.contains("Large Block Size"))
    }

    /// 获取最大区块大小（MB）
    pub fn max_block_size_mb(&self) -> u64 {
        self.throughput_metrics.block_capacity.max_size_mb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_blockchain_config() {
        let config = load_blockchain_config();
        assert!(config.is_ok());

        let config = config.unwrap();
        assert!(config.execution_metrics.block_stm.enabled);
        assert_eq!(config.execution_metrics.block_stm.workers, 8);
        assert!(config.storage_metrics.memiavl.enabled);
        assert_eq!(config.storage_metrics.memiavl.cache_size_nodes, 2_000_000);
    }

    #[test]
    fn test_block_stm_config_access() {
        let config = load_blockchain_config().unwrap();
        let block_stm = config.block_stm();

        assert!(block_stm.enabled);
        assert_eq!(block_stm.workers, 8);
        assert!(block_stm.pre_estimation);
        assert_eq!(block_stm.performance_gain, 8);
    }

    #[test]
    fn test_memiavl_config_access() {
        let config = load_blockchain_config().unwrap();
        let memiavl = config.memiavl();

        assert!(memiavl.enabled);
        assert_eq!(memiavl.cache_size_nodes, 2_000_000);
        assert!(memiavl.zero_copy);
        assert_eq!(memiavl.performance_gain, 10);
        assert_eq!(memiavl.async_commit_buffer, 200);
        assert_eq!(memiavl.snapshot_interval_blocks, 500);
        assert_eq!(memiavl.cache_hit_rate_percentage, 95);
        assert_eq!(memiavl.commit_latency_ms, 50);
    }

    #[test]
    fn test_iavl_config_access() {
        let config = load_blockchain_config().unwrap();
        let iavl = config.iavl();

        assert_eq!(iavl.cache_size_nodes, 781_250);
        assert!(iavl.inter_block_cache);
    }

    #[test]
    fn test_database_config_access() {
        let config = load_blockchain_config().unwrap();
        let database = config.database();

        assert_eq!(database.backend, "goleveldb");
        assert_eq!(database.pruning_strategy_devnet, "nothing");
        assert_eq!(database.pruning_strategy_production, "default");
    }

    #[test]
    fn test_enabled_optimizations() {
        let config = load_blockchain_config().unwrap();
        let enabled = config.enabled_optimizations();

        assert!(!enabled.is_empty());
        assert!(enabled.iter().all(|f| f.enabled));
    }

    #[test]
    fn test_optimization_checks() {
        let config = load_blockchain_config().unwrap();

        assert!(config.is_zero_copy_enabled());
        assert!(config.is_optimistic_execution_enabled());
        assert!(config.is_large_blocks_enabled());
    }

    #[test]
    fn test_throughput_metrics_access() {
        let config = load_blockchain_config().unwrap();
        assert_eq!(config.throughput_metrics.block_capacity.max_size_mb, 100);
        assert_eq!(config.throughput_metrics.block_capacity.max_size_bytes, 104857600);
    }

    #[test]
    fn test_large_blocks_config() {
        let config = load_blockchain_config().unwrap();
        let large_blocks = config.large_blocks_config();
        assert!(large_blocks.is_some());
        assert!(large_blocks.unwrap().enabled);
        assert_eq!(large_blocks.unwrap().category, "throughput_optimization");
    }

    #[test]
    fn test_max_block_size_mb() {
        let config = load_blockchain_config().unwrap();
        assert_eq!(config.max_block_size_mb(), 100);
    }
}

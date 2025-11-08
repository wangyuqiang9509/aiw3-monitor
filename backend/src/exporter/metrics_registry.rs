// Prometheus 指标注册器
use prometheus::{
    core::Collector, Encoder, Gauge, GaugeVec, Opts, Registry, TextEncoder,
};
use tracing::info;

use crate::error::Result;

/// Prometheus 指标注册器
#[derive(Clone)]
pub struct MetricsRegistry {
    pub registry: Registry,
    // 区块链指标
    pub block_height: GaugeVec,
    pub block_time: GaugeVec,
    pub node_count: GaugeVec,
    pub tx_pool_size: GaugeVec,
    pub network_latency: GaugeVec,
    pub validator_count: GaugeVec,
    pub tps: GaugeVec,
    pub sync_status: GaugeVec,
    // 资源指标
    pub container_cpu_percent: GaugeVec,
    pub container_memory_bytes: GaugeVec,
    pub container_memory_limit_bytes: GaugeVec,
    pub container_memory_percent: GaugeVec,
    pub container_network_rx_bytes: GaugeVec,
    pub container_network_tx_bytes: GaugeVec,
    pub container_block_read_bytes: GaugeVec,
    pub container_block_write_bytes: GaugeVec,
    // 优化特性指标
    pub optimization_block_stm_enabled: GaugeVec,
    pub optimization_block_stm_workers: GaugeVec,
    pub optimization_memiavl_enabled: GaugeVec,
    pub optimization_memiavl_cache_size: GaugeVec,
    pub optimization_memiavl_snapshot_interval: GaugeVec,
    pub optimization_zero_copy_enabled: GaugeVec,
    // TPS 和性能指标
    pub chain_tps_current: GaugeVec,
    pub chain_tps_average: GaugeVec,
    pub chain_tps_peak: GaugeVec,
    pub chain_total_transactions: GaugeVec,
    // Mempool 指标
    pub mempool_unconfirmed_txs: GaugeVec,
    pub mempool_size_bytes: GaugeVec,
    pub mempool_total_txs: GaugeVec,
    // 验证者指标
    pub validator_total: GaugeVec,
    pub validator_online: GaugeVec,
    pub validator_total_voting_power: GaugeVec,
    pub validator_average_voting_power: GaugeVec,
    // 网络指标
    pub network_inbound_peers: GaugeVec,
    pub network_outbound_peers: GaugeVec,
    pub network_total_peers: GaugeVec,
    pub network_listening_addresses: GaugeVec,
}

impl MetricsRegistry {
    /// 创建新的指标注册器
    pub fn new() -> Result<Self> {
        let registry = Registry::new();

        // 区块高度
        let block_height = GaugeVec::new(
            Opts::new("aiw3_chain_block_height", "Current block height of the chain"),
            &["node", "environment", "chain_id"],
        )?;
        registry.register(Box::new(block_height.clone()))?;

        // 区块时间
        let block_time = GaugeVec::new(
            Opts::new(
                "aiw3_chain_block_time_seconds",
                "Time since last block in seconds",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(block_time.clone()))?;

        // 节点数量
        let node_count = GaugeVec::new(
            Opts::new("aiw3_chain_node_count", "Number of connected peers"),
            &["node", "environment"],
        )?;
        registry.register(Box::new(node_count.clone()))?;

        // 交易池大小
        let tx_pool_size = GaugeVec::new(
            Opts::new(
                "aiw3_chain_tx_pool_size",
                "Number of unconfirmed transactions",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(tx_pool_size.clone()))?;

        // 网络延迟
        let network_latency = GaugeVec::new(
            Opts::new(
                "aiw3_chain_network_latency_ms",
                "Network latency in milliseconds",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(network_latency.clone()))?;

        // 验证者数量
        let validator_count = GaugeVec::new(
            Opts::new("aiw3_chain_validator_count", "Number of active validators"),
            &["node", "environment"],
        )?;
        registry.register(Box::new(validator_count.clone()))?;

        // TPS (每秒交易数)
        let tps = GaugeVec::new(
            Opts::new("aiw3_chain_tps", "Transactions per second"),
            &["node", "environment"],
        )?;
        registry.register(Box::new(tps.clone()))?;

        // 同步状态 (0=同步中, 1=已同步)
        let sync_status = GaugeVec::new(
            Opts::new("aiw3_chain_sync_status", "Chain synchronization status"),
            &["node", "environment"],
        )?;
        registry.register(Box::new(sync_status.clone()))?;

        // ========== 资源指标 ==========
        
        // CPU 使用率
        let container_cpu_percent = GaugeVec::new(
            Opts::new(
                "aiw3_container_cpu_percent",
                "Container CPU usage percentage (can exceed 100% for multi-core)",
            ),
            &["container", "node", "environment"],
        )?;
        registry.register(Box::new(container_cpu_percent.clone()))?;

        // 内存使用量（字节）
        let container_memory_bytes = GaugeVec::new(
            Opts::new(
                "aiw3_container_memory_bytes",
                "Container memory usage in bytes",
            ),
            &["container", "node", "environment"],
        )?;
        registry.register(Box::new(container_memory_bytes.clone()))?;

        // 内存限制（字节）
        let container_memory_limit_bytes = GaugeVec::new(
            Opts::new(
                "aiw3_container_memory_limit_bytes",
                "Container memory limit in bytes",
            ),
            &["container", "node", "environment"],
        )?;
        registry.register(Box::new(container_memory_limit_bytes.clone()))?;

        // 内存使用率
        let container_memory_percent = GaugeVec::new(
            Opts::new(
                "aiw3_container_memory_percent",
                "Container memory usage percentage",
            ),
            &["container", "node", "environment"],
        )?;
        registry.register(Box::new(container_memory_percent.clone()))?;

        // 网络接收字节数
        let container_network_rx_bytes = GaugeVec::new(
            Opts::new(
                "aiw3_container_network_rx_bytes",
                "Container network received bytes",
            ),
            &["container", "node", "environment"],
        )?;
        registry.register(Box::new(container_network_rx_bytes.clone()))?;

        // 网络发送字节数
        let container_network_tx_bytes = GaugeVec::new(
            Opts::new(
                "aiw3_container_network_tx_bytes",
                "Container network transmitted bytes",
            ),
            &["container", "node", "environment"],
        )?;
        registry.register(Box::new(container_network_tx_bytes.clone()))?;

        // 磁盘读取字节数
        let container_block_read_bytes = GaugeVec::new(
            Opts::new(
                "aiw3_container_block_read_bytes",
                "Container disk read bytes",
            ),
            &["container", "node", "environment"],
        )?;
        registry.register(Box::new(container_block_read_bytes.clone()))?;

        // 磁盘写入字节数
        let container_block_write_bytes = GaugeVec::new(
            Opts::new(
                "aiw3_container_block_write_bytes",
                "Container disk write bytes",
            ),
            &["container", "node", "environment"],
        )?;
        registry.register(Box::new(container_block_write_bytes.clone()))?;

        // Block-STM 启用状态
        let optimization_block_stm_enabled = GaugeVec::new(
            Opts::new(
                "aiw3_optimization_block_stm_enabled",
                "Whether Block-STM is enabled (1=enabled, 0=disabled)",
            ),
            &["container", "node", "environment"],
        )?;
        registry.register(Box::new(optimization_block_stm_enabled.clone()))?;

        // Block-STM 工作线程数
        let optimization_block_stm_workers = GaugeVec::new(
            Opts::new(
                "aiw3_optimization_block_stm_workers",
                "Number of Block-STM worker threads",
            ),
            &["container", "node", "environment"],
        )?;
        registry.register(Box::new(optimization_block_stm_workers.clone()))?;

        // MemIAVL 启用状态
        let optimization_memiavl_enabled = GaugeVec::new(
            Opts::new(
                "aiw3_optimization_memiavl_enabled",
                "Whether MemIAVL is enabled (1=enabled, 0=disabled)",
            ),
            &["container", "node", "environment"],
        )?;
        registry.register(Box::new(optimization_memiavl_enabled.clone()))?;

        // MemIAVL 缓存大小
        let optimization_memiavl_cache_size = GaugeVec::new(
            Opts::new(
                "aiw3_optimization_memiavl_cache_size",
                "MemIAVL cache size in nodes",
            ),
            &["container", "node", "environment"],
        )?;
        registry.register(Box::new(optimization_memiavl_cache_size.clone()))?;

        // MemIAVL 快照间隔
        let optimization_memiavl_snapshot_interval = GaugeVec::new(
            Opts::new(
                "aiw3_optimization_memiavl_snapshot_interval",
                "MemIAVL snapshot interval in blocks",
            ),
            &["container", "node", "environment"],
        )?;
        registry.register(Box::new(optimization_memiavl_snapshot_interval.clone()))?;

        // Zero-Copy 启用状态
        let optimization_zero_copy_enabled = GaugeVec::new(
            Opts::new(
                "aiw3_optimization_zero_copy_enabled",
                "Whether Zero-Copy is enabled (1=enabled, 0=disabled)",
            ),
            &["container", "node", "environment"],
        )?;
        registry.register(Box::new(optimization_zero_copy_enabled.clone()))?;

        // 当前 TPS
        let chain_tps_current = GaugeVec::new(
            Opts::new(
                "aiw3_chain_tps_current",
                "Current transactions per second",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(chain_tps_current.clone()))?;

        // 平均 TPS
        let chain_tps_average = GaugeVec::new(
            Opts::new(
                "aiw3_chain_tps_average",
                "Average transactions per second",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(chain_tps_average.clone()))?;

        // 峰值 TPS
        let chain_tps_peak = GaugeVec::new(
            Opts::new(
                "aiw3_chain_tps_peak",
                "Peak transactions per second",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(chain_tps_peak.clone()))?;

        // 总交易数（时间窗口内）
        let chain_total_transactions = GaugeVec::new(
            Opts::new(
                "aiw3_chain_total_transactions",
                "Total transactions in time window",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(chain_total_transactions.clone()))?;

        // Mempool 未确认交易数
        let mempool_unconfirmed_txs = GaugeVec::new(
            Opts::new(
                "aiw3_mempool_unconfirmed_txs",
                "Number of unconfirmed transactions in mempool",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(mempool_unconfirmed_txs.clone()))?;

        // Mempool 大小（字节）
        let mempool_size_bytes = GaugeVec::new(
            Opts::new(
                "aiw3_mempool_size_bytes",
                "Size of mempool in bytes",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(mempool_size_bytes.clone()))?;

        // Mempool 总交易数
        let mempool_total_txs = GaugeVec::new(
            Opts::new(
                "aiw3_mempool_total_txs",
                "Total transactions in mempool",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(mempool_total_txs.clone()))?;

        // 验证者总数
        let validator_total = GaugeVec::new(
            Opts::new(
                "aiw3_validator_total",
                "Total number of validators",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(validator_total.clone()))?;

        // 在线验证者数量
        let validator_online = GaugeVec::new(
            Opts::new(
                "aiw3_validator_online",
                "Number of online validators",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(validator_online.clone()))?;

        // 总投票权
        let validator_total_voting_power = GaugeVec::new(
            Opts::new(
                "aiw3_validator_total_voting_power",
                "Total voting power of all validators",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(validator_total_voting_power.clone()))?;

        // 平均投票权
        let validator_average_voting_power = GaugeVec::new(
            Opts::new(
                "aiw3_validator_average_voting_power",
                "Average voting power per validator",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(validator_average_voting_power.clone()))?;

        // 入站连接数
        let network_inbound_peers = GaugeVec::new(
            Opts::new(
                "aiw3_network_inbound_peers",
                "Number of inbound peer connections",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(network_inbound_peers.clone()))?;

        // 出站连接数
        let network_outbound_peers = GaugeVec::new(
            Opts::new(
                "aiw3_network_outbound_peers",
                "Number of outbound peer connections",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(network_outbound_peers.clone()))?;

        // 总连接数
        let network_total_peers = GaugeVec::new(
            Opts::new(
                "aiw3_network_total_peers",
                "Total number of peer connections",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(network_total_peers.clone()))?;

        // 监听地址数量
        let network_listening_addresses = GaugeVec::new(
            Opts::new(
                "aiw3_network_listening_addresses",
                "Number of listening addresses",
            ),
            &["node", "environment"],
        )?;
        registry.register(Box::new(network_listening_addresses.clone()))?;

        info!("Prometheus metrics registry initialized with 37 metrics (8 blockchain + 8 resource + 6 optimization + 4 TPS + 3 mempool + 4 validator + 4 network)");

        Ok(Self {
            registry,
            block_height,
            block_time,
            node_count,
            tx_pool_size,
            network_latency,
            validator_count,
            tps,
            sync_status,
            container_cpu_percent,
            container_memory_bytes,
            container_memory_limit_bytes,
            container_memory_percent,
            container_network_rx_bytes,
            container_network_tx_bytes,
            container_block_read_bytes,
            container_block_write_bytes,
            optimization_block_stm_enabled,
            optimization_block_stm_workers,
            optimization_memiavl_enabled,
            optimization_memiavl_cache_size,
            optimization_memiavl_snapshot_interval,
            optimization_zero_copy_enabled,
            chain_tps_current,
            chain_tps_average,
            chain_tps_peak,
            chain_total_transactions,
            mempool_unconfirmed_txs,
            mempool_size_bytes,
            mempool_total_txs,
            validator_total,
            validator_online,
            validator_total_voting_power,
            validator_average_voting_power,
            network_inbound_peers,
            network_outbound_peers,
            network_total_peers,
            network_listening_addresses,
        })
    }

    /// 更新区块高度
    pub fn set_block_height(&self, node: &str, environment: &str, chain_id: &str, value: f64) {
        self.block_height
            .with_label_values(&[node, environment, chain_id])
            .set(value);
    }

    /// 更新节点数量
    pub fn set_node_count(&self, node: &str, environment: &str, value: f64) {
        self.node_count
            .with_label_values(&[node, environment])
            .set(value);
    }

    /// 更新交易池大小
    pub fn set_tx_pool_size(&self, node: &str, environment: &str, value: f64) {
        self.tx_pool_size
            .with_label_values(&[node, environment])
            .set(value);
    }

    /// 更新同步状态
    pub fn set_sync_status(&self, node: &str, environment: &str, catching_up: bool) {
        let value = if catching_up { 0.0 } else { 1.0 };
        self.sync_status
            .with_label_values(&[node, environment])
            .set(value);
    }

    // ========== 资源指标 Setters ==========

    /// 更新容器 CPU 使用率
    pub fn set_container_cpu_percent(&self, container: &str, node: &str, environment: &str, value: f64) {
        self.container_cpu_percent
            .with_label_values(&[container, node, environment])
            .set(value);
    }

    /// 更新容器内存使用量
    pub fn set_container_memory_bytes(&self, container: &str, node: &str, environment: &str, value: f64) {
        self.container_memory_bytes
            .with_label_values(&[container, node, environment])
            .set(value);
    }

    /// 更新容器内存限制
    pub fn set_container_memory_limit_bytes(&self, container: &str, node: &str, environment: &str, value: f64) {
        self.container_memory_limit_bytes
            .with_label_values(&[container, node, environment])
            .set(value);
    }

    /// 更新容器内存使用率
    pub fn set_container_memory_percent(&self, container: &str, node: &str, environment: &str, value: f64) {
        self.container_memory_percent
            .with_label_values(&[container, node, environment])
            .set(value);
    }

    /// 更新容器网络接收字节数
    pub fn set_container_network_rx_bytes(&self, container: &str, node: &str, environment: &str, value: f64) {
        self.container_network_rx_bytes
            .with_label_values(&[container, node, environment])
            .set(value);
    }

    /// 更新容器网络发送字节数
    pub fn set_container_network_tx_bytes(&self, container: &str, node: &str, environment: &str, value: f64) {
        self.container_network_tx_bytes
            .with_label_values(&[container, node, environment])
            .set(value);
    }

    /// 更新容器磁盘读取字节数
    pub fn set_container_block_read_bytes(&self, container: &str, node: &str, environment: &str, value: f64) {
        self.container_block_read_bytes
            .with_label_values(&[container, node, environment])
            .set(value);
    }

    /// 更新容器磁盘写入字节数
    pub fn set_container_block_write_bytes(&self, container: &str, node: &str, environment: &str, value: f64) {
        self.container_block_write_bytes
            .with_label_values(&[container, node, environment])
            .set(value);
    }

    /// 批量更新资源指标（从 ResourceMetrics）
    pub fn set_resource_metrics(
        &self,
        metrics: &crate::collectors::ResourceMetrics,
        node: &str,
        environment: &str,
    ) {
        self.set_container_cpu_percent(&metrics.container_name, node, environment, metrics.cpu_percent);
        self.set_container_memory_bytes(&metrics.container_name, node, environment, metrics.memory_bytes as f64);
        self.set_container_memory_limit_bytes(&metrics.container_name, node, environment, metrics.memory_limit_bytes as f64);
        self.set_container_memory_percent(&metrics.container_name, node, environment, metrics.memory_percent);
        self.set_container_network_rx_bytes(&metrics.container_name, node, environment, metrics.network_rx_bytes as f64);
        self.set_container_network_tx_bytes(&metrics.container_name, node, environment, metrics.network_tx_bytes as f64);
        self.set_container_block_read_bytes(&metrics.container_name, node, environment, metrics.block_read_bytes as f64);
        self.set_container_block_write_bytes(&metrics.container_name, node, environment, metrics.block_write_bytes as f64);
    }

    /// 更新 Block-STM 启用状态
    pub fn set_optimization_block_stm_enabled(&self, container: &str, node: &str, environment: &str, enabled: bool) {
        self.optimization_block_stm_enabled
            .with_label_values(&[container, node, environment])
            .set(if enabled { 1.0 } else { 0.0 });
    }

    /// 更新 Block-STM 工作线程数
    pub fn set_optimization_block_stm_workers(&self, container: &str, node: &str, environment: &str, workers: i32) {
        self.optimization_block_stm_workers
            .with_label_values(&[container, node, environment])
            .set(workers as f64);
    }

    /// 更新 MemIAVL 启用状态
    pub fn set_optimization_memiavl_enabled(&self, container: &str, node: &str, environment: &str, enabled: bool) {
        self.optimization_memiavl_enabled
            .with_label_values(&[container, node, environment])
            .set(if enabled { 1.0 } else { 0.0 });
    }

    /// 更新 MemIAVL 缓存大小
    pub fn set_optimization_memiavl_cache_size(&self, container: &str, node: &str, environment: &str, cache_size: i64) {
        self.optimization_memiavl_cache_size
            .with_label_values(&[container, node, environment])
            .set(cache_size as f64);
    }

    /// 更新 MemIAVL 快照间隔
    pub fn set_optimization_memiavl_snapshot_interval(&self, container: &str, node: &str, environment: &str, interval: i32) {
        self.optimization_memiavl_snapshot_interval
            .with_label_values(&[container, node, environment])
            .set(interval as f64);
    }

    /// 更新 Zero-Copy 启用状态
    pub fn set_optimization_zero_copy_enabled(&self, container: &str, node: &str, environment: &str, enabled: bool) {
        self.optimization_zero_copy_enabled
            .with_label_values(&[container, node, environment])
            .set(if enabled { 1.0 } else { 0.0 });
    }

    /// 批量更新优化特性指标（从 OptimizationMetrics）
    pub fn set_optimization_metrics(
        &self,
        metrics: &crate::collectors::OptimizationMetrics,
        node: &str,
        environment: &str,
    ) {
        self.set_optimization_block_stm_enabled(&metrics.container_name, node, environment, metrics.block_stm_enabled);
        if let Some(workers) = metrics.block_stm_workers {
            self.set_optimization_block_stm_workers(&metrics.container_name, node, environment, workers);
        }
        self.set_optimization_memiavl_enabled(&metrics.container_name, node, environment, metrics.memiavl_enabled);
        if let Some(cache_size) = metrics.memiavl_cache_size {
            self.set_optimization_memiavl_cache_size(&metrics.container_name, node, environment, cache_size);
        }
        if let Some(interval) = metrics.memiavl_snapshot_interval {
            self.set_optimization_memiavl_snapshot_interval(&metrics.container_name, node, environment, interval);
        }
        self.set_optimization_zero_copy_enabled(&metrics.container_name, node, environment, metrics.zero_copy_enabled);
    }

    /// 更新当前 TPS
    pub fn set_tps_current(&self, node: &str, environment: &str, value: f64) {
        self.chain_tps_current
            .with_label_values(&[node, environment])
            .set(value);
    }

    /// 更新平均 TPS
    pub fn set_tps_average(&self, node: &str, environment: &str, value: f64) {
        self.chain_tps_average
            .with_label_values(&[node, environment])
            .set(value);
    }

    /// 更新峰值 TPS
    pub fn set_tps_peak(&self, node: &str, environment: &str, value: f64) {
        self.chain_tps_peak
            .with_label_values(&[node, environment])
            .set(value);
    }

    /// 更新总交易数
    pub fn set_total_transactions(&self, node: &str, environment: &str, value: f64) {
        self.chain_total_transactions
            .with_label_values(&[node, environment])
            .set(value);
    }

    /// 批量更新 TPS 指标（从 TpsMetrics）
    pub fn set_tps_metrics(
        &self,
        metrics: &crate::collectors::TpsMetrics,
        node: &str,
        environment: &str,
    ) {
        self.set_tps_current(node, environment, metrics.current_tps);
        self.set_tps_average(node, environment, metrics.average_tps);
        self.set_tps_peak(node, environment, metrics.peak_tps);
        self.set_total_transactions(node, environment, metrics.total_transactions as f64);
    }

    /// 批量更新 Mempool 指标
    pub fn set_mempool_metrics(
        &self,
        metrics: &crate::collectors::MempoolMetrics,
        node: &str,
        environment: &str,
    ) {
        self.mempool_unconfirmed_txs
            .with_label_values(&[node, environment])
            .set(metrics.unconfirmed_txs as f64);
        self.mempool_size_bytes
            .with_label_values(&[node, environment])
            .set(metrics.mempool_size_bytes as f64);
        self.mempool_total_txs
            .with_label_values(&[node, environment])
            .set(metrics.total_txs as f64);
    }

    /// 批量更新验证者指标
    pub fn set_validator_metrics(
        &self,
        metrics: &crate::collectors::ValidatorMetrics,
        node: &str,
        environment: &str,
    ) {
        self.validator_total
            .with_label_values(&[node, environment])
            .set(metrics.total_validators as f64);
        self.validator_online
            .with_label_values(&[node, environment])
            .set(metrics.online_validators as f64);
        self.validator_total_voting_power
            .with_label_values(&[node, environment])
            .set(metrics.total_voting_power as f64);
        self.validator_average_voting_power
            .with_label_values(&[node, environment])
            .set(metrics.average_voting_power);
    }

    /// 批量更新网络指标
    pub fn set_network_metrics(
        &self,
        metrics: &crate::collectors::NetworkMetrics,
        node: &str,
        environment: &str,
    ) {
        self.network_inbound_peers
            .with_label_values(&[node, environment])
            .set(metrics.inbound_peers as f64);
        self.network_outbound_peers
            .with_label_values(&[node, environment])
            .set(metrics.outbound_peers as f64);
        self.network_total_peers
            .with_label_values(&[node, environment])
            .set(metrics.total_peers as f64);
        self.network_listening_addresses
            .with_label_values(&[node, environment])
            .set(metrics.listening_addresses as f64);
    }

    /// 导出指标为 Prometheus 文本格式
    pub fn export(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        String::from_utf8(buffer).map_err(|e| crate::error::AppError::generic(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_registry_creation() {
        let registry = MetricsRegistry::new();
        assert!(registry.is_ok());
    }

    #[test]
    fn test_set_block_height() {
        let registry = MetricsRegistry::new().unwrap();
        registry.set_block_height("test-node", "devnet", "aiw3chain-devnet", 12345.0);

        let exported = registry.export().unwrap();
        assert!(exported.contains("aiw3_chain_block_height"));
        assert!(exported.contains("12345"));
    }

    #[test]
    fn test_export_metrics() {
        let registry = MetricsRegistry::new().unwrap();
        registry.set_block_height("test-node", "devnet", "aiw3chain-devnet", 100.0);
        registry.set_node_count("test-node", "devnet", 5.0);

        let exported = registry.export().unwrap();
        assert!(exported.contains("aiw3_chain_block_height"));
        assert!(exported.contains("aiw3_chain_node_count"));
    }

    #[test]
    fn test_all_metric_setters() {
        let registry = MetricsRegistry::new().unwrap();
        
        // 测试所有 setter 方法
        registry.set_block_height("node1", "prod", "chain1", 1000.0);
        registry.set_node_count("node1", "prod", 5.0);
        registry.set_tx_pool_size("node1", "prod", 10.0);
        registry.set_sync_status("node1", "prod", false);
        
        let output = registry.export().unwrap();
        
        assert!(output.contains("1000"));
        assert!(output.contains("5"));
        assert!(output.contains("10"));
    }

    #[test]
    fn test_export_format_compliance() {
        let registry = MetricsRegistry::new().unwrap();
        
        registry.set_block_height("test", "test", "test", 100.0);
        
        let output = registry.export().unwrap();
        
        // 验证 Prometheus 格式
        assert!(output.contains("# HELP"));
        assert!(output.contains("# TYPE"));
        assert!(output.contains("gauge"));
    }

    #[test]
    fn test_multiple_nodes() {
        let registry = MetricsRegistry::new().unwrap();
        
        // 为多个节点设置指标
        registry.set_block_height("node1", "prod", "chain1", 1000.0);
        registry.set_block_height("node2", "prod", "chain1", 2000.0);
        registry.set_block_height("node3", "dev", "chain2", 500.0);
        
        let output = registry.export().unwrap();
        
        // 验证所有节点的数据都存在
        assert!(output.contains("node=\"node1\""));
        assert!(output.contains("node=\"node2\""));
        assert!(output.contains("node=\"node3\""));
        assert!(output.contains("environment=\"prod\""));
        assert!(output.contains("environment=\"dev\""));
    }

    #[test]
    fn test_clone_works() {
        let registry = MetricsRegistry::new().unwrap();
        registry.set_block_height("test", "test", "test", 100.0);
        
        let cloned = registry.clone();
        
        // 克隆后的注册器应该能导出相同的数据
        let output1 = registry.export().unwrap();
        let output2 = cloned.export().unwrap();
        
        assert_eq!(output1, output2);
    }

    #[test]
    fn test_metric_update() {
        let registry = MetricsRegistry::new().unwrap();
        
        // 第一次设置
        registry.set_block_height("test", "test", "test", 100.0);
        let output1 = registry.export().unwrap();
        assert!(output1.contains("100"));
        
        // 更新值
        registry.set_block_height("test", "test", "test", 200.0);
        let output2 = registry.export().unwrap();
        assert!(output2.contains("200"));
    }
}

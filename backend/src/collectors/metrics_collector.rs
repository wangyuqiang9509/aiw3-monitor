// 指标采集协调器
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::collectors::blockchain_metrics::BlockchainMetricsCollector;
use crate::collectors::docker::{DockerLogsCollector, DockerStatsCollector};
use crate::collectors::rpc_client::RpcClient;
use crate::collectors::{ExecutionCalculator, LatencyCalculator, TpsCalculator};
use crate::config::{load_blockchain_config, BlockchainConfig};
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
    docker_collectors: HashMap<String, DockerStatsCollector>,
    docker_log_collectors: HashMap<String, DockerLogsCollector>,
    tps_calculators: Arc<RwLock<HashMap<String, TpsCalculator>>>,
    latency_calculators: Arc<RwLock<HashMap<String, LatencyCalculator>>>,
    blockchain_config: Arc<BlockchainConfig>,
    last_abci_info_time: Arc<RwLock<Option<Instant>>>,
}

impl MetricsCollector {
    /// 创建新的指标采集器
    pub fn new(pool: PgPool, metrics_registry: MetricsRegistry) -> Result<Self> {
        // 加载区块链静态配置
        let blockchain_config = Arc::new(load_blockchain_config()?);
        info!("Blockchain static configuration loaded successfully");

        // 设置网络目标配置指标
        metrics_registry
            .set_network_target_config(blockchain_config.p2p_config(), "aiw3chain-devnet");
        info!("Network target configuration metrics set");

        Ok(Self {
            node_store: NodeStore::new(pool.clone()),
            metrics_store: MetricsStore::new(pool),
            metrics_registry,
            docker_collectors: HashMap::new(),
            docker_log_collectors: HashMap::new(),
            tps_calculators: Arc::new(RwLock::new(HashMap::new())),
            latency_calculators: Arc::new(RwLock::new(HashMap::new())),
            blockchain_config,
            last_abci_info_time: Arc::new(RwLock::new(None)),
        })
    }

    /// 注册 Docker 容器监控（资源指标）
    pub fn register_docker_container(&mut self, node_name: String, container_name: String) {
        info!("Registering Docker container for stats: {} (node: {})", container_name, node_name);
        let collector = DockerStatsCollector::new(container_name);
        self.docker_collectors.insert(node_name, collector);
    }

    /// 注册 Docker 容器日志监控（优化特性）
    pub fn register_docker_logs(&mut self, node_name: String, container_name: String) {
        info!("Registering Docker container for logs: {} (node: {})", container_name, node_name);
        let collector = DockerLogsCollector::new(container_name);
        self.docker_log_collectors.insert(node_name, collector);
    }

    /// 注册 TPS 计算器（默认 60 秒窗口）
    pub async fn register_tps_calculator(&self, node_name: String) {
        self.register_tps_calculator_with_window(node_name, 60).await;
    }

    /// 注册 TPS 计算器（自定义窗口大小）
    pub async fn register_tps_calculator_with_window(&self, node_name: String, window_size: u64) {
        info!("Registering TPS calculator for node: {} (window: {}s)", node_name, window_size);
        let calculator = TpsCalculator::new(window_size);
        self.tps_calculators.write().await.insert(node_name, calculator);
    }

    /// 注册延迟计算器（默认 10 个区块窗口）
    pub async fn register_latency_calculator(&self, node_name: String) {
        self.register_latency_calculator_with_window(node_name, 10).await;
    }

    /// 注册延迟计算器（自定义窗口大小）
    pub async fn register_latency_calculator_with_window(
        &self,
        node_name: String,
        window_size: usize,
    ) {
        info!(
            "Registering latency calculator for node: {} (window: {} blocks)",
            node_name, window_size
        );
        let calculator = LatencyCalculator::new(window_size);
        self.latency_calculators.write().await.insert(node_name, calculator);
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
                    let chain_id =
                        node.labels.get("chain_id").map(|s| s.as_str()).unwrap_or("unknown");
                    self.metrics_registry.set_block_height(
                        &node.name,
                        &node.environment,
                        chain_id,
                        height as f64,
                    );

                    // ===== 获取区块详情并计算 TPS 和延迟 =====
                    match client.get_latest_block_with_latency().await {
                        Ok((block_result, api_latency_ms)) => {
                            let tx_count = block_result.tx_count();

                            debug!(
                                "Block {} has {} transactions, API latency: {:.2}ms",
                                height, tx_count, api_latency_ms
                            );

                            // 记录 API 延迟
                            self.metrics_registry.set_api_latency(
                                &node.name,
                                &node.environment,
                                "block",
                                api_latency_ms,
                            );

                            // 存储 API 延迟到数据库
                            metrics.push(MetricData::new(
                                node.id,
                                MetricType::ApiLatency,
                                api_latency_ms,
                                Some(serde_json::json!({"endpoint": "block"})),
                            ));

                            // 提取区块时间戳用于延迟计算
                            if let Ok(block_timestamp_str) = block_result.block_timestamp() {
                                if let Ok(block_timestamp) =
                                    chrono::DateTime::parse_from_rfc3339(&block_timestamp_str)
                                {
                                    let block_timestamp_utc =
                                        block_timestamp.with_timezone(&chrono::Utc);

                                    // 更新延迟计算器
                                    let mut latency_calculators =
                                        self.latency_calculators.write().await;
                                    if let Some(calculator) =
                                        latency_calculators.get_mut(&node.name)
                                    {
                                        calculator.add_block(height, block_timestamp_utc);

                                        // 计算延迟指标
                                        let latency_metrics = calculator.calculate_latency();

                                        if latency_metrics.total_blocks > 0 {
                                            debug!(
                                                "Latency metrics for {}: current={:.3}s, avg={:.3}s, min={:.3}s, max={:.3}s",
                                                node.name,
                                                latency_metrics.current_block_time,
                                                latency_metrics.average_block_time,
                                                latency_metrics.min_block_time,
                                                latency_metrics.max_block_time
                                            );

                                            // 更新 Prometheus 指标
                                            self.metrics_registry.set_latency_metrics(
                                                &latency_metrics,
                                                &node.name,
                                                &node.environment,
                                            );

                                            // 存储区块时间到数据库
                                            metrics.push(MetricData::new(
                                                node.id,
                                                MetricType::BlockTime,
                                                latency_metrics.average_block_time,
                                                Some(serde_json::json!({
                                                    "current": latency_metrics.current_block_time,
                                                    "min": latency_metrics.min_block_time,
                                                    "max": latency_metrics.max_block_time,
                                                    "std_dev": latency_metrics.std_deviation,
                                                    "window_size": latency_metrics.window_size,
                                                    "total_blocks": latency_metrics.total_blocks,
                                                })),
                                            ));
                                        }
                                    } else {
                                        debug!(
                                            "Latency calculator not found for node: {}",
                                            node.name
                                        );
                                    }
                                }
                            }

                            // 将区块数据输入 TPS 计算器
                            let mut tps_calculators = self.tps_calculators.write().await;
                            if let Some(calculator) = tps_calculators.get_mut(&node.name) {
                                calculator.add_block(height, tx_count);

                                // 计算 TPS 指标
                                let tps_metrics = calculator.calculate_tps();

                                debug!(
                                    "TPS metrics for {}: current={:.2}, avg={:.2}, peak={:.2}",
                                    node.name,
                                    tps_metrics.current_tps,
                                    tps_metrics.average_tps,
                                    tps_metrics.peak_tps
                                );

                                // 更新 Prometheus 指标
                                self.metrics_registry.set_tps_metrics(
                                    &tps_metrics,
                                    &node.name,
                                    &node.environment,
                                );

                                // 存储 TPS 到数据库
                                metrics.push(MetricData::new(
                                    node.id,
                                    MetricType::Tps,
                                    tps_metrics.current_tps,
                                    Some(serde_json::json!({
                                        "average_tps": tps_metrics.average_tps,
                                        "peak_tps": tps_metrics.peak_tps,
                                        "total_transactions": tps_metrics.total_transactions,
                                        "window_size": tps_metrics.window_size,
                                    })),
                                ));
                            } else {
                                warn!("TPS calculator not found for node: {}", node.name);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to get block details from {}: {}", node.name, e);
                        }
                    }
                    // ===== TPS 和延迟计算结束 =====
                }

                // 同步状态
                let catching_up = status.sync_info.catching_up;
                self.metrics_registry.set_sync_status(&node.name, &node.environment, catching_up);
            }
            Err(e) => {
                warn!("Failed to get status from {}: {}", node.name, e);
            }
        }

        // 采集网络指标（完整版本）
        let blockchain_collector_for_network =
            BlockchainMetricsCollector::new(node.rpc_url.clone());
        match blockchain_collector_for_network.collect_network_metrics().await {
            Ok(network_metrics) => {
                debug!(
                    "Collected network metrics for {}: total={}, inbound={}, outbound={}, listeners={}",
                    node.name,
                    network_metrics.total_peers,
                    network_metrics.inbound_peers,
                    network_metrics.outbound_peers,
                    network_metrics.listening_addresses
                );

                // 更新 Prometheus 指标
                self.metrics_registry.set_network_metrics(
                    &network_metrics,
                    &node.name,
                    &node.environment,
                );

                // 存储到数据库
                metrics.push(MetricData::new(
                    node.id,
                    MetricType::NodeCount,
                    network_metrics.total_peers as f64,
                    Some(serde_json::json!({
                        "inbound_peers": network_metrics.inbound_peers,
                        "outbound_peers": network_metrics.outbound_peers,
                        "listening_addresses": network_metrics.listening_addresses,
                    })),
                ));
            }
            Err(e) => {
                warn!("Failed to collect network metrics from {}: {}", node.name, e);
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
                    self.metrics_registry.set_tx_pool_size(
                        &node.name,
                        &node.environment,
                        tx_count as f64,
                    );
                }
            }
            Err(e) => {
                warn!("Failed to get unconfirmed_txs from {}: {}", node.name, e);
            }
        }

        // 采集验证者指标
        let blockchain_collector = BlockchainMetricsCollector::new(node.rpc_url.clone());
        match blockchain_collector.collect_validator_metrics().await {
            Ok(validator_metrics) => {
                debug!(
                    "Collected validator metrics for {}: total={}, online={}, total_power={}, avg_power={}",
                    node.name,
                    validator_metrics.total_validators,
                    validator_metrics.online_validators,
                    validator_metrics.total_voting_power,
                    validator_metrics.average_voting_power
                );

                // 更新 Prometheus 指标
                self.metrics_registry.set_validator_metrics(
                    &validator_metrics,
                    &node.name,
                    &node.environment,
                );

                // TODO: 将验证者指标存储到数据库
            }
            Err(e) => {
                warn!("Failed to collect validator metrics from {}: {}", node.name, e);
            }
        }

        // 采集共识状态指标
        match blockchain_collector.collect_consensus_metrics().await {
            Ok(consensus_metrics) => {
                debug!(
                    "Collected consensus metrics for {}: height={}, round={}, step={}, proposer={}",
                    node.name,
                    consensus_metrics.height,
                    consensus_metrics.round,
                    consensus_metrics.step,
                    consensus_metrics.proposer_address
                );

                // 更新 Prometheus 指标
                self.metrics_registry.set_consensus_metrics(
                    &consensus_metrics,
                    &node.name,
                    &node.environment,
                );

                // TODO: 将共识状态指标存储到数据库
            }
            Err(e) => {
                warn!("Failed to collect consensus metrics from {}: {}", node.name, e);
            }
        }

        // 采集 Docker 资源指标
        if let Some(docker_collector) = self.docker_collectors.get(&node.name) {
            match docker_collector.collect().await {
                Ok(resource_metrics) => {
                    debug!(
                        "Collected Docker stats for {}: CPU={:.2}%, Memory={:.2}%",
                        node.name, resource_metrics.cpu_percent, resource_metrics.memory_percent
                    );

                    // 更新 Prometheus 指标
                    self.metrics_registry.set_resource_metrics(
                        &resource_metrics,
                        &node.name,
                        &node.environment,
                    );

                    // TODO: 将资源指标存储到数据库
                    // 可以扩展 MetricType 枚举来支持资源指标类型
                }
                Err(e) => {
                    warn!("Failed to collect Docker stats for {}: {}", node.name, e);
                }
            }
        }

        // 采集 Docker 日志（优化特性）
        if let Some(log_collector) = self.docker_log_collectors.get(&node.name) {
            match log_collector.collect().await {
                Ok(optimization_metrics) => {
                    debug!(
                        "Collected optimization metrics for {}: Block-STM={}, MemIAVL={}",
                        node.name,
                        optimization_metrics.block_stm_enabled,
                        optimization_metrics.memiavl_enabled
                    );

                    // 更新 Prometheus 指标
                    self.metrics_registry.set_optimization_metrics(
                        &optimization_metrics,
                        &node.name,
                        &node.environment,
                    );

                    // TODO: 将优化特性指标存储到数据库
                }
                Err(e) => {
                    warn!("Failed to collect Docker logs for {}: {}", node.name, e);
                }
            }
        }

        // ===== 采集执行指标 =====

        // 1. 设置静态配置指标（每次采集都更新，确保指标存在）
        let chain_id = node.labels.get("chain_id").map(|s| s.as_str()).unwrap_or("unknown");
        self.metrics_registry.set_blockchain_static_config(
            &self.blockchain_config,
            &node.name,
            &node.environment,
            chain_id,
        );
        debug!("Set blockchain static config metrics for {}", node.name);

        // 2. 每小时采集一次 ABCI Info
        let should_collect_abci = {
            let last_time = self.last_abci_info_time.read().await;
            match *last_time {
                None => true,
                Some(t) => t.elapsed() >= Duration::from_secs(3600),
            }
        };

        if should_collect_abci {
            match client.get_abci_info().await {
                Ok(abci_info) => {
                    info!(
                        "Collected ABCI info from {}: version={}, last_block_height={}",
                        node.name,
                        abci_info.version(),
                        abci_info.last_block_height().unwrap_or(0)
                    );

                    // 更新 Prometheus 指标
                    self.metrics_registry.set_abci_info(&abci_info, &node.name, &node.environment);

                    // 更新采集时间
                    *self.last_abci_info_time.write().await = Some(Instant::now());
                }
                Err(e) => {
                    warn!("Failed to get ABCI info from {}: {}", node.name, e);
                }
            }
        }

        // 3. 计算执行性能（基于已采集的 TPS 和延迟数据）
        let tps_calculators = self.tps_calculators.read().await;
        let latency_calculators = self.latency_calculators.read().await;

        if let (Some(tps_calc), Some(latency_calc)) =
            (tps_calculators.get(&node.name), latency_calculators.get(&node.name))
        {
            let tps_metrics = tps_calc.calculate_tps();
            let latency_metrics = latency_calc.calculate_latency();

            // 只有当有足够的数据时才计算执行性能
            if tps_metrics.total_transactions > 0 && latency_metrics.total_blocks > 0 {
                let execution_performance = ExecutionCalculator::calculate(
                    tps_metrics.current_tps,
                    latency_metrics.average_block_time,
                    self.blockchain_config.execution_metrics.block_stm.workers,
                );

                debug!(
                    "Execution performance for {}: TPS={:.2}, speedup={:.2}x, parallelism={:.1}%",
                    node.name,
                    execution_performance.actual_tps,
                    execution_performance.speedup,
                    execution_performance.estimated_parallelism_rate
                );

                // 更新 Prometheus 指标
                self.metrics_registry.set_execution_performance(
                    &execution_performance,
                    &node.name,
                    &node.environment,
                );

                // 注意：执行性能指标主要通过 Prometheus 暴露，不存储到数据库
                // 因为这些是计算得出的指标，而不是原始采集的指标
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
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:secret@localhost:5432/aiw3_monitor_test".to_string()
        });

        let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
        let registry = MetricsRegistry::new().unwrap();
        let collector = MetricsCollector::new(pool, registry).unwrap();

        let result = collector.collect_all().await;
        assert!(result.is_ok());
    }
}

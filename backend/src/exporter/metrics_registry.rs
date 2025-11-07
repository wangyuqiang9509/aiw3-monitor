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
    pub block_height: GaugeVec,
    pub block_time: GaugeVec,
    pub node_count: GaugeVec,
    pub tx_pool_size: GaugeVec,
    pub network_latency: GaugeVec,
    pub validator_count: GaugeVec,
    pub tps: GaugeVec,
    pub sync_status: GaugeVec,
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

        info!("Prometheus metrics registry initialized with 8 metrics");

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

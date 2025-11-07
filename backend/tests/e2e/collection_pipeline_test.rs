// 端到端集成测试: 数据采集管道
// 测试从 RPC 采集 → 存储 → Prometheus 导出的完整流程

use aiw3_monitor::collectors::metrics_collector::MetricsCollector;
use aiw3_monitor::collectors::rpc_client::RpcClient;
use aiw3_monitor::exporter::metrics_registry::MetricsRegistry;
use aiw3_monitor::models::node::BlockchainNode;
use aiw3_monitor::storage::metrics_store::MetricsStore;
use aiw3_monitor::storage::node_store::NodeStore;
use sqlx::PgPool;
use std::collections::HashMap;

/// 辅助函数: 获取测试数据库连接
async fn get_test_pool() -> Option<PgPool> {
    let database_url = std::env::var("DATABASE_URL").ok()?;
    PgPool::connect(&database_url).await.ok()
}

/// 辅助函数: 创建测试节点
fn create_test_node() -> BlockchainNode {
    let mut labels = HashMap::new();
    labels.insert("test".to_string(), "true".to_string());
    
    BlockchainNode {
        id: 1,
        name: "Test Node".to_string(),
        rpc_url: "https://devnet-rpc3.aiw3.io".to_string(),
        rest_url: "https://devnet-api3.aiw3.io".to_string(),
        grpc_url: "https://devnet-grpc3.aiw3.io:443".to_string(),
        environment: "test".to_string(),
        enabled: true,
        labels,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[tokio::test]
#[ignore] // 需要数据库和网络连接
async fn test_end_to_end_collection_pipeline() {
    // 1. 设置测试环境
    let pool = get_test_pool().await.expect("DATABASE_URL not set");
    let metrics_registry = MetricsRegistry::new().unwrap();
    
    // 2. 创建采集器
    let collector = MetricsCollector::new(pool.clone(), metrics_registry.clone());
    
    // 3. 执行采集
    let result = collector.collect_all().await;
    assert!(result.is_ok(), "数据采集应该成功");
    
    let collected_count = result.unwrap();
    assert!(collected_count > 0, "应该采集到至少一个指标");
    
    // 4. 验证数据已存储到数据库
    let metrics_store = MetricsStore::new(pool.clone());
    let latest_metrics = metrics_store
        .get_latest(1, aiw3_monitor::models::metric::MetricType::BlockHeight, 10)
        .await;
    
    assert!(latest_metrics.is_ok(), "应该能从数据库读取指标");
    assert!(!latest_metrics.unwrap().is_empty(), "数据库中应该有指标数据");
    
    // 5. 验证 Prometheus 指标已更新
    let exported = metrics_registry.export().unwrap();
    assert!(exported.contains("aiw3_chain_block_height"), "应该包含区块高度指标");
    assert!(exported.contains("aiw3_chain_node_count"), "应该包含节点数量指标");
}

#[tokio::test]
#[ignore]
async fn test_rpc_client_real_endpoint() {
    // 测试真实 RPC 端点
    let client = RpcClient::new(
        "https://devnet-rpc3.aiw3.io".to_string(),
        10,
        3,
    );
    
    // 测试获取状态
    let status_result = client.get_status().await;
    assert!(status_result.is_ok(), "应该能获取节点状态");
    
    let status = status_result.unwrap();
    assert!(!status.node_info.network.is_empty(), "网络名称不应为空");
    assert!(!status.sync_info.latest_block_height.is_empty(), "区块高度不应为空");
    
    // 测试获取网络信息
    let net_info_result = client.get_net_info().await;
    assert!(net_info_result.is_ok(), "应该能获取网络信息");
    
    // 测试获取未确认交易
    let txs_result = client.get_num_unconfirmed_txs().await;
    assert!(txs_result.is_ok(), "应该能获取未确认交易数");
}

#[tokio::test]
#[ignore]
async fn test_metrics_persistence() {
    // 测试指标持久化
    let pool = get_test_pool().await.expect("DATABASE_URL not set");
    let metrics_store = MetricsStore::new(pool.clone());
    
    // 插入测试指标
    let test_metric = aiw3_monitor::models::metric::MetricData {
        id: 0,
        node_id: 1,
        metric_type: aiw3_monitor::models::metric::MetricType::BlockHeight,
        metric_value: 99999.0,
        collected_at: chrono::Utc::now(),
        metadata: None,
    };
    
    let insert_result = metrics_store.insert(&test_metric).await;
    assert!(insert_result.is_ok(), "应该能插入指标");
    
    // 等待一小段时间确保数据写入
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // 读取并验证
    let latest = metrics_store
        .get_latest(1, aiw3_monitor::models::metric::MetricType::BlockHeight, 1)
        .await
        .unwrap();
    
    assert!(!latest.is_empty(), "应该能读取到刚插入的指标");
    assert_eq!(latest[0].metric_value, 99999.0, "指标值应该匹配");
}

#[tokio::test]
#[ignore]
async fn test_node_store_operations() {
    // 测试节点存储操作
    let pool = get_test_pool().await.expect("DATABASE_URL not set");
    let node_store = NodeStore::new(pool);
    
    // 获取启用的节点
    let nodes_result = node_store.get_enabled_nodes().await;
    assert!(nodes_result.is_ok(), "应该能获取启用的节点");
    
    let nodes = nodes_result.unwrap();
    assert!(!nodes.is_empty(), "应该至少有一个启用的节点");
    
    // 验证节点数据
    for node in nodes {
        assert!(!node.name.is_empty(), "节点名称不应为空");
        assert!(!node.rpc_url.is_empty(), "RPC URL 不应为空");
        assert!(node.enabled, "节点应该是启用状态");
    }
}

#[tokio::test]
#[ignore]
async fn test_prometheus_export_after_collection() {
    // 测试采集后的 Prometheus 导出
    let pool = get_test_pool().await.expect("DATABASE_URL not set");
    let metrics_registry = MetricsRegistry::new().unwrap();
    
    // 手动设置一些指标
    metrics_registry.set_block_height("test-node", "test", "test-chain", 12345.0);
    metrics_registry.set_node_count("test-node", "test", 5.0);
    metrics_registry.set_tx_pool_size("test-node", "test", 10.0);
    metrics_registry.set_sync_status("test-node", "test", 1.0);
    
    // 导出并验证
    let exported = metrics_registry.export().unwrap();
    
    // 验证所有指标都存在
    assert!(exported.contains("aiw3_chain_block_height{"));
    assert!(exported.contains("aiw3_chain_node_count{"));
    assert!(exported.contains("aiw3_chain_tx_pool_size{"));
    assert!(exported.contains("aiw3_chain_sync_status{"));
    
    // 验证值
    assert!(exported.contains("12345"));
    assert!(exported.contains("5"));
    assert!(exported.contains("10"));
    
    // 验证标签
    assert!(exported.contains("node=\"test-node\""));
    assert!(exported.contains("environment=\"test\""));
}

#[tokio::test]
#[ignore]
async fn test_concurrent_collections() {
    // 测试并发采集
    let pool = get_test_pool().await.expect("DATABASE_URL not set");
    let metrics_registry = MetricsRegistry::new().unwrap();
    
    let collector = MetricsCollector::new(pool.clone(), metrics_registry.clone());
    
    // 启动多个并发采集任务
    let mut handles = vec![];
    for _ in 0..3 {
        let collector_clone = collector.clone();
        let handle = tokio::spawn(async move {
            collector_clone.collect_all().await
        });
        handles.push(handle);
    }
    
    // 等待所有任务完成
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok(), "并发采集任务应该成功");
        assert!(result.unwrap().is_ok(), "采集应该成功");
    }
}

#[tokio::test]
async fn test_metrics_collector_creation() {
    // 测试采集器创建(不需要数据库)
    use sqlx::postgres::PgPoolOptions;
    
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgres://localhost/test")
        .unwrap();
    
    let metrics_registry = MetricsRegistry::new().unwrap();
    let _collector = MetricsCollector::new(pool, metrics_registry);
    
    // 如果能创建,测试通过
}


// 测试优化特性指标
use aiw3_monitor::config::load_blockchain_config;
use aiw3_monitor::exporter::metrics_registry::MetricsRegistry;

fn main() {
    println!("=== 测试优化特性指标 ===\n");
    
    // 1. 加载区块链配置
    println!("1. 加载区块链配置...");
    let config = load_blockchain_config().expect("Failed to load blockchain config");
    println!("   ✓ 配置加载成功");
    println!("   - Block-STM: {}", config.execution_metrics.block_stm.enabled);
    println!("   - MemIAVL: {}", config.storage_metrics.memiavl.enabled);
    println!("   - Optimistic Execution: {}", config.is_optimistic_execution_enabled());
    println!("   - Large Blocks: {}", config.is_large_blocks_enabled());
    println!("   - Max Block Size: {} MB", config.max_block_size_mb());
    println!("   - Zero-Copy: {}\n", config.is_zero_copy_enabled());
    
    // 2. 创建 MetricsRegistry
    println!("2. 创建 MetricsRegistry...");
    let registry = MetricsRegistry::new().expect("Failed to create metrics registry");
    println!("   ✓ MetricsRegistry 创建成功\n");
    
    // 3. 设置静态配置指标
    println!("3. 设置静态配置指标...");
    registry.set_blockchain_static_config(&config, "test-node", "devnet", "aiw3chain-devnet");
    println!("   ✓ 静态配置指标设置成功\n");
    
    // 4. 导出指标并验证
    println!("4. 导出并验证指标...");
    let metrics = registry.export().expect("Failed to export metrics");
    
    // 验证新增的优化特性指标
    let checks = vec![
        ("aiw3_blockchain_optimistic_execution_enabled", "Optimistic Execution 启用状态"),
        ("aiw3_blockchain_large_blocks_enabled", "Large Blocks 启用状态"),
        ("aiw3_blockchain_large_blocks_max_size_mb", "Large Blocks 最大大小"),
        ("aiw3_blockchain_optimization_features_info", "优化特性统一摘要"),
    ];
    
    println!("   验证新增指标:");
    let mut all_found = true;
    for (metric_name, description) in checks {
        if metrics.contains(metric_name) {
            println!("   ✓ {} - {}", metric_name, description);
        } else {
            println!("   ✗ {} - {} (未找到)", metric_name, description);
            all_found = false;
        }
    }
    
    if all_found {
        println!("\n✅ 所有新增指标都已正确暴露！");
    } else {
        println!("\n❌ 部分指标未找到");
    }
    
    // 显示优化特性相关的指标
    println!("\n=== 优化特性指标详情 ===");
    for line in metrics.lines() {
        if line.contains("optimization") || line.contains("optimistic_execution") || line.contains("large_blocks") {
            println!("{}", line);
        }
    }
    
    println!("\n=== 测试完成 ===");
}


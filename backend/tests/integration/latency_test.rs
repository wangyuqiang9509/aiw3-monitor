// 延迟监控集成测试
use aiw3_monitor::collectors::{LatencyCalculator, LatencyMetrics};
use aiw3_monitor::exporter::metrics_registry::MetricsRegistry;
use aiw3_monitor::models::metric::MetricType;
use chrono::{TimeZone, Utc};

#[test]
fn test_latency_calculator_integration() {
    let mut calculator = LatencyCalculator::new(10);
    
    // 模拟添加多个区块
    let base_time = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    
    // 添加 15 个区块，每个间隔 1.8 秒（接近目标 1.78 秒）
    for i in 0..15 {
        let timestamp = base_time + chrono::Duration::milliseconds((i as i64) * 1800);
        calculator.add_block(100 + i, timestamp);
    }
    
    // 计算延迟指标
    let metrics = calculator.calculate_latency();
    
    // 验证窗口大小
    assert_eq!(metrics.window_size, 10);
    
    // 验证有效区块数（应该是 10 个，因为窗口大小是 10）
    assert_eq!(metrics.total_blocks, 10);
    
    // 验证平均区块时间（应该接近 1.8 秒）
    assert!((metrics.average_block_time - 1.8).abs() < 0.01);
    
    // 验证最小和最大值
    assert_eq!(metrics.min_block_time, 1.8);
    assert_eq!(metrics.max_block_time, 1.8);
    
    // 验证标准差（所有区块时间相同，标准差应该是 0）
    assert_eq!(metrics.std_deviation, 0.0);
}

#[test]
fn test_latency_calculator_with_varying_block_times() {
    let mut calculator = LatencyCalculator::new(10);
    
    let base_time = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    
    // 添加区块，间隔时间变化：1s, 2s, 3s, 2s, 1s, ...
    let intervals = vec![1000, 2000, 3000, 2000, 1000, 2000, 3000, 2000, 1000, 2000, 3000];
    let mut cumulative_ms = 0i64;
    
    for (i, interval_ms) in intervals.iter().enumerate() {
        let timestamp = base_time + chrono::Duration::milliseconds(cumulative_ms);
        calculator.add_block(100 + i as u64, timestamp);
        cumulative_ms += interval_ms;
    }
    
    let metrics = calculator.calculate_latency();
    
    // 验证有效区块数
    assert_eq!(metrics.total_blocks, 10);
    
    // 验证平均值（(1+2+3+2+1+2+3+2+1+2)/10 = 1.9）
    assert!((metrics.average_block_time - 1.9).abs() < 0.01);
    
    // 验证最小和最大值
    assert_eq!(metrics.min_block_time, 1.0);
    assert_eq!(metrics.max_block_time, 3.0);
    
    // 验证标准差（应该大于 0）
    assert!(metrics.std_deviation > 0.0);
}

#[test]
fn test_metrics_registry_latency_metrics() {
    let registry = MetricsRegistry::new().expect("Failed to create registry");
    
    // 创建测试指标
    let metrics = LatencyMetrics {
        current_block_time: 1.78,
        average_block_time: 1.80,
        min_block_time: 1.50,
        max_block_time: 2.50,
        std_deviation: 0.25,
        window_size: 10,
        total_blocks: 10,
    };
    
    // 设置指标
    registry.set_latency_metrics(&metrics, "test-node", "devnet");
    
    // 导出并验证
    let exported = registry.export().expect("Failed to export metrics");
    
    // 验证指标存在
    assert!(exported.contains("aiw3_chain_block_time_current_seconds"));
    assert!(exported.contains("aiw3_chain_block_time_average_seconds"));
    assert!(exported.contains("aiw3_chain_block_time_min_seconds"));
    assert!(exported.contains("aiw3_chain_block_time_max_seconds"));
    assert!(exported.contains("aiw3_chain_block_time_std_dev_seconds"));
    
    // 验证标签
    assert!(exported.contains("node=\"test-node\""));
    assert!(exported.contains("environment=\"devnet\""));
    
    // 验证值（大致检查）
    assert!(exported.contains("1.78") || exported.contains("1.8"));
}

#[test]
fn test_api_latency_metrics() {
    let registry = MetricsRegistry::new().expect("Failed to create registry");
    
    // 设置 API 延迟指标
    registry.set_api_latency("test-node", "devnet", "status", 45.2);
    registry.set_api_latency("test-node", "devnet", "block", 67.8);
    registry.set_api_latency("test-node", "devnet", "net_info", 38.5);
    
    // 导出并验证
    let exported = registry.export().expect("Failed to export metrics");
    
    // 验证指标存在
    assert!(exported.contains("aiw3_api_latency_ms"));
    
    // 验证端点标签
    assert!(exported.contains("endpoint=\"status\""));
    assert!(exported.contains("endpoint=\"block\""));
    assert!(exported.contains("endpoint=\"net_info\""));
    
    // 验证值存在
    assert!(exported.contains("45.2") || exported.contains("45"));
    assert!(exported.contains("67.8") || exported.contains("67"));
    assert!(exported.contains("38.5") || exported.contains("38"));
}

#[test]
fn test_metric_type_enum() {
    // 验证 MetricType 枚举包含延迟相关类型
    let block_time = MetricType::BlockTime;
    let api_latency = MetricType::ApiLatency;
    
    // 确保可以序列化
    let block_time_str = format!("{:?}", block_time);
    let api_latency_str = format!("{:?}", api_latency);
    
    assert!(block_time_str.contains("BlockTime"));
    assert!(api_latency_str.contains("ApiLatency"));
}

#[test]
fn test_latency_calculator_edge_cases() {
    let mut calculator = LatencyCalculator::new(10);
    
    // 测试空计算器
    let empty_metrics = calculator.calculate_latency();
    assert_eq!(empty_metrics.total_blocks, 0);
    assert_eq!(empty_metrics.average_block_time, 0.0);
    
    // 添加单个区块
    let timestamp = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    calculator.add_block(100, timestamp);
    
    let single_metrics = calculator.calculate_latency();
    assert_eq!(single_metrics.total_blocks, 0); // 单个区块没有时间差
    
    // 添加第二个区块
    calculator.add_block(101, timestamp + chrono::Duration::seconds(2));
    
    let two_metrics = calculator.calculate_latency();
    assert_eq!(two_metrics.total_blocks, 1);
    assert_eq!(two_metrics.current_block_time, 2.0);
}

#[test]
fn test_latency_calculator_time_precision() {
    let mut calculator = LatencyCalculator::new(10);
    
    let base_time = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    
    // 测试毫秒级精度
    calculator.add_block(100, base_time);
    calculator.add_block(101, base_time + chrono::Duration::milliseconds(1780)); // 1.78 秒
    
    let metrics = calculator.calculate_latency();
    assert_eq!(metrics.total_blocks, 1);
    assert_eq!(metrics.current_block_time, 1.78);
}

#[test]
fn test_latency_calculator_sliding_window() {
    let mut calculator = LatencyCalculator::new(3); // 小窗口便于测试
    
    let base_time = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    
    // 添加 6 个区块
    for i in 0..6 {
        let timestamp = base_time + chrono::Duration::seconds(i * 2);
        calculator.add_block(100 + i as u64, timestamp);
    }
    
    // 窗口大小是 3，所以应该只保留最近 3 个有效的区块时间
    let metrics = calculator.calculate_latency();
    assert_eq!(metrics.window_size, 3);
    assert_eq!(metrics.total_blocks, 3);
    
    // 所有区块时间都是 2 秒
    assert_eq!(metrics.average_block_time, 2.0);
    assert_eq!(metrics.current_block_time, 2.0);
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn test_latency_calculator_performance() {
        let mut calculator = LatencyCalculator::new(10);
        let base_time = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        
        // 预填充数据
        for i in 0..10 {
            let timestamp = base_time + chrono::Duration::seconds(i * 2);
            calculator.add_block(100 + i, timestamp);
        }
        
        // 测试计算性能（应该非常快）
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = calculator.calculate_latency();
        }
        let duration = start.elapsed();
        
        // 1000 次计算应该在 100ms 内完成
        assert!(duration.as_millis() < 100, "Latency calculation too slow: {:?}", duration);
    }
    
    #[test]
    fn test_metrics_registry_performance() {
        let registry = MetricsRegistry::new().expect("Failed to create registry");
        
        let metrics = LatencyMetrics {
            current_block_time: 1.78,
            average_block_time: 1.80,
            min_block_time: 1.50,
            max_block_time: 2.50,
            std_deviation: 0.25,
            window_size: 10,
            total_blocks: 10,
        };
        
        // 测试设置指标的性能
        let start = Instant::now();
        for _ in 0..1000 {
            registry.set_latency_metrics(&metrics, "test-node", "devnet");
        }
        let duration = start.elapsed();
        
        // 1000 次设置应该在 50ms 内完成
        assert!(duration.as_millis() < 50, "Metrics setting too slow: {:?}", duration);
    }
}


// 测试 Docker Stats 采集器
use aiw3_monitor::collectors::DockerStatsCollector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("=== AIW3 Monitor - Docker Stats 采集器测试 ===\n");

    // 测试容器列表
    let containers = vec!["aiw3defi-validator1", "aiw3defi-validator2", "aiw3defi-validator3"];

    for container_name in containers {
        println!("📊 采集容器: {}", container_name);
        println!("{}", "=".repeat(60));

        let collector = DockerStatsCollector::new(container_name.to_string());

        match collector.collect().await {
            Ok(metrics) => {
                println!("✅ 采集成功!");
                println!();
                println!("容器名称: {}", metrics.container_name);
                println!("采集时间: {}", metrics.timestamp);
                println!();
                println!("📈 CPU 指标:");
                println!("  CPU 使用率: {:.2}%", metrics.cpu_percent);
                println!();
                println!("💾 内存指标:");
                println!(
                    "  内存使用: {:.2} GB / {:.2} GB",
                    metrics.memory_bytes as f64 / 1024.0 / 1024.0 / 1024.0,
                    metrics.memory_limit_bytes as f64 / 1024.0 / 1024.0 / 1024.0
                );
                println!("  内存使用率: {:.2}%", metrics.memory_percent);
                println!();
                println!("🌐 网络指标:");
                println!("  网络接收: {:.2} MB", metrics.network_rx_bytes as f64 / 1024.0 / 1024.0);
                println!("  网络发送: {:.2} MB", metrics.network_tx_bytes as f64 / 1024.0 / 1024.0);
                println!();
                println!("💿 磁盘指标:");
                println!("  磁盘读取: {:.2} MB", metrics.block_read_bytes as f64 / 1024.0 / 1024.0);
                println!(
                    "  磁盘写入: {:.2} MB",
                    metrics.block_write_bytes as f64 / 1024.0 / 1024.0
                );
                println!();

                // 告警检查
                println!("🚨 告警检查:");
                if metrics.cpu_percent > 700.0 {
                    println!("  ⚠️  WARNING: CPU 使用率过高 (> 700%)");
                } else if metrics.cpu_percent > 600.0 {
                    println!("  ⚡ INFO: CPU 使用率较高");
                } else {
                    println!("  ✅ CPU 使用率正常");
                }

                if metrics.memory_percent > 90.0 {
                    println!("  🔴 CRITICAL: 内存使用率过高 (> 90%)");
                } else if metrics.memory_percent > 70.0 {
                    println!("  ⚠️  WARNING: 内存使用率较高 (> 70%)");
                } else {
                    println!("  ✅ 内存使用率正常");
                }
            }
            Err(e) => {
                println!("❌ 采集失败: {}", e);
                println!("提示: 请确保容器正在运行");
            }
        }

        println!();
        println!("{}", "=".repeat(60));
        println!();
    }

    println!("✅ 测试完成!");
    Ok(())
}

// 简单的 Docker Stats 测试（不依赖数据库）
use std::process::Command;

fn main() {
    println!("=== Docker Stats 简单测试 ===\n");

    let containers = vec!["aiw3defi-validator1", "aiw3defi-validator2", "aiw3defi-validator3"];

    for container in containers {
        println!("测试容器: {}", container);

        let output = Command::new("docker")
            .args(&["stats", "--no-stream", "--format", "json", container])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    println!("✅ 成功!");
                    println!("输出: {}", stdout);

                    // 尝试解析 JSON
                    match serde_json::from_str::<serde_json::Value>(&stdout) {
                        Ok(json) => {
                            println!("JSON 解析成功:");
                            println!("  CPUPerc: {}", json["CPUPerc"].as_str().unwrap_or("N/A"));
                            println!("  MemUsage: {}", json["MemUsage"].as_str().unwrap_or("N/A"));
                            println!("  MemPerc: {}", json["MemPerc"].as_str().unwrap_or("N/A"));
                        }
                        Err(e) => {
                            println!("❌ JSON 解析失败: {}", e);
                        }
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("❌ 命令失败: {}", stderr);
                }
            }
            Err(e) => {
                println!("❌ 执行失败: {}", e);
            }
        }

        println!();
    }
}

// Docker Stats 资源监控采集器
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{debug, warn};

use crate::error::Result;

/// 资源使用指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// 容器名称
    pub container_name: String,
    /// CPU 使用率（百分比，可能超过 100%）
    pub cpu_percent: f64,
    /// 内存使用量（字节）
    pub memory_bytes: u64,
    /// 内存限制（字节）
    pub memory_limit_bytes: u64,
    /// 内存使用率（百分比）
    pub memory_percent: f64,
    /// 网络接收字节数
    pub network_rx_bytes: u64,
    /// 网络发送字节数
    pub network_tx_bytes: u64,
    /// 磁盘读取字节数
    pub block_read_bytes: u64,
    /// 磁盘写入字节数
    pub block_write_bytes: u64,
    /// 采集时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Docker Stats 采集器
pub struct DockerStatsCollector {
    container_name: String,
}

impl DockerStatsCollector {
    /// 创建新的 Docker Stats 采集器
    pub fn new(container_name: String) -> Self {
        Self { container_name }
    }

    /// 采集资源指标
    pub async fn collect(&self) -> Result<ResourceMetrics> {
        debug!("Collecting Docker stats for container: {}", self.container_name);

        // 执行 docker stats 命令
        let output = tokio::task::spawn_blocking({
            let container_name = self.container_name.clone();
            move || {
                Command::new("docker")
                    .args(&[
                        "stats",
                        "--no-stream",
                        "--format",
                        "json",
                        &container_name,
                    ])
                    .output()
            }
        })
        .await
        .map_err(|e| crate::error::AppError::generic(format!("Task join error: {}", e)))??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Docker stats command failed for {}: {}", self.container_name, stderr);
            return Err(crate::error::AppError::generic(format!(
                "Docker stats failed: {}",
                stderr
            )));
        }

        // 解析 JSON 输出
        let stats: serde_json::Value = serde_json::from_slice(&output.stdout)?;

        // 提取指标
        let metrics = ResourceMetrics {
            container_name: self.container_name.clone(),
            cpu_percent: parse_cpu_percent(stats["CPUPerc"].as_str().unwrap_or("0%"))?,
            memory_bytes: parse_memory_bytes(stats["MemUsage"].as_str().unwrap_or("0B / 0B"))?.0,
            memory_limit_bytes: parse_memory_bytes(stats["MemUsage"].as_str().unwrap_or("0B / 0B"))?.1,
            memory_percent: parse_percentage(stats["MemPerc"].as_str().unwrap_or("0%"))?,
            network_rx_bytes: parse_network_io(stats["NetIO"].as_str().unwrap_or("0B / 0B"))?.0,
            network_tx_bytes: parse_network_io(stats["NetIO"].as_str().unwrap_or("0B / 0B"))?.1,
            block_read_bytes: parse_block_io(stats["BlockIO"].as_str().unwrap_or("0B / 0B"))?.0,
            block_write_bytes: parse_block_io(stats["BlockIO"].as_str().unwrap_or("0B / 0B"))?.1,
            timestamp: chrono::Utc::now(),
        };

        debug!(
            "Collected metrics for {}: CPU={:.2}%, Memory={:.2}GB/{:.2}GB",
            self.container_name,
            metrics.cpu_percent,
            metrics.memory_bytes as f64 / 1024.0 / 1024.0 / 1024.0,
            metrics.memory_limit_bytes as f64 / 1024.0 / 1024.0 / 1024.0
        );

        Ok(metrics)
    }
}

// ============================================================================
// 辅助解析函数
// ============================================================================

/// 解析 CPU 百分比 "450.32%" -> 450.32
fn parse_cpu_percent(s: &str) -> Result<f64> {
    let cleaned = s.trim().trim_end_matches('%');
    cleaned
        .parse::<f64>()
        .map_err(|e| crate::error::AppError::generic(format!("Failed to parse CPU percent '{}': {}", s, e)))
}

/// 解析百分比 "40.59%" -> 40.59
fn parse_percentage(s: &str) -> Result<f64> {
    let cleaned = s.trim().trim_end_matches('%');
    cleaned
        .parse::<f64>()
        .map_err(|e| crate::error::AppError::generic(format!("Failed to parse percentage '{}': {}", s, e)))
}

/// 解析内存使用量 "3.247GiB / 8GiB" -> (3485989273, 8589934592)
fn parse_memory_bytes(s: &str) -> Result<(u64, u64)> {
    let parts: Vec<&str> = s.split(" / ").collect();
    if parts.len() != 2 {
        return Ok((0, 0));
    }

    let used = parse_size_to_bytes(parts[0])?;
    let limit = parse_size_to_bytes(parts[1])?;

    Ok((used, limit))
}

/// 解析网络 I/O "1.234MB / 3.456MB" -> (1293942, 3623878)
fn parse_network_io(s: &str) -> Result<(u64, u64)> {
    let parts: Vec<&str> = s.split(" / ").collect();
    if parts.len() != 2 {
        return Ok((0, 0));
    }

    let rx = parse_size_to_bytes(parts[0])?;
    let tx = parse_size_to_bytes(parts[1])?;

    Ok((rx, tx))
}

/// 解析磁盘 I/O "12.3MB / 45.6MB" -> (12897484, 47824691)
fn parse_block_io(s: &str) -> Result<(u64, u64)> {
    let parts: Vec<&str> = s.split(" / ").collect();
    if parts.len() != 2 {
        return Ok((0, 0));
    }

    let read = parse_size_to_bytes(parts[0])?;
    let write = parse_size_to_bytes(parts[1])?;

    Ok((read, write))
}

/// 解析大小字符串到字节数
/// 支持: B, KB, KiB, MB, MiB, GB, GiB, TB, TiB
fn parse_size_to_bytes(s: &str) -> Result<u64> {
    let s = s.trim();
    
    if s == "0B" || s == "0" {
        return Ok(0);
    }
    
    // 尝试匹配不同的单位
    if let Some(num_str) = s.strip_suffix("TiB") {
        let num = num_str.parse::<f64>()
            .map_err(|e| crate::error::AppError::generic(format!("Failed to parse size '{}': {}", s, e)))?;
        return Ok((num * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64);
    } else if let Some(num_str) = s.strip_suffix("TB") {
        let num = num_str.parse::<f64>()
            .map_err(|e| crate::error::AppError::generic(format!("Failed to parse size '{}': {}", s, e)))?;
        return Ok((num * 1000.0 * 1000.0 * 1000.0 * 1000.0) as u64);
    } else if let Some(num_str) = s.strip_suffix("GiB") {
        let num = num_str.parse::<f64>()
            .map_err(|e| crate::error::AppError::generic(format!("Failed to parse size '{}': {}", s, e)))?;
        return Ok((num * 1024.0 * 1024.0 * 1024.0) as u64);
    } else if let Some(num_str) = s.strip_suffix("GB") {
        let num = num_str.parse::<f64>()
            .map_err(|e| crate::error::AppError::generic(format!("Failed to parse size '{}': {}", s, e)))?;
        return Ok((num * 1000.0 * 1000.0 * 1000.0) as u64);
    } else if let Some(num_str) = s.strip_suffix("MiB") {
        let num = num_str.parse::<f64>()
            .map_err(|e| crate::error::AppError::generic(format!("Failed to parse size '{}': {}", s, e)))?;
        return Ok((num * 1024.0 * 1024.0) as u64);
    } else if let Some(num_str) = s.strip_suffix("MB") {
        let num = num_str.parse::<f64>()
            .map_err(|e| crate::error::AppError::generic(format!("Failed to parse size '{}': {}", s, e)))?;
        return Ok((num * 1000.0 * 1000.0) as u64);
    } else if let Some(num_str) = s.strip_suffix("KiB") {
        let num = num_str.parse::<f64>()
            .map_err(|e| crate::error::AppError::generic(format!("Failed to parse size '{}': {}", s, e)))?;
        return Ok((num * 1024.0) as u64);
    } else if let Some(num_str) = s.strip_suffix("KB") {
        let num = num_str.parse::<f64>()
            .map_err(|e| crate::error::AppError::generic(format!("Failed to parse size '{}': {}", s, e)))?;
        return Ok((num * 1000.0) as u64);
    } else if let Some(num_str) = s.strip_suffix('B') {
        let num = num_str.parse::<f64>()
            .map_err(|e| crate::error::AppError::generic(format!("Failed to parse size '{}': {}", s, e)))?;
        return Ok(num as u64);
    }
    
    // 如果没有单位，假设是字节
    s.parse::<u64>()
        .map_err(|e| crate::error::AppError::generic(format!("Failed to parse size '{}': {}", s, e)))
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpu_percent() {
        assert_eq!(parse_cpu_percent("450.32%").unwrap(), 450.32);
        assert_eq!(parse_cpu_percent("0%").unwrap(), 0.0);
        assert_eq!(parse_cpu_percent("100.5%").unwrap(), 100.5);
    }

    #[test]
    fn test_parse_percentage() {
        assert_eq!(parse_percentage("40.59%").unwrap(), 40.59);
        assert_eq!(parse_percentage("0%").unwrap(), 0.0);
        assert_eq!(parse_percentage("99.99%").unwrap(), 99.99);
    }

    #[test]
    fn test_parse_memory_bytes() {
        let (used, limit) = parse_memory_bytes("3.247GiB / 8GiB").unwrap();
        assert!(used > 3_000_000_000);
        assert!(used < 4_000_000_000);
        assert_eq!(limit, 8 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_parse_network_io() {
        let (rx, tx) = parse_network_io("1.234MB / 3.456MB").unwrap();
        assert!(rx > 1_000_000);
        assert!(rx < 2_000_000);
        assert!(tx > 3_000_000);
        assert!(tx < 4_000_000);
    }

    #[test]
    fn test_parse_size_to_bytes() {
        assert_eq!(parse_size_to_bytes("0B").unwrap(), 0);
        assert_eq!(parse_size_to_bytes("1B").unwrap(), 1);
        assert_eq!(parse_size_to_bytes("1KB").unwrap(), 1000);
        assert_eq!(parse_size_to_bytes("1KiB").unwrap(), 1024);
        assert_eq!(parse_size_to_bytes("1MB").unwrap(), 1_000_000);
        assert_eq!(parse_size_to_bytes("1MiB").unwrap(), 1_048_576);
        assert_eq!(parse_size_to_bytes("1GB").unwrap(), 1_000_000_000);
        assert_eq!(parse_size_to_bytes("1GiB").unwrap(), 1_073_741_824);
        assert_eq!(parse_size_to_bytes("1TB").unwrap(), 1_000_000_000_000);
        assert_eq!(parse_size_to_bytes("1TiB").unwrap(), 1_099_511_627_776);
    }

    #[test]
    fn test_parse_size_with_decimals() {
        let size = parse_size_to_bytes("3.247GiB").unwrap();
        assert!(size > 3_000_000_000);
        assert!(size < 4_000_000_000);
    }
}


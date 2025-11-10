use crate::error::{AppError, Result};
use regex::Regex;
use std::process::Command;
use tracing::debug;

/// Docker Logs 采集器
/// 用于从 Docker 容器日志中提取配置和优化特性信息
pub struct DockerLogsCollector {
    container_name: String,
    // 缓存编译后的正则表达式
    block_stm_regex: Regex,
    memiavl_regex: Regex,
    zero_copy_regex: Regex,
}

/// 优化特性指标
#[derive(Debug, Clone)]
pub struct OptimizationMetrics {
    pub container_name: String,
    pub block_stm_enabled: bool,
    pub block_stm_workers: Option<i32>,
    pub memiavl_enabled: bool,
    pub memiavl_cache_size: Option<i64>,
    pub memiavl_snapshot_interval: Option<i32>,
    pub zero_copy_enabled: bool,
}

impl DockerLogsCollector {
    /// 创建新的 Docker Logs 采集器
    pub fn new(container_name: String) -> Self {
        Self {
            container_name,
            // 预编译正则表达式以提高性能
            block_stm_regex: Regex::new(r"(?i)block[-_]?stm[:\s]+(\w+)").unwrap(),
            memiavl_regex: Regex::new(r"(?i)memiavl[:\s]+(\w+)").unwrap(),
            zero_copy_regex: Regex::new(r"(?i)zero[-_]?copy[:\s]+(\w+)").unwrap(),
        }
    }

    /// 采集优化特性指标
    pub async fn collect(&self) -> Result<OptimizationMetrics> {
        debug!("Collecting Docker logs from container: {}", self.container_name);

        // 获取最近的日志（最后 100 行）
        let logs = self.get_container_logs(100).await?;

        // 解析优化特性配置
        let block_stm_enabled = self.parse_block_stm_enabled(&logs);
        let block_stm_workers = self.parse_block_stm_workers(&logs);
        let memiavl_enabled = self.parse_memiavl_enabled(&logs);
        let memiavl_cache_size = self.parse_memiavl_cache_size(&logs);
        let memiavl_snapshot_interval = self.parse_memiavl_snapshot_interval(&logs);
        let zero_copy_enabled = self.parse_zero_copy_enabled(&logs);

        Ok(OptimizationMetrics {
            container_name: self.container_name.clone(),
            block_stm_enabled,
            block_stm_workers,
            memiavl_enabled,
            memiavl_cache_size,
            memiavl_snapshot_interval,
            zero_copy_enabled,
        })
    }

    /// 获取容器日志
    async fn get_container_logs(&self, tail: usize) -> Result<String> {
        let output = Command::new("docker")
            .args(&["logs", "--tail", &tail.to_string(), &self.container_name])
            .output()
            .map_err(|e| {
                AppError::generic(format!("Failed to execute docker logs command: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::generic(format!("Docker logs command failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        debug!("Retrieved {} bytes of logs from {}", stdout.len(), self.container_name);
        Ok(stdout)
    }

    /// 解析 Block-STM 是否启用
    fn parse_block_stm_enabled(&self, logs: &str) -> bool {
        // 查找日志中的 Block-STM 配置
        // 示例日志格式:
        // - "block-stm: enabled"
        // - "Block-STM enabled: true"
        // - "Using Block-STM execution"

        if let Some(captures) = self.block_stm_regex.captures(logs) {
            if let Some(value) = captures.get(1) {
                let val = value.as_str().to_lowercase();
                return val == "enabled" || val == "true" || val == "on" || val == "yes";
            }
        }

        // 检查是否有明确的启用消息
        logs.to_lowercase().contains("block-stm")
            && (logs.to_lowercase().contains("enabled") || logs.to_lowercase().contains("using"))
    }

    /// 解析 Block-STM 工作线程数
    fn parse_block_stm_workers(&self, logs: &str) -> Option<i32> {
        // 查找工作线程数配置
        // 示例: "block-stm.workers: 4" 或 "Block-STM workers: 8"
        let workers_regex = Regex::new(r"(?i)block[-_]?stm[.\s]+workers?[:\s]+(\d+)").ok()?;

        if let Some(captures) = workers_regex.captures(logs) {
            if let Some(value) = captures.get(1) {
                return value.as_str().parse::<i32>().ok();
            }
        }

        None
    }

    /// 解析 MemIAVL 是否启用
    fn parse_memiavl_enabled(&self, logs: &str) -> bool {
        if let Some(captures) = self.memiavl_regex.captures(logs) {
            if let Some(value) = captures.get(1) {
                let val = value.as_str().to_lowercase();
                return val == "enabled" || val == "true" || val == "on" || val == "yes";
            }
        }

        logs.to_lowercase().contains("memiavl")
            && (logs.to_lowercase().contains("enabled") || logs.to_lowercase().contains("using"))
    }

    /// 解析 MemIAVL 缓存大小
    fn parse_memiavl_cache_size(&self, logs: &str) -> Option<i64> {
        // 查找缓存大小配置
        // 示例: "memiavl.cache-size: 1000000" 或 "MemIAVL cache size: 1000000 nodes"
        let cache_regex = Regex::new(r"(?i)memiavl[.\s]+cache[-_]?size[:\s]+(\d+)").ok()?;

        if let Some(captures) = cache_regex.captures(logs) {
            if let Some(value) = captures.get(1) {
                return value.as_str().parse::<i64>().ok();
            }
        }

        None
    }

    /// 解析 MemIAVL 快照间隔
    fn parse_memiavl_snapshot_interval(&self, logs: &str) -> Option<i32> {
        // 查找快照间隔配置
        // 示例: "memiavl.snapshot-interval: 10000" 或 "Snapshot interval: 10000 blocks"
        let interval_regex = Regex::new(r"(?i)snapshot[-_]?interval[:\s]+(\d+)").ok()?;

        if let Some(captures) = interval_regex.captures(logs) {
            if let Some(value) = captures.get(1) {
                return value.as_str().parse::<i32>().ok();
            }
        }

        None
    }

    /// 解析 Zero-Copy 是否启用
    fn parse_zero_copy_enabled(&self, logs: &str) -> bool {
        if let Some(captures) = self.zero_copy_regex.captures(logs) {
            if let Some(value) = captures.get(1) {
                let val = value.as_str().to_lowercase();
                return val == "enabled" || val == "true" || val == "on" || val == "yes";
            }
        }

        logs.to_lowercase().contains("zero-copy")
            && (logs.to_lowercase().contains("enabled") || logs.to_lowercase().contains("using"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_block_stm_enabled() {
        let collector = DockerLogsCollector::new("test-container".to_string());

        // 测试不同的日志格式
        assert!(collector.parse_block_stm_enabled("block-stm: enabled"));
        assert!(collector.parse_block_stm_enabled("Block-STM enabled: true"));
        assert!(collector.parse_block_stm_enabled("Using Block-STM execution"));
        assert!(!collector.parse_block_stm_enabled("block-stm: disabled"));
        assert!(!collector.parse_block_stm_enabled("No optimization enabled"));
    }

    #[test]
    fn test_parse_block_stm_workers() {
        let collector = DockerLogsCollector::new("test-container".to_string());

        assert_eq!(collector.parse_block_stm_workers("block-stm.workers: 4"), Some(4));
        assert_eq!(collector.parse_block_stm_workers("Block-STM workers: 8"), Some(8));
        assert_eq!(collector.parse_block_stm_workers("no workers config"), None);
    }

    #[test]
    fn test_parse_memiavl_enabled() {
        let collector = DockerLogsCollector::new("test-container".to_string());

        assert!(collector.parse_memiavl_enabled("memiavl: enabled"));
        assert!(collector.parse_memiavl_enabled("MemIAVL enabled: true"));
        assert!(collector.parse_memiavl_enabled("Using MemIAVL storage"));
        assert!(!collector.parse_memiavl_enabled("memiavl: disabled"));
    }

    #[test]
    fn test_parse_memiavl_cache_size() {
        let collector = DockerLogsCollector::new("test-container".to_string());

        assert_eq!(
            collector.parse_memiavl_cache_size("memiavl.cache-size: 1000000"),
            Some(1000000)
        );
        assert_eq!(
            collector.parse_memiavl_cache_size("MemIAVL cache size: 500000 nodes"),
            Some(500000)
        );
        assert_eq!(collector.parse_memiavl_cache_size("no cache config"), None);
    }

    #[test]
    fn test_parse_zero_copy_enabled() {
        let collector = DockerLogsCollector::new("test-container".to_string());

        assert!(collector.parse_zero_copy_enabled("zero-copy: enabled"));
        assert!(collector.parse_zero_copy_enabled("Zero-Copy enabled: true"));
        assert!(!collector.parse_zero_copy_enabled("zero-copy: disabled"));
    }
}

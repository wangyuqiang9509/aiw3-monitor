use std::collections::VecDeque;
use std::time::{Duration, SystemTime};
use tracing::debug;

/// TPS 计算器
/// 使用滑动窗口算法计算实时 TPS
pub struct TpsCalculator {
    /// 区块时间戳和交易数量的历史记录
    block_history: VecDeque<BlockRecord>,
    /// 时间窗口大小（秒）
    window_size: u64,
    /// 最大历史记录数量
    max_history: usize,
}

/// 区块记录
#[derive(Debug, Clone)]
struct BlockRecord {
    /// 区块高度
    height: u64,
    /// 区块时间戳
    timestamp: SystemTime,
    /// 交易数量
    tx_count: u64,
}

/// TPS 统计结果
#[derive(Debug, Clone)]
pub struct TpsMetrics {
    /// 当前 TPS（每秒交易数）
    pub current_tps: f64,
    /// 平均 TPS
    pub average_tps: f64,
    /// 峰值 TPS
    pub peak_tps: f64,
    /// 最近 N 秒的总交易数
    pub total_transactions: u64,
    /// 时间窗口大小（秒）
    pub window_size: u64,
}

impl TpsCalculator {
    /// 创建新的 TPS 计算器
    /// 
    /// # 参数
    /// * `window_size` - 时间窗口大小（秒），默认 60 秒
    pub fn new(window_size: u64) -> Self {
        Self {
            block_history: VecDeque::new(),
            window_size,
            max_history: 1000, // 最多保留 1000 个区块记录
        }
    }

    /// 添加新的区块记录
    /// 
    /// # 参数
    /// * `height` - 区块高度
    /// * `tx_count` - 交易数量
    pub fn add_block(&mut self, height: u64, tx_count: u64) {
        let record = BlockRecord {
            height,
            timestamp: SystemTime::now(),
            tx_count,
        };

        self.block_history.push_back(record);

        // 清理过期记录
        self.cleanup_old_records();

        // 限制历史记录数量
        while self.block_history.len() > self.max_history {
            self.block_history.pop_front();
        }

        debug!(
            "Added block {} with {} transactions, history size: {}",
            height,
            tx_count,
            self.block_history.len()
        );
    }

    /// 计算当前 TPS
    pub fn calculate_tps(&self) -> TpsMetrics {
        if self.block_history.is_empty() {
            return TpsMetrics {
                current_tps: 0.0,
                average_tps: 0.0,
                peak_tps: 0.0,
                total_transactions: 0,
                window_size: self.window_size,
            };
        }

        let now = SystemTime::now();
        let window_start = now - Duration::from_secs(self.window_size);

        // 统计时间窗口内的交易
        let mut total_tx = 0u64;
        let mut block_count = 0usize;
        let mut max_block_tx = 0u64;

        for record in self.block_history.iter().rev() {
            if record.timestamp >= window_start {
                total_tx += record.tx_count;
                block_count += 1;
                max_block_tx = max_block_tx.max(record.tx_count);
            } else {
                break;
            }
        }

        // 计算实际时间跨度
        let actual_duration = if block_count > 0 {
            let oldest_in_window = self
                .block_history
                .iter()
                .rev()
                .take(block_count)
                .last()
                .unwrap();
            
            now.duration_since(oldest_in_window.timestamp)
                .unwrap_or(Duration::from_secs(1))
                .as_secs_f64()
        } else {
            1.0
        };

        // 计算 TPS
        let current_tps = if actual_duration > 0.0 {
            total_tx as f64 / actual_duration
        } else {
            0.0
        };

        // 计算平均每区块交易数
        let avg_tx_per_block = if block_count > 0 {
            total_tx as f64 / block_count as f64
        } else {
            0.0
        };

        // 估算平均区块时间
        let avg_block_time = if block_count > 1 {
            actual_duration / (block_count - 1) as f64
        } else {
            6.0 // 默认 6 秒
        };

        // 计算平均 TPS
        let average_tps = if avg_block_time > 0.0 {
            avg_tx_per_block / avg_block_time
        } else {
            0.0
        };

        // 计算峰值 TPS（基于单个区块的最大交易数）
        let peak_tps = if avg_block_time > 0.0 {
            max_block_tx as f64 / avg_block_time
        } else {
            0.0
        };

        TpsMetrics {
            current_tps,
            average_tps,
            peak_tps,
            total_transactions: total_tx,
            window_size: self.window_size,
        }
    }

    /// 清理过期的记录
    fn cleanup_old_records(&mut self) {
        let cutoff = SystemTime::now() - Duration::from_secs(self.window_size * 2);

        while let Some(record) = self.block_history.front() {
            if record.timestamp < cutoff {
                self.block_history.pop_front();
            } else {
                break;
            }
        }
    }

    /// 获取历史记录数量
    pub fn history_size(&self) -> usize {
        self.block_history.len()
    }

    /// 清空所有历史记录
    pub fn clear(&mut self) {
        self.block_history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_tps_calculator_creation() {
        let calculator = TpsCalculator::new(60);
        assert_eq!(calculator.window_size, 60);
        assert_eq!(calculator.history_size(), 0);
    }

    #[test]
    fn test_add_block() {
        let mut calculator = TpsCalculator::new(60);
        
        calculator.add_block(100, 50);
        assert_eq!(calculator.history_size(), 1);
        
        calculator.add_block(101, 60);
        assert_eq!(calculator.history_size(), 2);
    }

    #[test]
    fn test_calculate_tps_empty() {
        let calculator = TpsCalculator::new(60);
        let metrics = calculator.calculate_tps();
        
        assert_eq!(metrics.current_tps, 0.0);
        assert_eq!(metrics.total_transactions, 0);
    }

    #[test]
    fn test_calculate_tps_single_block() {
        let mut calculator = TpsCalculator::new(60);
        calculator.add_block(100, 100);
        
        let metrics = calculator.calculate_tps();
        assert!(metrics.current_tps > 0.0);
        assert_eq!(metrics.total_transactions, 100);
    }

    #[test]
    fn test_calculate_tps_multiple_blocks() {
        let mut calculator = TpsCalculator::new(60);
        
        // 添加多个区块
        for i in 0..10 {
            calculator.add_block(100 + i, 50);
            thread::sleep(Duration::from_millis(100));
        }
        
        let metrics = calculator.calculate_tps();
        assert!(metrics.current_tps > 0.0);
        assert_eq!(metrics.total_transactions, 500);
        assert!(metrics.average_tps > 0.0);
    }

    #[test]
    fn test_cleanup_old_records() {
        let mut calculator = TpsCalculator::new(1); // 1 秒窗口
        
        calculator.add_block(100, 50);
        thread::sleep(Duration::from_secs(2));
        calculator.add_block(101, 60);
        
        // 触发清理
        calculator.cleanup_old_records();
        
        // 应该只保留最新的记录
        assert!(calculator.history_size() <= 2);
    }

    #[test]
    fn test_clear() {
        let mut calculator = TpsCalculator::new(60);
        
        calculator.add_block(100, 50);
        calculator.add_block(101, 60);
        assert_eq!(calculator.history_size(), 2);
        
        calculator.clear();
        assert_eq!(calculator.history_size(), 0);
    }

    #[test]
    fn test_max_history_limit() {
        let mut calculator = TpsCalculator::new(60);
        calculator.max_history = 10; // 设置较小的限制用于测试
        
        // 添加超过限制的区块
        for i in 0..20 {
            calculator.add_block(100 + i, 50);
        }
        
        // 应该不超过最大限制
        assert!(calculator.history_size() <= 10);
    }
}


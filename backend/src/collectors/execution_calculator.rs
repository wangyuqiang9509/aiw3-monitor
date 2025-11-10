// 执行性能计算器
// 基于实际 TPS 和区块时间计算并行执行效率

use tracing::debug;

/// 执行性能指标
#[derive(Debug, Clone)]
pub struct ExecutionPerformance {
    /// 实际 TPS
    pub actual_tps: f64,
    /// 基准 TPS（单线程）
    pub baseline_tps: f64,
    /// 加速比（实际 TPS / 基准 TPS）
    pub speedup: f64,
    /// 估算的并行执行率（百分比）
    pub estimated_parallelism_rate: f64,
    /// Block-STM 工作线程数
    pub block_stm_workers: u32,
}

/// 执行性能计算器
pub struct ExecutionCalculator;

impl ExecutionCalculator {
    /// 基准 TPS（单线程 Cosmos SDK 的理论 TPS）
    const BASELINE_TPS: f64 = 3000.0;

    /// 计算执行性能指标
    ///
    /// # 参数
    /// * `current_tps` - 当前实际 TPS
    /// * `block_time` - 平均区块时间（秒）
    /// * `workers` - Block-STM 工作线程数
    ///
    /// # 返回
    /// 执行性能指标
    pub fn calculate(current_tps: f64, block_time: f64, workers: u32) -> ExecutionPerformance {
        // 如果 TPS 或区块时间无效，返回默认值
        if current_tps <= 0.0 || block_time <= 0.0 || workers == 0 {
            debug!(
                "Invalid input for execution performance calculation: tps={}, block_time={}, workers={}",
                current_tps, block_time, workers
            );
            return ExecutionPerformance {
                actual_tps: current_tps,
                baseline_tps: Self::BASELINE_TPS,
                speedup: 0.0,
                estimated_parallelism_rate: 0.0,
                block_stm_workers: workers,
            };
        }

        // 计算加速比 = 实际 TPS / 基准 TPS
        let speedup = current_tps / Self::BASELINE_TPS;

        // 推算并行执行率
        // 理论上，如果有 N 个 workers，完美并行执行可以达到 N 倍加速
        // 实际并行执行率 = (实际加速比 / 理论最大加速比) * 100%
        let theoretical_max_speedup = workers as f64;
        let parallelism_rate = (speedup / theoretical_max_speedup).min(1.0) * 100.0;

        debug!(
            "Execution performance: TPS={:.2}, speedup={:.2}x, parallelism={:.1}% (workers={})",
            current_tps, speedup, parallelism_rate, workers
        );

        ExecutionPerformance {
            actual_tps: current_tps,
            baseline_tps: Self::BASELINE_TPS,
            speedup,
            estimated_parallelism_rate: parallelism_rate,
            block_stm_workers: workers,
        }
    }

    /// 基于区块数据计算执行性能
    ///
    /// # 参数
    /// * `tx_count` - 区块中的交易数量
    /// * `block_time` - 区块时间（秒）
    /// * `workers` - Block-STM 工作线程数
    ///
    /// # 返回
    /// 执行性能指标
    pub fn calculate_from_block(
        tx_count: u64,
        block_time: f64,
        workers: u32,
    ) -> ExecutionPerformance {
        // 计算实际 TPS = 交易数 / 区块时间
        let actual_tps = if block_time > 0.0 { tx_count as f64 / block_time } else { 0.0 };

        Self::calculate(actual_tps, block_time, workers)
    }

    /// 估算理论最大 TPS
    ///
    /// # 参数
    /// * `workers` - Block-STM 工作线程数
    ///
    /// # 返回
    /// 理论最大 TPS
    pub fn theoretical_max_tps(workers: u32) -> f64 {
        Self::BASELINE_TPS * workers as f64
    }

    /// 计算并行效率
    ///
    /// 并行效率 = 加速比 / 处理器数量
    ///
    /// # 参数
    /// * `speedup` - 加速比
    /// * `workers` - 工作线程数
    ///
    /// # 返回
    /// 并行效率（0.0 - 1.0）
    pub fn parallel_efficiency(speedup: f64, workers: u32) -> f64 {
        if workers == 0 {
            return 0.0;
        }
        (speedup / workers as f64).min(1.0)
    }
}

impl ExecutionPerformance {
    /// 获取并行效率（0.0 - 1.0）
    pub fn parallel_efficiency(&self) -> f64 {
        ExecutionCalculator::parallel_efficiency(self.speedup, self.block_stm_workers)
    }

    /// 获取理论最大 TPS
    pub fn theoretical_max_tps(&self) -> f64 {
        ExecutionCalculator::theoretical_max_tps(self.block_stm_workers)
    }

    /// 检查性能是否良好
    ///
    /// 如果并行执行率 > 70%，认为性能良好
    pub fn is_performing_well(&self) -> bool {
        self.estimated_parallelism_rate > 70.0
    }

    /// 获取性能评级
    pub fn performance_rating(&self) -> &'static str {
        if self.estimated_parallelism_rate >= 90.0 {
            "Excellent"
        } else if self.estimated_parallelism_rate >= 70.0 {
            "Good"
        } else if self.estimated_parallelism_rate >= 50.0 {
            "Fair"
        } else if self.estimated_parallelism_rate >= 30.0 {
            "Poor"
        } else {
            "Critical"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_basic() {
        let performance = ExecutionCalculator::calculate(24000.0, 1.5, 8);

        assert_eq!(performance.actual_tps, 24000.0);
        assert_eq!(performance.baseline_tps, 3000.0);
        assert_eq!(performance.speedup, 8.0);
        assert_eq!(performance.block_stm_workers, 8);
        // 8x 加速 / 8 workers = 100% 并行执行率
        assert_eq!(performance.estimated_parallelism_rate, 100.0);
    }

    #[test]
    fn test_calculate_from_block() {
        // 1000 个交易，2 秒区块时间 = 500 TPS
        let performance = ExecutionCalculator::calculate_from_block(1000, 2.0, 8);

        assert_eq!(performance.actual_tps, 500.0);
        assert_eq!(performance.baseline_tps, 3000.0);
        // 500 / 3000 = 0.167x 加速
        assert!((performance.speedup - 0.167).abs() < 0.001);
    }

    #[test]
    fn test_theoretical_max_tps() {
        assert_eq!(ExecutionCalculator::theoretical_max_tps(8), 24000.0);
        assert_eq!(ExecutionCalculator::theoretical_max_tps(4), 12000.0);
        assert_eq!(ExecutionCalculator::theoretical_max_tps(1), 3000.0);
    }

    #[test]
    fn test_parallel_efficiency() {
        // 完美并行：8x 加速 / 8 workers = 1.0 效率
        assert_eq!(ExecutionCalculator::parallel_efficiency(8.0, 8), 1.0);

        // 50% 并行：4x 加速 / 8 workers = 0.5 效率
        assert_eq!(ExecutionCalculator::parallel_efficiency(4.0, 8), 0.5);

        // 超线性加速（理论上不可能，但计算会限制在 1.0）
        assert_eq!(ExecutionCalculator::parallel_efficiency(10.0, 8), 1.0);
    }

    #[test]
    fn test_performance_rating() {
        let excellent = ExecutionCalculator::calculate(27000.0, 1.0, 8);
        assert_eq!(excellent.performance_rating(), "Excellent");
        assert!(excellent.is_performing_well());

        let good = ExecutionCalculator::calculate(21000.0, 1.0, 8);
        assert_eq!(good.performance_rating(), "Good");
        assert!(good.is_performing_well());

        let fair = ExecutionCalculator::calculate(15000.0, 1.0, 8);
        assert_eq!(fair.performance_rating(), "Fair");
        assert!(!fair.is_performing_well());

        let poor = ExecutionCalculator::calculate(10000.0, 1.0, 8);
        assert_eq!(poor.performance_rating(), "Poor");
        assert!(!poor.is_performing_well());

        let critical = ExecutionCalculator::calculate(3000.0, 1.0, 8);
        assert_eq!(critical.performance_rating(), "Critical");
        assert!(!critical.is_performing_well());
    }

    #[test]
    fn test_invalid_inputs() {
        // 无效的 TPS
        let perf1 = ExecutionCalculator::calculate(0.0, 1.0, 8);
        assert_eq!(perf1.speedup, 0.0);
        assert_eq!(perf1.estimated_parallelism_rate, 0.0);

        // 无效的区块时间
        let perf2 = ExecutionCalculator::calculate(1000.0, 0.0, 8);
        assert_eq!(perf2.speedup, 0.0);

        // 无效的 workers
        let perf3 = ExecutionCalculator::calculate(1000.0, 1.0, 0);
        assert_eq!(perf3.speedup, 0.0);
    }

    #[test]
    fn test_real_world_scenario() {
        // 模拟真实场景：224,719 TPS，1.78 秒区块时间，8 workers
        let performance = ExecutionCalculator::calculate(224719.0, 1.78, 8);

        assert_eq!(performance.actual_tps, 224719.0);
        assert_eq!(performance.baseline_tps, 3000.0);
        // 224719 / 3000 = 74.906x 加速
        assert!((performance.speedup - 74.906).abs() < 0.01);
        // 74.906 / 8 * 100 = 936.325%，但被限制为 100%
        assert_eq!(performance.estimated_parallelism_rate, 100.0);
        assert_eq!(performance.performance_rating(), "Excellent");
        assert!(performance.is_performing_well());
    }
}

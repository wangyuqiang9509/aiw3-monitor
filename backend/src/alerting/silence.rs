// 告警静默逻辑
// 用于防止在静默期内重复发送告警
// 使用数据库记录最近触发时间，避免在静默期内重复通知

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, warn};

use crate::error::Result;

/// 告警静默管理器
/// 
/// 负责管理告警的静默期，防止在短时间内重复发送相同的告警通知
pub struct AlertSilence {
    pool: Arc<PgPool>,
}

impl AlertSilence {
    /// 创建新的告警静默管理器
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// 检查告警规则是否在静默期内
    /// 
    /// # 参数
    /// - `rule_id`: 告警规则 ID
    /// - `silence_period_seconds`: 静默期时长（秒）
    /// 
    /// # 返回
    /// - `Ok(true)`: 在静默期内，不应发送告警
    /// - `Ok(false)`: 不在静默期内，可以发送告警
    pub async fn is_silenced(&self, rule_id: i32, silence_period_seconds: i32) -> Result<bool> {
        // 查询该规则最近的活跃告警事件
        let recent_alert: Option<(DateTime<Utc>,)> = sqlx::query_as(
            r#"
            SELECT triggered_at 
            FROM alert_events 
            WHERE rule_id = $1 
              AND status = 'active' 
              AND notification_sent = true
            ORDER BY triggered_at DESC 
            LIMIT 1
            "#,
        )
        .bind(rule_id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        if let Some((last_triggered_at,)) = recent_alert {
            let now = Utc::now();
            let elapsed_seconds = (now - last_triggered_at).num_seconds();
            
            if elapsed_seconds < silence_period_seconds as i64 {
                debug!(
                    rule_id = rule_id,
                    elapsed_seconds = elapsed_seconds,
                    silence_period = silence_period_seconds,
                    "Alert is within silence period"
                );
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// 记录告警已发送，开始静默期
    /// 
    /// # 参数
    /// - `event_id`: 告警事件 ID
    pub async fn mark_alert_sent(&self, event_id: i64) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE alert_events 
            SET notification_sent = true, updated_at = NOW() 
            WHERE id = $1
            "#,
        )
        .bind(event_id)
        .execute(self.pool.as_ref())
        .await?;

        debug!(event_id = event_id, "Marked alert as sent");
        Ok(())
    }

    /// 清理过期的静默记录（可选的维护操作）
    /// 
    /// 将已恢复的告警事件标记为已处理，避免数据库中积累过多活跃告警
    pub async fn cleanup_resolved_alerts(&self) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE alert_events 
            SET status = 'resolved', resolved_at = NOW(), updated_at = NOW() 
            WHERE status = 'active' 
              AND triggered_at < NOW() - INTERVAL '24 hours'
            "#,
        )
        .execute(self.pool.as_ref())
        .await?;

        let rows_affected = result.rows_affected();
        if rows_affected > 0 {
            warn!(
                rows_affected = rows_affected,
                "Auto-resolved stale active alerts older than 24 hours"
            );
        }

        Ok(rows_affected)
    }

    /// 获取规则的活跃告警数量
    /// 
    /// # 参数
    /// - `rule_id`: 告警规则 ID
    /// 
    /// # 返回
    /// 活跃告警的数量
    pub async fn get_active_alert_count(&self, rule_id: i32) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) 
            FROM alert_events 
            WHERE rule_id = $1 AND status = 'active'
            "#,
        )
        .bind(rule_id)
        .fetch_one(self.pool.as_ref())
        .await?;

        Ok(count)
    }

    /// 恢复告警（当条件不再满足时）
    /// 
    /// # 参数
    /// - `rule_id`: 告警规则 ID
    pub async fn resolve_alerts(&self, rule_id: i32) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE alert_events 
            SET status = 'resolved', resolved_at = NOW(), updated_at = NOW() 
            WHERE rule_id = $1 AND status = 'active'
            "#,
        )
        .bind(rule_id)
        .execute(self.pool.as_ref())
        .await?;

        let rows_affected = result.rows_affected();
        if rows_affected > 0 {
            debug!(
                rule_id = rule_id,
                rows_affected = rows_affected,
                "Resolved active alerts for rule"
            );
        }

        Ok(rows_affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试需要实际的数据库连接
    // 在 CI 环境中，应使用 testcontainers 或类似工具提供测试数据库

    #[tokio::test]
    #[ignore] // 需要数据库连接，使用 `cargo test -- --ignored` 运行
    async fn test_silence_logic() {
        // 此测试需要实际的数据库环境
        // 在集成测试中进行完整测试
    }
}


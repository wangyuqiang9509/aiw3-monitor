use crate::error::{AppError, Result};
use crate::models::alert_event::AlertEvent;
use crate::models::alert_rule::AlertRule;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::{debug, info};

/// 告警存储
pub struct AlertStore {
    pool: PgPool,
}

impl AlertStore {
    /// 创建新的告警存储实例
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ==================== 告警规则管理 ====================

    /// 创建告警规则
    pub async fn create_rule(&self, rule: &AlertRule) -> Result<i32> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO alert_rules (
                name, node_id, metric_name, condition_type, threshold_value,
                window_seconds, comparison_operator, severity, email_recipients,
                silence_period_seconds, enabled, description
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id
            "#,
            rule.name,
            rule.node_id,
            rule.metric_name,
            rule.condition_type,
            rule.threshold_value,
            rule.window_seconds,
            rule.comparison_operator,
            rule.severity,
            &rule.email_recipients,
            rule.silence_period_seconds,
            rule.enabled,
            rule.description
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::Database)?;

        info!("Created alert rule: {} (id: {})", rule.name, id);
        Ok(id)
    }

    /// 获取所有启用的告警规则
    pub async fn get_enabled_rules(&self) -> Result<Vec<AlertRule>> {
        let rules = sqlx::query_as!(
            AlertRule,
            r#"
            SELECT id, name, node_id, metric_name, condition_type, threshold_value,
                   window_seconds, comparison_operator, severity, email_recipients,
                   silence_period_seconds, enabled, description, created_at, updated_at
            FROM alert_rules
            WHERE enabled = true
            ORDER BY severity DESC, created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)?;

        debug!("Retrieved {} enabled alert rules", rules.len());
        Ok(rules)
    }

    /// 根据 ID 获取告警规则
    pub async fn get_rule_by_id(&self, id: i32) -> Result<Option<AlertRule>> {
        let rule = sqlx::query_as!(
            AlertRule,
            r#"
            SELECT id, name, node_id, metric_name, condition_type, threshold_value,
                   window_seconds, comparison_operator, severity, email_recipients,
                   silence_period_seconds, enabled, description, created_at, updated_at
            FROM alert_rules
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(rule)
    }

    /// 更新告警规则
    pub async fn update_rule(&self, rule: &AlertRule) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE alert_rules
            SET name = $1, node_id = $2, metric_name = $3, condition_type = $4,
                threshold_value = $5, window_seconds = $6, comparison_operator = $7,
                severity = $8, email_recipients = $9, silence_period_seconds = $10,
                enabled = $11, description = $12, updated_at = NOW()
            WHERE id = $13
            "#,
            rule.name,
            rule.node_id,
            rule.metric_name,
            rule.condition_type,
            rule.threshold_value,
            rule.window_seconds,
            rule.comparison_operator,
            rule.severity,
            &rule.email_recipients,
            rule.silence_period_seconds,
            rule.enabled,
            rule.description,
            rule.id
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        info!("Updated alert rule: {} (id: {})", rule.name, rule.id);
        Ok(())
    }

    /// 删除告警规则
    pub async fn delete_rule(&self, id: i32) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM alert_rules
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        info!("Deleted alert rule: id={}", id);
        Ok(())
    }

    // ==================== 告警事件管理 ====================

    /// 创建告警事件
    pub async fn create_event(&self, event: &AlertEvent) -> Result<i64> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO alert_events (
                rule_id, node_id, triggered_at, trigger_value, status,
                notification_sent, notification_error, user_notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
            event.rule_id,
            event.node_id,
            event.triggered_at,
            event.trigger_value,
            event.status,
            event.notification_sent,
            event.notification_error,
            event.user_notes
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::Database)?;

        info!(
            "Created alert event: rule_id={}, node_id={}, id={}",
            event.rule_id, event.node_id, id
        );
        Ok(id)
    }

    /// 更新告警事件
    pub async fn update_event(&self, event: &AlertEvent) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE alert_events
            SET resolved_at = $1, status = $2, notification_sent = $3,
                notification_error = $4, user_notes = $5, updated_at = NOW()
            WHERE id = $6
            "#,
            event.resolved_at,
            event.status,
            event.notification_sent,
            event.notification_error,
            event.user_notes,
            event.id
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        debug!("Updated alert event: id={}", event.id);
        Ok(())
    }

    /// 获取活跃的告警事件
    pub async fn get_active_events(&self) -> Result<Vec<AlertEvent>> {
        let events = sqlx::query_as!(
            AlertEvent,
            r#"
            SELECT id, rule_id, node_id, triggered_at, resolved_at, trigger_value,
                   status, notification_sent, notification_error, user_notes,
                   created_at, updated_at
            FROM alert_events
            WHERE status = 'active'
            ORDER BY triggered_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)?;

        debug!("Retrieved {} active alert events", events.len());
        Ok(events)
    }

    /// 获取指定规则的活跃告警事件
    pub async fn get_active_events_by_rule(&self, rule_id: i32) -> Result<Vec<AlertEvent>> {
        let events = sqlx::query_as!(
            AlertEvent,
            r#"
            SELECT id, rule_id, node_id, triggered_at, resolved_at, trigger_value,
                   status, notification_sent, notification_error, user_notes,
                   created_at, updated_at
            FROM alert_events
            WHERE rule_id = $1 AND status = 'active'
            ORDER BY triggered_at DESC
            "#,
            rule_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(events)
    }

    /// 获取历史告警事件（分页）
    pub async fn get_events_history(
        &self,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AlertEvent>> {
        let start = start_time.unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
        let end = end_time.unwrap_or_else(Utc::now);

        let events = sqlx::query_as!(
            AlertEvent,
            r#"
            SELECT id, rule_id, node_id, triggered_at, resolved_at, trigger_value,
                   status, notification_sent, notification_error, user_notes,
                   created_at, updated_at
            FROM alert_events
            WHERE triggered_at >= $1 AND triggered_at <= $2
            ORDER BY triggered_at DESC
            LIMIT $3 OFFSET $4
            "#,
            start,
            end,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)?;

        debug!(
            "Retrieved {} alert events from history (limit: {}, offset: {})",
            events.len(),
            limit,
            offset
        );
        Ok(events)
    }

    /// 获取告警统计信息
    pub async fn get_alert_statistics(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<AlertStatistics> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_count,
                COUNT(*) FILTER (WHERE status = 'active') as active_count,
                COUNT(*) FILTER (WHERE status = 'resolved') as resolved_count,
                COUNT(*) FILTER (WHERE notification_sent = true) as notified_count,
                COUNT(*) FILTER (WHERE notification_sent = false) as failed_notification_count
            FROM alert_events
            WHERE triggered_at >= $1 AND triggered_at <= $2
            "#,
            start_time,
            end_time
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(AlertStatistics {
            total_count: stats.total_count.unwrap_or(0),
            active_count: stats.active_count.unwrap_or(0),
            resolved_count: stats.resolved_count.unwrap_or(0),
            notified_count: stats.notified_count.unwrap_or(0),
            failed_notification_count: stats.failed_notification_count.unwrap_or(0),
        })
    }

    /// 清理旧的告警事件
    pub async fn cleanup_old_events(&self, before: DateTime<Utc>) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM alert_events
            WHERE triggered_at < $1 AND status = 'resolved'
            "#,
            before
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        let deleted = result.rows_affected();
        info!("Cleaned up {} old alert events", deleted);
        Ok(deleted)
    }

    // ==================== 告警历史与分析 (Phase 5) ====================

    /// 获取告警历史（支持高级筛选）
    pub async fn query_alert_history(
        &self,
        filter: &AlertHistoryFilter,
    ) -> Result<Vec<AlertEventWithDetails>> {
        let start = filter.start_time.unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
        let end = filter.end_time.unwrap_or_else(Utc::now);

        let mut query = String::from(
            r#"
            SELECT 
                ae.id, ae.rule_id, ae.node_id, ae.triggered_at, ae.resolved_at,
                ae.trigger_value, ae.status, ae.notification_sent, ae.notification_error,
                ae.user_notes, ae.created_at, ae.updated_at,
                ar.name as rule_name, ar.severity, ar.metric_name,
                bn.name as node_name, bn.environment
            FROM alert_events ae
            JOIN alert_rules ar ON ae.rule_id = ar.id
            JOIN blockchain_nodes bn ON ae.node_id = bn.id
            WHERE ae.triggered_at >= $1 AND ae.triggered_at <= $2
            "#,
        );

        // 添加可选筛选条件
        let mut param_index = 3;
        if filter.node_id.is_some() {
            query.push_str(&format!(" AND ae.node_id = ${}", param_index));
            param_index += 1;
        }
        if filter.rule_id.is_some() {
            query.push_str(&format!(" AND ae.rule_id = ${}", param_index));
            param_index += 1;
        }
        if filter.status.is_some() {
            query.push_str(&format!(" AND ae.status = ${}", param_index));
            param_index += 1;
        }
        if filter.severity.is_some() {
            query.push_str(&format!(" AND ar.severity = ${}", param_index));
            param_index += 1;
        }

        query.push_str(" ORDER BY ae.triggered_at DESC");
        query.push_str(&format!(" LIMIT ${} OFFSET ${}", param_index, param_index + 1));

        // 构建查询（简化版本，使用原始 SQL）
        let events = sqlx::query_as::<_, AlertEventWithDetails>(&query)
            .bind(start)
            .bind(end)
            .bind(filter.limit.unwrap_or(100))
            .bind(filter.offset.unwrap_or(0))
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::Database)?;

        debug!(
            "Retrieved {} alert events with filters",
            events.len()
        );
        Ok(events)
    }

    /// 获取告警频率统计（按时间分组）
    pub async fn get_alert_frequency(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        interval: &str, // 'hour', 'day', 'week'
    ) -> Result<Vec<AlertFrequency>> {
        let query = format!(
            r#"
            SELECT 
                date_trunc($1, triggered_at) as time_bucket,
                COUNT(*) as count,
                COUNT(*) FILTER (WHERE severity = 'critical') as critical_count,
                COUNT(*) FILTER (WHERE severity = 'warning') as warning_count,
                COUNT(*) FILTER (WHERE severity = 'info') as info_count
            FROM alert_events ae
            JOIN alert_rules ar ON ae.rule_id = ar.id
            WHERE ae.triggered_at >= $2 AND ae.triggered_at <= $3
            GROUP BY time_bucket
            ORDER BY time_bucket DESC
            "#
        );

        let frequencies = sqlx::query_as::<_, AlertFrequency>(&query)
            .bind(interval)
            .bind(start_time)
            .bind(end_time)
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::Database)?;

        Ok(frequencies)
    }

    /// 获取告警类型分布
    pub async fn get_alert_type_distribution(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<AlertTypeDistribution>> {
        let distributions = sqlx::query_as!(
            AlertTypeDistribution,
            r#"
            SELECT 
                ar.metric_name,
                ar.severity,
                COUNT(*) as count,
                AVG(ae.trigger_value) as avg_trigger_value,
                MIN(ae.trigger_value) as min_trigger_value,
                MAX(ae.trigger_value) as max_trigger_value
            FROM alert_events ae
            JOIN alert_rules ar ON ae.rule_id = ar.id
            WHERE ae.triggered_at >= $1 AND ae.triggered_at <= $2
            GROUP BY ar.metric_name, ar.severity
            ORDER BY count DESC
            "#,
            start_time,
            end_time
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(distributions)
    }

    /// 获取告警响应时间统计（P95, P99）
    pub async fn get_alert_response_stats(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<AlertResponseStats> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                AVG(EXTRACT(EPOCH FROM (resolved_at - triggered_at))) as avg_response_time,
                percentile_cont(0.95) WITHIN GROUP (ORDER BY EXTRACT(EPOCH FROM (resolved_at - triggered_at))) as p95_response_time,
                percentile_cont(0.99) WITHIN GROUP (ORDER BY EXTRACT(EPOCH FROM (resolved_at - triggered_at))) as p99_response_time,
                MIN(EXTRACT(EPOCH FROM (resolved_at - triggered_at))) as min_response_time,
                MAX(EXTRACT(EPOCH FROM (resolved_at - triggered_at))) as max_response_time
            FROM alert_events
            WHERE triggered_at >= $1 AND triggered_at <= $2
              AND resolved_at IS NOT NULL
              AND status = 'resolved'
            "#,
            start_time,
            end_time
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(AlertResponseStats {
            avg_response_time_seconds: stats.avg_response_time.unwrap_or(0.0),
            p95_response_time_seconds: stats.p95_response_time.unwrap_or(0.0),
            p99_response_time_seconds: stats.p99_response_time.unwrap_or(0.0),
            min_response_time_seconds: stats.min_response_time.unwrap_or(0.0),
            max_response_time_seconds: stats.max_response_time.unwrap_or(0.0),
        })
    }

    /// 更新告警事件备注
    pub async fn update_event_notes(&self, event_id: i64, notes: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE alert_events
            SET user_notes = $1, updated_at = NOW()
            WHERE id = $2
            "#,
            notes,
            event_id
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        info!("Updated notes for alert event: id={}", event_id);
        Ok(())
    }

    /// 获取节点的告警统计
    pub async fn get_node_alert_stats(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<NodeAlertStats>> {
        let stats = sqlx::query_as!(
            NodeAlertStats,
            r#"
            SELECT 
                bn.id as node_id,
                bn.name as node_name,
                COUNT(*) as total_alerts,
                COUNT(*) FILTER (WHERE ae.status = 'active') as active_alerts,
                COUNT(*) FILTER (WHERE ar.severity = 'critical') as critical_alerts,
                COUNT(*) FILTER (WHERE ar.severity = 'warning') as warning_alerts
            FROM alert_events ae
            JOIN blockchain_nodes bn ON ae.node_id = bn.id
            JOIN alert_rules ar ON ae.rule_id = ar.id
            WHERE ae.triggered_at >= $1 AND ae.triggered_at <= $2
            GROUP BY bn.id, bn.name
            ORDER BY total_alerts DESC
            "#,
            start_time,
            end_time
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(stats)
    }
}

/// 告警统计信息
#[derive(Debug, Clone)]
pub struct AlertStatistics {
    pub total_count: i64,
    pub active_count: i64,
    pub resolved_count: i64,
    pub notified_count: i64,
    pub failed_notification_count: i64,
}

/// 告警历史筛选器
#[derive(Debug, Clone, Default)]
pub struct AlertHistoryFilter {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub node_id: Option<i32>,
    pub rule_id: Option<i32>,
    pub status: Option<String>,
    pub severity: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 告警事件详情（包含关联信息）
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AlertEventWithDetails {
    pub id: i64,
    pub rule_id: i32,
    pub node_id: i32,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub trigger_value: f64,
    pub status: String,
    pub notification_sent: bool,
    pub notification_error: Option<String>,
    pub user_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub rule_name: String,
    pub severity: String,
    pub metric_name: String,
    pub node_name: String,
    pub environment: String,
}

/// 告警频率统计
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AlertFrequency {
    pub time_bucket: DateTime<Utc>,
    pub count: i64,
    pub critical_count: i64,
    pub warning_count: i64,
    pub info_count: i64,
}

/// 告警类型分布
#[derive(Debug, Clone)]
pub struct AlertTypeDistribution {
    pub metric_name: String,
    pub severity: String,
    pub count: i64,
    pub avg_trigger_value: Option<f64>,
    pub min_trigger_value: Option<f64>,
    pub max_trigger_value: Option<f64>,
}

/// 告警响应时间统计
#[derive(Debug, Clone)]
pub struct AlertResponseStats {
    pub avg_response_time_seconds: f64,
    pub p95_response_time_seconds: f64,
    pub p99_response_time_seconds: f64,
    pub min_response_time_seconds: f64,
    pub max_response_time_seconds: f64,
}

/// 节点告警统计
#[derive(Debug, Clone)]
pub struct NodeAlertStats {
    pub node_id: i32,
    pub node_name: String,
    pub total_alerts: i64,
    pub active_alerts: i64,
    pub critical_alerts: i64,
    pub warning_alerts: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_store_creation() {
        // 这个测试需要数据库连接，标记为 ignore
        // 实际测试在集成测试中进行
    }

    #[test]
    fn test_alert_statistics() {
        let stats = AlertStatistics {
            total_count: 100,
            active_count: 10,
            resolved_count: 90,
            notified_count: 95,
            failed_notification_count: 5,
        };

        assert_eq!(stats.total_count, 100);
        assert_eq!(stats.active_count, 10);
        assert_eq!(stats.resolved_count, 90);
    }
}


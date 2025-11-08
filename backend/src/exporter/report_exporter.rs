// 告警报告导出器
// 支持 CSV 和 JSON 格式导出告警历史数据

use crate::error::Result;
use crate::storage::alert_store::{AlertEventWithDetails, AlertHistoryFilter, AlertStore};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::io::Write;
use tracing::info;

/// 报告导出器
pub struct ReportExporter {
    alert_store: AlertStore,
}

impl ReportExporter {
    /// 创建新的报告导出器
    pub fn new(alert_store: AlertStore) -> Self {
        Self { alert_store }
    }

    /// 导出告警报告为 CSV 格式
    ///
    /// # 参数
    /// - `filter`: 告警历史筛选器
    /// - `output`: 输出写入器
    pub async fn export_csv<W: Write>(
        &self,
        filter: &AlertHistoryFilter,
        mut output: W,
    ) -> Result<usize> {
        // 获取告警历史数据
        let events = self.alert_store.query_alert_history(filter).await?;

        // 写入 CSV 头部
        writeln!(
            output,
            "ID,Rule Name,Node Name,Environment,Metric,Severity,Status,Trigger Value,Triggered At,Resolved At,Duration (seconds),Notification Sent,User Notes"
        )?;

        // 写入数据行
        for event in &events {
            let duration = event.resolved_at.map(|resolved| {
                (resolved - event.triggered_at).num_seconds()
            });

            writeln!(
                output,
                "{},{},{},{},{},{},{},{},{},{},{},{},\"{}\"",
                event.id,
                escape_csv(&event.rule_name),
                escape_csv(&event.node_name),
                escape_csv(&event.environment),
                escape_csv(&event.metric_type),
                escape_csv(&event.severity),
                escape_csv(&event.status),
                event.metric_value.unwrap_or(0.0),
                event.triggered_at.to_rfc3339(),
                event.resolved_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
                duration.map(|d| d.to_string()).unwrap_or_default(),
                event.notification_sent,
                escape_csv(&event.notes.clone().unwrap_or_default())
            )?;
        }

        let count = events.len();
        info!("Exported {} alert events to CSV", count);
        Ok(count)
    }

    /// 导出告警报告为 JSON 格式
    ///
    /// # 参数
    /// - `filter`: 告警历史筛选器
    /// - `output`: 输出写入器
    pub async fn export_json<W: Write>(
        &self,
        filter: &AlertHistoryFilter,
        mut output: W,
    ) -> Result<usize> {
        // 获取告警历史数据
        let events = self.alert_store.query_alert_history(filter).await?;

        // 转换为可序列化的格式
        let export_events: Vec<AlertEventExport> = events
            .iter()
            .map(|e| AlertEventExport::from_details(e))
            .collect();

        // 创建导出报告
        let report = AlertReport {
            generated_at: Utc::now(),
            filter: FilterSummary::from_filter(filter),
            total_count: export_events.len(),
            events: export_events,
        };

        // 序列化为 JSON
        let json = serde_json::to_string_pretty(&report)?;
        write!(output, "{}", json)?;

        let count = report.total_count;
        info!("Exported {} alert events to JSON", count);
        Ok(count)
    }

    /// 导出告警统计报告为 JSON 格式
    pub async fn export_statistics_json<W: Write>(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        mut output: W,
    ) -> Result<()> {
        // 获取各种统计数据
        let basic_stats = self.alert_store.get_alert_statistics(start_time, end_time).await?;
        let frequency_stats = self.alert_store.get_alert_frequency(start_time, end_time, "day").await?;
        let type_distribution = self.alert_store.get_alert_type_distribution(start_time, end_time).await?;
        let response_stats = self.alert_store.get_alert_response_stats(start_time, end_time).await?;
        let node_stats = self.alert_store.get_node_alert_stats(start_time, end_time).await?;

        // 创建统计报告
        let report = AlertStatisticsReport {
            generated_at: Utc::now(),
            period_start: start_time,
            period_end: end_time,
            basic_statistics: basic_stats,
            frequency_by_day: frequency_stats,
            type_distribution,
            response_time_stats: response_stats,
            node_statistics: node_stats,
        };

        // 序列化为 JSON
        let json = serde_json::to_string_pretty(&report)?;
        write!(output, "{}", json)?;

        info!("Exported alert statistics report");
        Ok(())
    }
}

/// CSV 字段转义
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

// ==================== 导出数据结构 ====================

/// 告警事件导出格式
#[derive(Debug, Clone, Serialize)]
pub struct AlertEventExport {
    pub id: i64,
    pub rule_name: String,
    pub node_name: String,
    pub environment: String,
    pub metric_type: String,
    pub severity: String,
    pub status: String,
    pub metric_value: Option<f64>,
    pub triggered_at: String,
    pub resolved_at: Option<String>,
    pub duration_seconds: Option<i64>,
    pub notification_sent: bool,
    pub notes: Option<String>,
}

impl AlertEventExport {
    fn from_details(event: &AlertEventWithDetails) -> Self {
        let duration_seconds = event.resolved_at.map(|resolved| {
            (resolved - event.triggered_at).num_seconds()
        });

        Self {
            id: event.id,
            rule_name: event.rule_name.clone(),
            node_name: event.node_name.clone(),
            environment: event.environment.clone(),
            metric_type: event.metric_type.clone(),
            severity: event.severity.clone(),
            status: event.status.clone(),
            metric_value: event.metric_value,
            triggered_at: event.triggered_at.to_rfc3339(),
            resolved_at: event.resolved_at.map(|t| t.to_rfc3339()),
            duration_seconds,
            notification_sent: event.notification_sent,
            notes: event.notes.clone(),
        }
    }
}

/// 筛选器摘要
#[derive(Debug, Clone, Serialize)]
pub struct FilterSummary {
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub node_id: Option<i32>,
    pub rule_id: Option<i32>,
    pub status: Option<String>,
    pub severity: Option<String>,
}

impl FilterSummary {
    fn from_filter(filter: &AlertHistoryFilter) -> Self {
        Self {
            start_time: filter.start_time.map(|t| t.to_rfc3339()),
            end_time: filter.end_time.map(|t| t.to_rfc3339()),
            node_id: filter.node_id,
            rule_id: filter.rule_id,
            status: filter.status.clone(),
            severity: filter.severity.clone(),
        }
    }
}

/// 告警报告
#[derive(Debug, Clone, Serialize)]
pub struct AlertReport {
    pub generated_at: DateTime<Utc>,
    pub filter: FilterSummary,
    pub total_count: usize,
    pub events: Vec<AlertEventExport>,
}

/// 告警统计报告
#[derive(Debug, Clone, Serialize)]
pub struct AlertStatisticsReport {
    pub generated_at: DateTime<Utc>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub basic_statistics: crate::storage::alert_store::AlertStatistics,
    pub frequency_by_day: Vec<crate::storage::alert_store::AlertFrequency>,
    pub type_distribution: Vec<crate::storage::alert_store::AlertTypeDistribution>,
    pub response_time_stats: crate::storage::alert_store::AlertResponseStats,
    pub node_statistics: Vec<crate::storage::alert_store::NodeAlertStats>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_escape() {
        assert_eq!(escape_csv("simple"), "simple");
        assert_eq!(escape_csv("with,comma"), "\"with,comma\"");
        assert_eq!(escape_csv("with\"quote"), "\"with\"\"quote\"");
        assert_eq!(escape_csv("with\nnewline"), "\"with\nnewline\"");
    }

    #[test]
    fn test_alert_event_export_creation() {
        // 测试数据结构创建
        let export = AlertEventExport {
            id: 1,
            rule_name: "Test Rule".to_string(),
            node_name: "Test Node".to_string(),
            environment: "test".to_string(),
            metric_type: "test_metric".to_string(),
            severity: "warning".to_string(),
            status: "active".to_string(),
            metric_value: Some(10.5),
            triggered_at: "2025-11-06T14:30:00Z".to_string(),
            resolved_at: None,
            duration_seconds: None,
            notification_sent: true,
            notes: Some("Test note".to_string()),
        };

        assert_eq!(export.id, 1);
        assert_eq!(export.severity, "warning");
    }
}


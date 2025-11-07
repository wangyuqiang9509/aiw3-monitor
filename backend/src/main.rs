use tracing::{error, info};

mod alerting;
mod collectors;
mod config;
mod error;
mod exporter;
mod models;
mod scheduler;
mod storage;
mod utils;

use error::Result;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    utils::logging::init();

    info!("Starting AIWS Monitor v1.0.0");

    // 加载配置
    let settings = config::settings::Settings::new()?;
    info!("Configuration loaded successfully");

    // 连接数据库
    let db_pool = storage::database::create_pool(&settings.database).await?;
    info!("Connected to database");

    // 初始化 Prometheus 指标注册器
    let metrics_registry = exporter::metrics_registry::MetricsRegistry::new()?;
    info!("Prometheus metrics registry initialized");

    // 启动 Prometheus HTTP 服务器
    let metrics_registry_clone = metrics_registry.clone();
    tokio::spawn(async move {
        if let Err(e) = exporter::http_server::start_server(metrics_registry_clone, 9090).await {
            error!("Prometheus exporter failed: {}", e);
        }
    });
    info!("Prometheus exporter listening on :9090");

    // 启动数据采集调度器
    let collector = collectors::metrics_collector::MetricsCollector::new(
        db_pool.clone(),
        metrics_registry.clone(),
    );
    
    let collection_scheduler = scheduler::collection_scheduler::CollectionScheduler::new(
        collector,
        settings.collection.interval_seconds,
    );
    
    tokio::spawn(async move {
        collection_scheduler.run().await;
    });
    info!("Started metrics collection scheduler (interval: {}s)", settings.collection.interval_seconds);

    // 初始化告警引擎
    let alert_engine = Arc::new(alerting::rule_engine::AlertRuleEngine::new(
        Arc::new(db_pool.clone()),
    ));
    info!("Alert engine initialized");

    // 启动告警检查调度器
    let alert_scheduler = scheduler::alert_scheduler::AlertScheduler::new(
        alert_engine.clone(),
        settings.alerting.check_interval_seconds,
    );
    
    tokio::spawn(async move {
        alert_scheduler.start().await;
    });
    info!("Started alert scheduler (interval: {}s)", settings.alerting.check_interval_seconds);

    // 保持主线程运行
    tokio::signal::ctrl_c().await?;
    info!("Shutting down AIWS Monitor");

    Ok(())
}

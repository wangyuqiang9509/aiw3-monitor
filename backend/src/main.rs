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
    )
    .expect("Failed to create MetricsCollector");

    // 为所有启用的节点注册 TPS 和延迟计算器
    let node_store_temp = storage::node_store::NodeStore::new(db_pool.clone());
    match node_store_temp.get_enabled_nodes().await {
        Ok(nodes) => {
            for node in nodes {
                // 注册 TPS 计算器
                collector
                    .register_tps_calculator_with_window(
                        node.name.clone(),
                        60, // 60 秒滑动窗口
                    )
                    .await;
                info!("Registered TPS calculator for node: {}", node.name);

                // 注册延迟计算器
                collector
                    .register_latency_calculator_with_window(
                        node.name.clone(),
                        10, // 10 个区块滑动窗口
                    )
                    .await;
                info!("Registered latency calculator for node: {}", node.name);
            }
        }
        Err(e) => {
            error!("Failed to get enabled nodes for calculator registration: {}", e);
        }
    }

    let collection_scheduler = scheduler::collection_scheduler::CollectionScheduler::new(
        collector,
        settings.collection.interval_seconds,
    );

    tokio::spawn(async move {
        collection_scheduler.run().await;
    });
    info!(
        "Started metrics collection scheduler (interval: {}s)",
        settings.collection.interval_seconds
    );

    // 初始化存储层
    let alert_store = Arc::new(storage::alert_store::AlertStore::new(db_pool.clone()));
    let metrics_store = Arc::new(storage::metrics_store::MetricsStore::new(db_pool.clone()));
    let node_store = Arc::new(storage::node_store::NodeStore::new(db_pool.clone()));

    // 初始化邮件通知器
    let smtp_config = models::smtp_config::SmtpConfig {
        id: 0,
        name: "Default".to_string(),
        server: settings.smtp.server.clone(),
        port: settings.smtp.port as i32,
        username: settings.smtp.username.clone(),
        password: settings.smtp.password.clone(),
        from_address: settings.smtp.from.clone(),
        use_tls: settings.smtp.use_tls,
        is_default: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let notifier = Arc::new(alerting::notifier::RetryableEmailNotifier::new(
        smtp_config,
        settings.alerting.max_retries,
    )?);

    // 初始化告警引擎
    let alert_engine = Arc::new(alerting::rule_engine::AlertRuleEngine::new(
        alert_store,
        metrics_store,
        node_store,
        notifier,
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

// Prometheus HTTP 服务器
use warp::{Filter, Reply};
use tracing::{info, error};

use crate::error::Result;
use crate::exporter::metrics_registry::MetricsRegistry;

/// 启动 Prometheus HTTP 服务器
pub async fn start_server(registry: MetricsRegistry, port: u16) -> Result<()> {
    info!("Starting Prometheus HTTP server on port {}", port);

    // /metrics 端点
    let metrics_route = warp::path("metrics")
        .and(warp::get())
        .and(with_registry(registry.clone()))
        .and_then(metrics_handler);

    // /health 健康检查端点
    let health_route = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::with_status("OK", warp::http::StatusCode::OK));

    // 根路径
    let root_route = warp::path::end()
        .and(warp::get())
        .map(|| {
            warp::reply::html(
                r#"
                <html>
                <head><title>AIWS Monitor</title></head>
                <body>
                    <h1>AIWS Blockchain Monitor</h1>
                    <p>Prometheus metrics exporter</p>
                    <ul>
                        <li><a href="/metrics">Metrics</a></li>
                        <li><a href="/health">Health Check</a></li>
                    </ul>
                </body>
                </html>
                "#,
            )
        });

    // 添加 CORS 支持
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "OPTIONS"])
        .allow_headers(vec!["Content-Type"]);

    let routes = metrics_route.or(health_route).or(root_route).with(cors);

    info!("Prometheus HTTP server listening on 0.0.0.0:{}", port);

    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;

    Ok(())
}

/// 注入 MetricsRegistry 到 handler
fn with_registry(
    registry: MetricsRegistry,
) -> impl Filter<Extract = (MetricsRegistry,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || registry.clone())
}

/// /metrics 端点处理器
async fn metrics_handler(
    registry: MetricsRegistry,
) -> std::result::Result<impl Reply, warp::Rejection> {
    match registry.export() {
        Ok(metrics) => Ok(warp::reply::with_header(
            metrics,
            "Content-Type",
            "text/plain; version=0.0.4",
        )),
        Err(e) => {
            error!("Failed to export metrics: {}", e);
            Ok(warp::reply::with_header(
                format!("Error: {}", e),
                "Content-Type",
                "text/plain",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_handler() {
        let registry = MetricsRegistry::new().unwrap();
        registry.set_block_height("test", "devnet", "test-chain", 100.0);

        let result = metrics_handler(registry).await;
        assert!(result.is_ok());
    }
}

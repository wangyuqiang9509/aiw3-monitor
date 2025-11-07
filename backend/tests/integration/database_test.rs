use sqlx::PgPool;

/// 获取测试数据库连接池
async fn get_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:secret@localhost:5432/aiw3_monitor_test".to_string());
    
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create test pool")
}

#[tokio::test]
#[ignore] // 需要数据库连接,使用 cargo test -- --ignored 运行
async fn test_database_connection() {
    let pool = get_test_pool().await;
    
    // 测试简单查询
    let result = sqlx::query("SELECT 1 as value")
        .fetch_one(&pool)
        .await;
    
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_blockchain_nodes_table_exists() {
    let pool = get_test_pool().await;
    
    // 检查 blockchain_nodes 表是否存在
    let result = sqlx::query(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_name = 'blockchain_nodes'
        )"
    )
    .fetch_one(&pool)
    .await;
    
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_metric_data_table_exists() {
    let pool = get_test_pool().await;
    
    // 检查 metric_data 表是否存在
    let result = sqlx::query(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_name = 'metric_data'
        )"
    )
    .fetch_one(&pool)
    .await;
    
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_insert_and_query_node() {
    let pool = get_test_pool().await;
    
    // 清理测试数据
    sqlx::query("DELETE FROM blockchain_nodes WHERE name LIKE 'Test Node%'")
        .execute(&pool)
        .await
        .unwrap();
    
    // 插入测试节点
    let result = sqlx::query(
        "INSERT INTO blockchain_nodes 
        (name, rpc_url, rest_url, grpc_url, environment, labels) 
        VALUES ($1, $2, $3, $4, $5, $6) 
        RETURNING id"
    )
    .bind("Test Node 1")
    .bind("https://test-rpc.example.com")
    .bind("https://test-api.example.com")
    .bind("https://test-grpc.example.com:443")
    .bind("test")
    .bind(serde_json::json!({"region": "test"}))
    .fetch_one(&pool)
    .await;
    
    assert!(result.is_ok());
    
    // 查询节点
    let nodes = sqlx::query("SELECT * FROM blockchain_nodes WHERE name = $1")
        .bind("Test Node 1")
        .fetch_all(&pool)
        .await
        .unwrap();
    
    assert_eq!(nodes.len(), 1);
    
    // 清理
    sqlx::query("DELETE FROM blockchain_nodes WHERE name = $1")
        .bind("Test Node 1")
        .execute(&pool)
        .await
        .unwrap();
}


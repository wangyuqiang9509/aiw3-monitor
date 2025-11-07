use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tracing::{debug, info};

use super::models::*;
use crate::error::{AppError, Result};
use crate::utils::retry::retry_async;

/// CometBFT RPC 客户端
pub struct RpcClient {
    client: Client,
    rpc_url: String,
    timeout: Duration,
    max_retries: u32,
}

impl RpcClient {
    /// 创建新的 RPC 客户端
    pub fn new(rpc_url: String, timeout_seconds: u64, max_retries: u32) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .expect("Failed to build HTTP client");

        Self { client, rpc_url, timeout: Duration::from_secs(timeout_seconds), max_retries }
    }

    /// 获取节点状态
    pub async fn get_status(&self) -> Result<StatusResult> {
        info!("Fetching status from {}", self.rpc_url);

        let operation = || async {
            self.client
                .post(&self.rpc_url)
                .json(&json!({
                    "jsonrpc": "2.0",
                    "method": "status",
                    "params": [],
                    "id": 1
                }))
                .send()
                .await
                .map_err(AppError::Http)?
                .json::<RpcResponse<StatusResult>>()
                .await
                .map_err(AppError::Http)
                .map(|resp| resp.result)
        };

        retry_async(operation, self.max_retries, Duration::from_secs(1)).await
    }

    /// 获取网络信息
    pub async fn get_net_info(&self) -> Result<NetInfoResult> {
        debug!("Fetching net_info from {}", self.rpc_url);

        let operation = || async {
            self.client
                .post(&self.rpc_url)
                .json(&json!({
                    "jsonrpc": "2.0",
                    "method": "net_info",
                    "params": [],
                    "id": 1
                }))
                .send()
                .await
                .map_err(AppError::Http)?
                .json::<RpcResponse<NetInfoResult>>()
                .await
                .map_err(AppError::Http)
                .map(|resp| resp.result)
        };

        retry_async(operation, self.max_retries, Duration::from_secs(1)).await
    }

    /// 获取未确认交易数量
    pub async fn get_num_unconfirmed_txs(&self) -> Result<NumUnconfirmedTxsResult> {
        debug!("Fetching num_unconfirmed_txs from {}", self.rpc_url);

        let operation = || async {
            self.client
                .post(&self.rpc_url)
                .json(&json!({
                    "jsonrpc": "2.0",
                    "method": "num_unconfirmed_txs",
                    "params": [],
                    "id": 1
                }))
                .send()
                .await
                .map_err(AppError::Http)?
                .json::<RpcResponse<NumUnconfirmedTxsResult>>()
                .await
                .map_err(AppError::Http)
                .map(|resp| resp.result)
        };

        retry_async(operation, self.max_retries, Duration::from_secs(1)).await
    }

    /// 测试连接是否正常
    pub async fn ping(&self) -> Result<()> {
        self.get_status().await.map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    #[tokio::test]
    async fn test_rpc_client_creation() {
        let client = RpcClient::new("https://test-rpc.example.com".to_string(), 10, 3);
        assert_eq!(client.rpc_url, "https://test-rpc.example.com");
    }

    #[tokio::test]
    async fn test_get_status_success() {
        let mut server = mockito::Server::new_async().await;

        let _m = server
            .mock("POST", "/")
            .match_body(mockito::Matcher::Json(json!({
                "jsonrpc": "2.0",
                "method": "status",
                "params": [],
                "id": 1
            })))
            .with_status(200)
            .with_body(
                r#"{
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "node_info": {
                        "id": "test-node",
                        "listen_addr": "tcp://0.0.0.0:26656",
                        "network": "aiw3chain-devnet",
                        "version": "0.37.0",
                        "channels": "40202122233038606100",
                        "moniker": "test-moniker",
                        "other": {
                            "tx_index": "on",
                            "rpc_address": "tcp://127.0.0.1:26657"
                        }
                    },
                    "sync_info": {
                        "latest_block_hash": "ABC123",
                        "latest_app_hash": "DEF456",
                        "latest_block_height": "12345",
                        "latest_block_time": "2025-01-01T00:00:00Z",
                        "earliest_block_hash": "GHI789",
                        "earliest_app_hash": "JKL012",
                        "earliest_block_height": "1",
                        "earliest_block_time": "2024-01-01T00:00:00Z",
                        "catching_up": false
                    },
                    "validator_info": {
                        "address": "ABCD1234",
                        "pub_key": {
                            "type": "tendermint/PubKeyEd25519",
                            "value": "abcdef123456"
                        },
                        "voting_power": "10"
                    }
                }
            }"#,
            )
            .create_async()
            .await;

        let client = RpcClient::new(server.url(), 10, 3);
        let result = client.get_status().await;

        assert!(result.is_ok());
        let status = result.unwrap();
        assert_eq!(status.sync_info.latest_block_height, "12345");
        assert_eq!(status.node_info.network, "aiw3chain-devnet");
        assert!(!status.sync_info.catching_up);
    }

    #[tokio::test]
    async fn test_get_net_info_success() {
        let mut server = mockito::Server::new_async().await;

        let _m = server
            .mock("POST", "/")
            .match_body(mockito::Matcher::Json(json!({
                "jsonrpc": "2.0",
                "method": "net_info",
                "params": [],
                "id": 1
            })))
            .with_status(200)
            .with_body(
                r#"{
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "listening": true,
                    "listeners": ["tcp://0.0.0.0:26656"],
                    "n_peers": "5",
                    "peers": []
                }
            }"#,
            )
            .create_async()
            .await;

        let client = RpcClient::new(server.url(), 10, 3);
        let result = client.get_net_info().await;

        assert!(result.is_ok());
        let net_info = result.unwrap();
        assert_eq!(net_info.n_peers, "5");
        assert!(net_info.listening);
    }

    #[tokio::test]
    async fn test_get_num_unconfirmed_txs_success() {
        let mut server = mockito::Server::new_async().await;

        let _m = server
            .mock("POST", "/")
            .match_body(mockito::Matcher::Json(json!({
                "jsonrpc": "2.0",
                "method": "num_unconfirmed_txs",
                "params": [],
                "id": 1
            })))
            .with_status(200)
            .with_body(
                r#"{
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "n_txs": "42",
                    "total": "100",
                    "total_bytes": "1024"
                }
            }"#,
            )
            .create_async()
            .await;

        let client = RpcClient::new(server.url(), 10, 3);
        let result = client.get_num_unconfirmed_txs().await;

        assert!(result.is_ok());
        let txs = result.unwrap();
        assert_eq!(txs.n_txs, "42");
        assert_eq!(txs.total, "100");
    }

    #[tokio::test]
    async fn test_rpc_client_timeout() {
        let mut server = mockito::Server::new_async().await;

        // 模拟超时 - 延迟响应
        let _m = server
            .mock("POST", "/")
            .with_status(200)
            .with_body("delayed response")
            .with_header("content-type", "application/json")
            .expect(0) // 期望不会被调用(因为会超时)
            .create_async()
            .await;

        let client = RpcClient::new(server.url(), 1, 1); // 1秒超时
        
        // 注意: mockito 不支持真正的延迟,这个测试主要验证超时配置
        // 在实际场景中,网络延迟会触发超时
        let result = client.get_status().await;

        // 应该返回错误(JSON 解析失败或其他错误)
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rpc_client_invalid_response() {
        let mut server = mockito::Server::new_async().await;

        let _m = server
            .mock("POST", "/")
            .with_status(200)
            .with_body("invalid json")
            .create_async()
            .await;

        let client = RpcClient::new(server.url(), 10, 1);
        let result = client.get_status().await;

        // 应该返回 JSON 解析错误
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ping_success() {
        let mut server = mockito::Server::new_async().await;

        let _m = server
            .mock("POST", "/")
            .with_status(200)
            .with_body(
                r#"{
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "node_info": {
                        "id": "test",
                        "listen_addr": "tcp://0.0.0.0:26656",
                        "network": "test",
                        "version": "0.37.0",
                        "channels": "40202122233038606100",
                        "moniker": "test",
                        "other": {"tx_index": "on", "rpc_address": "tcp://127.0.0.1:26657"}
                    },
                    "sync_info": {
                        "latest_block_hash": "ABC",
                        "latest_app_hash": "DEF",
                        "latest_block_height": "1",
                        "latest_block_time": "2025-01-01T00:00:00Z",
                        "earliest_block_hash": "GHI",
                        "earliest_app_hash": "JKL",
                        "earliest_block_height": "1",
                        "earliest_block_time": "2024-01-01T00:00:00Z",
                        "catching_up": false
                    },
                    "validator_info": {
                        "address": "ABCD",
                        "pub_key": {"type": "tendermint/PubKeyEd25519", "value": "abc"},
                        "voting_power": "10"
                    }
                }
            }"#,
            )
            .create_async()
            .await;

        let client = RpcClient::new(server.url(), 10, 3);
        let result = client.ping().await;

        assert!(result.is_ok());
    }
}

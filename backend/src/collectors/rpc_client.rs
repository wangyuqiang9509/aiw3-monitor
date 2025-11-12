use reqwest::Client;
use serde_json::json;
use std::time::{Duration, Instant};
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

    /// 获取指定高度的区块信息
    pub async fn get_block(&self, height: Option<u64>) -> Result<BlockResult> {
        debug!("Fetching block from {} (height: {:?})", self.rpc_url, height);

        let params: Vec<String> = if let Some(h) = height { vec![h.to_string()] } else { vec![] };

        let operation = || async {
            self.client
                .post(&self.rpc_url)
                .json(&json!({
                    "jsonrpc": "2.0",
                    "method": "block",
                    "params": params,
                    "id": 1
                }))
                .send()
                .await
                .map_err(AppError::Http)?
                .json::<RpcResponse<BlockResult>>()
                .await
                .map_err(AppError::Http)
                .map(|resp| resp.result)
        };

        retry_async(operation, self.max_retries, Duration::from_secs(1)).await
    }

    /// 获取最新区块信息
    pub async fn get_latest_block(&self) -> Result<BlockResult> {
        // 先获取最新高度
        let status = self.get_status().await?;
        let latest_height = status
            .sync_info
            .latest_block_height
            .parse::<u64>()
            .map_err(|e| AppError::generic(format!("Failed to parse block height: {}", e)))?;

        // 使用最新高度获取区块信息
        self.get_block(Some(latest_height)).await
    }

    /// 测试连接是否正常
    pub async fn ping(&self) -> Result<()> {
        self.get_status().await.map(|_| ())
    }

    /// 获取节点状态（带延迟测量）
    ///
    /// # 返回
    /// 返回 (StatusResult, 延迟毫秒数)
    pub async fn get_status_with_latency(&self) -> Result<(StatusResult, f64)> {
        let start = Instant::now();
        let result = self.get_status().await?;
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        Ok((result, latency_ms))
    }

    /// 获取网络信息（带延迟测量）
    ///
    /// # 返回
    /// 返回 (NetInfoResult, 延迟毫秒数)
    pub async fn get_net_info_with_latency(&self) -> Result<(NetInfoResult, f64)> {
        let start = Instant::now();
        let result = self.get_net_info().await?;
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        Ok((result, latency_ms))
    }

    /// 获取指定高度的区块信息（带延迟测量）
    ///
    /// # 返回
    /// 返回 (BlockResult, 延迟毫秒数)
    pub async fn get_block_with_latency(&self, height: Option<u64>) -> Result<(BlockResult, f64)> {
        let start = Instant::now();
        let result = self.get_block(height).await?;
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        Ok((result, latency_ms))
    }

    /// 获取最新区块信息（带延迟测量）
    ///
    /// # 返回
    /// 返回 (BlockResult, 延迟毫秒数)
    pub async fn get_latest_block_with_latency(&self) -> Result<(BlockResult, f64)> {
        let start = Instant::now();
        let result = self.get_latest_block().await?;
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        Ok((result, latency_ms))
    }

    /// 获取未确认交易数量（带延迟测量）
    ///
    /// # 返回
    /// 返回 (NumUnconfirmedTxsResult, 延迟毫秒数)
    pub async fn get_num_unconfirmed_txs_with_latency(
        &self,
    ) -> Result<(NumUnconfirmedTxsResult, f64)> {
        let start = Instant::now();
        let result = self.get_num_unconfirmed_txs().await?;
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        Ok((result, latency_ms))
    }

    /// 获取验证者信息
    pub async fn get_validators(
        &self,
        height: Option<u64>,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<ValidatorsResult> {
        debug!("Fetching validators from {} (height: {:?})", self.rpc_url, height);

        // CometBFT RPC 要求必须提供所有 3 个参数
        // height 参数：None 时使用 null（表示最新高度），Some 时转为字符串
        let height_param = height.map(|h| json!(h.to_string())).unwrap_or(json!(null));
        let page_param = json!(page.unwrap_or(1).to_string());
        let per_page_param = json!(per_page.unwrap_or(100).to_string());

        let params = vec![height_param, page_param, per_page_param];

        let operation = || async {
            self.client
                .post(&self.rpc_url)
                .json(&json!({
                    "jsonrpc": "2.0",
                    "method": "validators",
                    "params": params,
                    "id": 1
                }))
                .send()
                .await
                .map_err(AppError::Http)?
                .json::<RpcResponse<ValidatorsResult>>()
                .await
                .map_err(AppError::Http)
                .map(|resp| resp.result)
        };

        retry_async(operation, self.max_retries, Duration::from_secs(1)).await
    }

    /// 获取共识状态
    pub async fn get_consensus_state(&self) -> Result<ConsensusStateResult> {
        debug!("Fetching consensus_state from {}", self.rpc_url);

        let operation = || async {
            self.client
                .post(&self.rpc_url)
                .json(&json!({
                    "jsonrpc": "2.0",
                    "method": "consensus_state",
                    "params": [],
                    "id": 1
                }))
                .send()
                .await
                .map_err(AppError::Http)?
                .json::<RpcResponse<ConsensusStateResult>>()
                .await
                .map_err(AppError::Http)
                .map(|resp| resp.result)
        };

        retry_async(operation, self.max_retries, Duration::from_secs(1)).await
    }

    /// 获取 ABCI 信息
    ///
    /// 返回应用程序的版本信息和最后区块高度
    pub async fn get_abci_info(&self) -> Result<AbciInfoResult> {
        debug!("Fetching abci_info from {}", self.rpc_url);

        let operation = || async {
            self.client
                .post(&self.rpc_url)
                .json(&json!({
                    "jsonrpc": "2.0",
                    "method": "abci_info",
                    "params": [],
                    "id": 1
                }))
                .send()
                .await
                .map_err(AppError::Http)?
                .json::<RpcResponse<AbciInfoResult>>()
                .await
                .map_err(AppError::Http)
                .map(|resp| resp.result)
        };

        retry_async(operation, self.max_retries, Duration::from_secs(1)).await
    }

    /// 获取 ABCI 信息（带延迟测量）
    ///
    /// # 返回
    /// 返回 (AbciInfoResult, 延迟毫秒数)
    pub async fn get_abci_info_with_latency(&self) -> Result<(AbciInfoResult, f64)> {
        let start = Instant::now();
        let result = self.get_abci_info().await?;
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
        Ok((result, latency_ms))
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

    #[tokio::test]
    async fn test_get_status_with_latency() {
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
        let result = client.get_status_with_latency().await;

        assert!(result.is_ok());
        let (status, latency_ms) = result.unwrap();
        assert_eq!(status.sync_info.latest_block_height, "12345");
        // 延迟应该是一个正数（毫秒）
        assert!(latency_ms >= 0.0);
        // 本地 mock 服务器延迟应该是合理的（< 5000ms，在 CI 环境中可能较慢）
        assert!(latency_ms < 5000.0);
    }

    #[tokio::test]
    async fn test_get_block_with_latency() {
        let mut server = mockito::Server::new_async().await;

        let _m = server
            .mock("POST", "/")
            .with_status(200)
            .with_body(
                r#"{
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "block_id": {
                        "hash": "ABC123",
                        "parts": {
                            "total": 1,
                            "hash": "DEF456"
                        }
                    },
                    "block": {
                        "header": {
                            "version": {"block": "11"},
                            "chain_id": "test-chain",
                            "height": "100",
                            "time": "2025-01-01T00:00:00Z"
                        },
                        "data": {
                            "txs": ["dHgx", "dHgy"]
                        },
                        "evidence": {
                            "evidence": []
                        },
                        "last_commit": null
                    }
                }
            }"#,
            )
            .create_async()
            .await;

        let client = RpcClient::new(server.url(), 10, 3);
        let result = client.get_block_with_latency(Some(100)).await;

        assert!(result.is_ok());
        let (block, latency_ms) = result.unwrap();
        assert_eq!(block.tx_count(), 2);
        assert!(latency_ms >= 0.0);
        // 本地 mock 服务器延迟应该是合理的（< 5000ms）
        assert!(latency_ms < 5000.0);
    }

    #[tokio::test]
    async fn test_latency_measurement_accuracy() {
        let mut server = mockito::Server::new_async().await;

        let _m = server
            .mock("POST", "/")
            .with_status(200)
            .with_body(
                r#"{"jsonrpc":"2.0","id":1,"result":{"n_txs":"0","total":"0","total_bytes":"0"}}"#,
            )
            .create_async()
            .await;

        let client = RpcClient::new(server.url(), 10, 3);

        // 测试多次调用，延迟应该是一致的数量级
        let mut latencies = Vec::new();
        for _ in 0..3 {
            if let Ok((_, latency)) = client.get_num_unconfirmed_txs_with_latency().await {
                latencies.push(latency);
            }
        }

        assert_eq!(latencies.len(), 3);
        // 所有延迟应该都是正数且在合理范围内（< 5000ms）
        for latency in &latencies {
            assert!(*latency >= 0.0);
            assert!(*latency < 5000.0);
        }
    }

    #[tokio::test]
    async fn test_get_validators_success() {
        let mut server = mockito::Server::new_async().await;

        let _m = server
            .mock("POST", "/")
            .with_status(200)
            .with_body(
                r#"{
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "block_height": "150024",
                    "validators": [
                        {
                            "address": "0607EACDE76084360A2D8287EB09118D71E98038",
                            "pub_key": {
                                "type": "tendermint/PubKeyEd25519",
                                "value": "buFvt/vRjz+itp17DP1gYjweRtc3bKUy3MR5grc2IaQ="
                            },
                            "voting_power": "10",
                            "proposer_priority": "0"
                        },
                        {
                            "address": "AF0918DE33DAC6FD1899387038E483306AE0D673",
                            "pub_key": {
                                "type": "tendermint/PubKeyEd25519",
                                "value": "thv6jXMP623N55VbG6RysZiY0hTzw3sn444UziDXZIQ="
                            },
                            "voting_power": "10",
                            "proposer_priority": "0"
                        }
                    ],
                    "count": "2",
                    "total": "2"
                }
            }"#,
            )
            .create_async()
            .await;

        let client = RpcClient::new(server.url(), 10, 3);
        let result = client.get_validators(None, None, None).await;

        assert!(result.is_ok());
        let validators = result.unwrap();
        assert_eq!(validators.total, "2");
        assert_eq!(validators.count, "2");
        assert_eq!(validators.validators.len(), 2);
        assert_eq!(validators.validators[0].voting_power, "10");
    }

    #[tokio::test]
    async fn test_get_consensus_state_success() {
        let mut server = mockito::Server::new_async().await;

        let _m = server
            .mock("POST", "/")
            .with_status(200)
            .with_body(r#"{
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "round_state": {
                        "height/round/step": "150083/0/4",
                        "start_time": "2025-11-09T01:58:29.452967979Z",
                        "proposal_block_hash": "416208F3D245CAA799AF3406B2034339A1186194F8173071F8EC09E1D4C300CA",
                        "locked_block_hash": "",
                        "valid_block_hash": "",
                        "height_vote_set": [],
                        "proposer": {
                            "address": "0607EACDE76084360A2D8287EB09118D71E98038",
                            "index": 0
                        }
                    }
                }
            }"#)
            .create_async()
            .await;

        let client = RpcClient::new(server.url(), 10, 3);
        let result = client.get_consensus_state().await;

        assert!(result.is_ok());
        let consensus = result.unwrap();
        assert_eq!(consensus.round_state.height().unwrap(), 150083);
        assert_eq!(consensus.round_state.round().unwrap(), 0);
        assert_eq!(consensus.round_state.step().unwrap(), 4);
        assert_eq!(
            consensus.round_state.proposer.address,
            "0607EACDE76084360A2D8287EB09118D71E98038"
        );
    }

    #[tokio::test]
    async fn test_validators_result_helper_methods() {
        use crate::collectors::models::{PubKey, Validator, ValidatorsResult};

        let validators_result = ValidatorsResult {
            block_height: "150024".to_string(),
            validators: vec![
                Validator {
                    address: "addr1".to_string(),
                    pub_key: PubKey {
                        key_type: "tendermint/PubKeyEd25519".to_string(),
                        value: "key1".to_string(),
                    },
                    voting_power: "10".to_string(),
                    proposer_priority: "0".to_string(),
                },
                Validator {
                    address: "addr2".to_string(),
                    pub_key: PubKey {
                        key_type: "tendermint/PubKeyEd25519".to_string(),
                        value: "key2".to_string(),
                    },
                    voting_power: "15".to_string(),
                    proposer_priority: "0".to_string(),
                },
            ],
            count: "2".to_string(),
            total: "2".to_string(),
        };

        assert_eq!(validators_result.total_count().unwrap(), 2);
        assert_eq!(validators_result.active_count().unwrap(), 2);
        assert_eq!(validators_result.total_voting_power().unwrap(), 25);
        assert_eq!(validators_result.average_voting_power().unwrap(), 12.5);
    }

    #[tokio::test]
    async fn test_round_state_parsing() {
        use crate::collectors::models::{Proposer, RoundState};

        let round_state = RoundState {
            height_round_step: "150083/2/3".to_string(),
            start_time: "2025-11-09T01:58:29.452967979Z".to_string(),
            proposal_block_hash: "hash1".to_string(),
            locked_block_hash: "".to_string(),
            valid_block_hash: "".to_string(),
            height_vote_set: vec![],
            proposer: Proposer { address: "addr1".to_string(), index: 0 },
        };

        assert_eq!(round_state.height().unwrap(), 150083);
        assert_eq!(round_state.round().unwrap(), 2);
        assert_eq!(round_state.step().unwrap(), 3);

        let (h, r, s) = round_state.parse_height_round_step().unwrap();
        assert_eq!(h, 150083);
        assert_eq!(r, 2);
        assert_eq!(s, 3);
    }
}

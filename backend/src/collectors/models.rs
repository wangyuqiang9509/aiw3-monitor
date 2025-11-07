use serde::{Deserialize, Serialize};

/// CometBFT RPC 标准响应包装
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub id: i64,
    pub result: T,
}

/// 错误响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<String>,
}

/// Status 响应 (来自 /status 端点)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResult {
    pub node_info: NodeInfo,
    pub sync_info: SyncInfo,
    pub validator_info: ValidatorInfo,
}

/// 节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub listen_addr: String,
    pub network: String,
    pub version: String,
    pub channels: String,
    pub moniker: String,
    pub other: NodeInfoOther,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfoOther {
    pub tx_index: String,
    pub rpc_address: String,
}

/// 同步信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncInfo {
    pub latest_block_hash: String,
    pub latest_app_hash: String,
    pub latest_block_height: String,
    pub latest_block_time: String,
    pub earliest_block_hash: String,
    pub earliest_app_hash: String,
    pub earliest_block_height: String,
    pub earliest_block_time: String,
    pub catching_up: bool,
}

/// 验证者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub address: String,
    pub pub_key: PubKey,
    pub voting_power: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubKey {
    #[serde(rename = "type")]
    pub key_type: String,
    pub value: String,
}

/// NetInfo 响应 (来自 /net_info 端点)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetInfoResult {
    pub listening: bool,
    pub listeners: Vec<String>,
    pub n_peers: String,
    pub peers: Vec<Peer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub node_info: NodeInfo,
    pub is_outbound: bool,
    pub connection_status: ConnectionStatus,
    pub remote_ip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    #[serde(rename = "Duration")]
    pub duration: String,
    #[serde(rename = "SendMonitor")]
    pub send_monitor: Monitor,
    #[serde(rename = "RecvMonitor")]
    pub recv_monitor: Monitor,
    #[serde(rename = "Channels")]
    pub channels: Vec<Channel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    #[serde(rename = "Active")]
    pub active: bool,
    #[serde(rename = "Start")]
    pub start: String,
    #[serde(rename = "Bytes")]
    pub bytes: String,
    #[serde(rename = "Samples")]
    pub samples: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "SendQueueCapacity")]
    pub send_queue_capacity: String,
    #[serde(rename = "SendQueueSize")]
    pub send_queue_size: String,
    #[serde(rename = "Priority")]
    pub priority: String,
}

/// NumUnconfirmedTxs 响应 (来自 /num_unconfirmed_txs 端点)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumUnconfirmedTxsResult {
    pub n_txs: String,
    pub total: String,
    pub total_bytes: String,
}

impl SyncInfo {
    /// 获取区块高度 (u64)
    pub fn block_height(&self) -> Result<u64, std::num::ParseIntError> {
        self.latest_block_height.parse()
    }
}

impl NetInfoResult {
    /// 获取节点数量 (u64)
    pub fn peer_count(&self) -> Result<u64, std::num::ParseIntError> {
        self.n_peers.parse()
    }
}

impl NumUnconfirmedTxsResult {
    /// 获取未确认交易数量 (u64)
    pub fn tx_count(&self) -> Result<u64, std::num::ParseIntError> {
        self.n_txs.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_info_block_height() {
        let sync_info = SyncInfo {
            latest_block_hash: "hash".to_string(),
            latest_app_hash: "app_hash".to_string(),
            latest_block_height: "12345".to_string(),
            latest_block_time: "2025-01-01T00:00:00Z".to_string(),
            earliest_block_hash: "hash".to_string(),
            earliest_app_hash: "app_hash".to_string(),
            earliest_block_height: "1".to_string(),
            earliest_block_time: "2024-01-01T00:00:00Z".to_string(),
            catching_up: false,
        };

        assert_eq!(sync_info.block_height().unwrap(), 12345);
    }
}

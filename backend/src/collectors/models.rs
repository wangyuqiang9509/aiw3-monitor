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

/// Block 响应 (来自 /block 端点)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResult {
    pub block_id: BlockId,
    pub block: Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockId {
    pub hash: String,
    pub parts: BlockParts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockParts {
    pub total: u32,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub data: BlockData,
    pub evidence: Evidence,
    pub last_commit: Option<LastCommit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: BlockVersion,
    pub chain_id: String,
    pub height: String,
    pub time: String,
    // 其他字段可选，根据需要添加
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockVersion {
    pub block: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockData {
    pub txs: Vec<String>, // Base64 编码的交易数据
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastCommit {
    pub height: String,
    pub round: u32,
    pub block_id: BlockId,
    pub signatures: Vec<Signature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub block_id_flag: u32,
    pub validator_address: String,
    pub timestamp: String,
    pub signature: String,
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

impl BlockResult {
    /// 获取区块高度
    pub fn block_height(&self) -> Result<u64, std::num::ParseIntError> {
        self.block.header.height.parse()
    }

    /// 获取交易数量
    pub fn tx_count(&self) -> u64 {
        self.block.data.txs.len() as u64
    }

    /// 获取区块时间
    pub fn block_time(&self) -> &str {
        &self.block.header.time
    }

    /// 获取区块时间戳（返回 Result）
    pub fn block_timestamp(&self) -> Result<String, ()> {
        Ok(self.block.header.time.clone())
    }
}

/// Validators 响应 (来自 /validators 端点)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorsResult {
    pub block_height: String,
    pub validators: Vec<Validator>,
    pub count: String,
    pub total: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub address: String,
    pub pub_key: PubKey,
    pub voting_power: String,
    pub proposer_priority: String,
}

impl ValidatorsResult {
    /// 获取验证者总数
    pub fn total_count(&self) -> Result<u64, std::num::ParseIntError> {
        self.total.parse()
    }

    /// 获取活跃验证者数量
    pub fn active_count(&self) -> Result<u64, std::num::ParseIntError> {
        self.count.parse()
    }

    /// 计算总投票权
    pub fn total_voting_power(&self) -> Result<i64, std::num::ParseIntError> {
        self.validators.iter().map(|v| v.voting_power.parse::<i64>()).sum()
    }

    /// 计算平均投票权
    pub fn average_voting_power(&self) -> Result<f64, std::num::ParseIntError> {
        let total = self.total_voting_power()?;
        let count = self.active_count()? as f64;
        Ok(if count > 0.0 { total as f64 / count } else { 0.0 })
    }
}

/// ConsensusState 响应 (来自 /consensus_state 端点)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusStateResult {
    pub round_state: RoundState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundState {
    #[serde(rename = "height/round/step")]
    pub height_round_step: String, // 格式: "150083/0/4"
    pub start_time: String,
    pub proposal_block_hash: String,
    pub locked_block_hash: String,
    pub valid_block_hash: String,
    pub height_vote_set: Vec<serde_json::Value>,
    pub proposer: Proposer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposer {
    pub address: String,
    pub index: u32,
}

impl RoundState {
    /// 解析高度/轮次/步骤
    pub fn parse_height_round_step(&self) -> Result<(u64, u32, u32), String> {
        let parts: Vec<&str> = self.height_round_step.split('/').collect();
        if parts.len() != 3 {
            return Err("Invalid height/round/step format".to_string());
        }

        let height =
            parts[0].parse::<u64>().map_err(|e| format!("Failed to parse height: {}", e))?;
        let round = parts[1].parse::<u32>().map_err(|e| format!("Failed to parse round: {}", e))?;
        let step = parts[2].parse::<u32>().map_err(|e| format!("Failed to parse step: {}", e))?;

        Ok((height, round, step))
    }

    /// 获取当前高度
    pub fn height(&self) -> Result<u64, String> {
        self.parse_height_round_step().map(|(h, _, _)| h)
    }

    /// 获取当前轮次
    pub fn round(&self) -> Result<u32, String> {
        self.parse_height_round_step().map(|(_, r, _)| r)
    }

    /// 获取当前步骤
    pub fn step(&self) -> Result<u32, String> {
        self.parse_height_round_step().map(|(_, _, s)| s)
    }
}

/// AbciInfo 响应 (来自 /abci_info 端点)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbciInfoResult {
    pub response: AbciResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbciResponse {
    pub data: String,
    pub version: String,
    pub last_block_height: String,
    pub last_block_app_hash: String,
}

impl AbciInfoResult {
    /// 获取应用版本
    pub fn version(&self) -> &str {
        &self.response.version
    }

    /// 获取最后区块高度
    pub fn last_block_height(&self) -> Result<u64, std::num::ParseIntError> {
        self.response.last_block_height.parse()
    }

    /// 从版本号推断是否启用了 Block-STM
    pub fn infer_block_stm_enabled(&self) -> bool {
        self.response.version.to_lowercase().contains("blockstm")
    }

    /// 从版本号推断是否启用了 MemIAVL
    pub fn infer_memiavl_enabled(&self) -> bool {
        self.response.version.to_lowercase().contains("memiavl")
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

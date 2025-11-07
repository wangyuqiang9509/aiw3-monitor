-- 创建区块链节点表
CREATE TABLE IF NOT EXISTS blockchain_nodes (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    rpc_url VARCHAR(512) NOT NULL,
    rest_url VARCHAR(512) NOT NULL,
    grpc_url VARCHAR(512) NOT NULL,
    environment VARCHAR(50) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    labels JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引
CREATE INDEX idx_nodes_environment ON blockchain_nodes(environment);
CREATE INDEX idx_nodes_enabled ON blockchain_nodes(enabled);

-- 创建更新时间触发器
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_nodes_updated_at BEFORE UPDATE ON blockchain_nodes
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 插入示例节点
INSERT INTO blockchain_nodes (name, rpc_url, rest_url, grpc_url, environment, labels)
VALUES (
    'AIWS DevNet 3',
    'https://devnet-rpc3.aiw3.io',
    'https://devnet-api3.aiw3.io',
    'https://devnet-grpc3.aiw3.io:443',
    'devnet',
    '{"region": "us-west", "chain_id": "aiw3chain-devnet"}'::jsonb
);


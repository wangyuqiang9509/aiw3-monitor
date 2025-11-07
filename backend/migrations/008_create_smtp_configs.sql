-- 创建 SMTP 配置表
CREATE TABLE IF NOT EXISTS smtp_configs (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    server VARCHAR(255) NOT NULL,
    port INTEGER NOT NULL DEFAULT 587,
    username VARCHAR(255) NOT NULL,
    password_encrypted TEXT NOT NULL, -- 使用 AES-256 加密存储
    from_address VARCHAR(255) NOT NULL,
    from_name VARCHAR(255),
    use_tls BOOLEAN NOT NULL DEFAULT true,
    use_starttls BOOLEAN NOT NULL DEFAULT true,
    timeout_seconds INTEGER NOT NULL DEFAULT 30,
    is_default BOOLEAN NOT NULL DEFAULT false,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建邮件接收人表
CREATE TABLE IF NOT EXISTS alert_recipients (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建告警规则与接收人的关联表
CREATE TABLE IF NOT EXISTS alert_rule_recipients (
    rule_id INTEGER NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    recipient_id INTEGER NOT NULL REFERENCES alert_recipients(id) ON DELETE CASCADE,
    PRIMARY KEY (rule_id, recipient_id)
);

-- 创建索引
CREATE INDEX idx_smtp_configs_enabled ON smtp_configs(enabled) WHERE enabled = true;
CREATE INDEX idx_smtp_configs_default ON smtp_configs(is_default) WHERE is_default = true;
CREATE INDEX idx_alert_recipients_email ON alert_recipients(email);
CREATE INDEX idx_alert_recipients_enabled ON alert_recipients(enabled) WHERE enabled = true;

-- 创建更新时间触发器
CREATE TRIGGER update_smtp_configs_updated_at BEFORE UPDATE ON smtp_configs
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 确保只有一个默认 SMTP 配置
CREATE UNIQUE INDEX idx_smtp_configs_single_default ON smtp_configs(is_default) 
WHERE is_default = true;

-- 插入示例配置(密码需要在应用层加密)
INSERT INTO smtp_configs (
    name, server, port, username, password_encrypted, 
    from_address, from_name, use_tls, is_default
) VALUES (
    'Default SMTP',
    'smtp.gmail.com',
    587,
    'alerts@example.com',
    'ENCRYPTED_PASSWORD_PLACEHOLDER', -- 需要在应用中替换为真实加密密码
    'aiw3-monitor@example.com',
    'AIWS Monitor',
    true,
    true
) ON CONFLICT (name) DO NOTHING;

-- 插入示例接收人
INSERT INTO alert_recipients (email, name) VALUES
('admin@example.com', 'System Administrator'),
('ops@example.com', 'Operations Team')
ON CONFLICT DO NOTHING;

-- 添加注释
COMMENT ON TABLE smtp_configs IS 'SMTP 邮件服务器配置表';
COMMENT ON COLUMN smtp_configs.password_encrypted IS '密码使用 AES-256 加密存储';
COMMENT ON TABLE alert_recipients IS '告警通知接收人表';
COMMENT ON TABLE alert_rule_recipients IS '告警规则与接收人的多对多关联表';


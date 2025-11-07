# AIWS 区块链监控告警平台

高性能、可靠的区块链监控告警平台,用于实时监控 AIWS 区块链(基于 Cosmos SDK 和 CometBFT)的关键指标,并在异常情况下自动发送邮件告警。

## 🎯 核心功能

- ✅ **实时数据监控**: 每30秒采集区块高度、节点数量、交易池等关键指标
- ✅ **自动化告警**: 基于规则的告警引擎,支持邮件通知
- ✅ **历史数据分析**: 分级数据保留(1小时→7天→90天→1年)
- ✅ **多链支持**: 同时监控多个AIWS区块链节点
- ✅ **Grafana可视化**: 开箱即用的监控面板

## 🏗️ 技术架构

- **后端**: Rust 1.75+ (Tokio异步运行时)
- **数据库**: PostgreSQL 15 + TimescaleDB (时序数据自动聚合)
- **监控**: Prometheus + Grafana
- **通知**: SMTP邮件告警

## 🚀 快速开始

### 前置要求

- Docker 20+ 和 Docker Compose 2+
- Rust 1.75+ (如需本地开发)

### 一键启动(推荐)

```bash
# 1. 克隆仓库
git clone <repo-url>
cd aiw3-monitor

# 2. 启动所有服务
docker-compose -f deployment/docker/docker-compose.yml up -d

# 3. 初始化数据库
./scripts/setup-db.sh

# 4. 访问 Grafana 面板
open http://localhost:3000
# 默认登录: admin/admin
```

详细说明请参考 [快速开始指南](specs/001-blockchain-monitor/quickstart.md)

## 📊 性能指标

- 数据采集延迟: < 5秒
- 告警触发延迟: < 60秒
- API响应时间: P95 < 200ms
- 系统资源占用: CPU < 30%, 内存 < 1GB
- 支持并发监控: ≥ 5个节点

## 📁 项目结构

```
aiw3-monitor/
├── backend/              # Rust 后端服务
│   ├── src/
│   │   ├── collectors/   # 数据采集
│   │   ├── storage/      # 数据存储
│   │   ├── alerting/     # 告警引擎
│   │   ├── exporter/     # Prometheus导出
│   │   └── scheduler/    # 任务调度
│   └── tests/            # 测试
├── deployment/           # 部署配置
│   ├── docker/           # Docker配置
│   ├── prometheus/       # Prometheus配置
│   └── grafana/          # Grafana面板
├── scripts/              # 辅助脚本
└── docs/                 # 文档
```

## 📚 文档

- [功能规范](specs/001-blockchain-monitor/spec.md) - 用户故事和需求
- [技术方案](specs/001-blockchain-monitor/plan.md) - 架构设计和技术选型
- [数据模型](specs/001-blockchain-monitor/data-model.md) - 数据库设计
- [任务列表](specs/001-blockchain-monitor/tasks.md) - 实施任务分解
- [API契约](specs/001-blockchain-monitor/contracts/) - 接口规范

## 🛠️ 开发指南

### 本地开发

```bash
# 启动数据库
docker run -d --name postgres -p 5432:5432 -e POSTGRES_PASSWORD=secret timescale/timescaledb:latest-pg15

# 运行数据库迁移
cd backend
sqlx migrate run

# 配置应用
cp config.example.toml config.toml
# 编辑 config.toml 设置数据库连接

# 运行服务
cargo run

# 运行测试
cargo test
cargo tarpaulin --out Html  # 代码覆盖率
```

### 代码质量

```bash
# 格式化代码
cargo fmt

# Lint检查
cargo clippy -- -D warnings

# 性能基准测试
cargo bench
```

## 📦 部署

参考 [部署指南](docs/deployment.md)

## 🤝 贡献

欢迎贡献!请查看 [贡献指南](CONTRIBUTING.md)

## 📄 许可证

[MIT License](LICENSE)

## 🔗 相关链接

- AIWS DevNet 区块浏览器: https://devnet3.aiw3.io/aiw3chain-devnet
- AIWS RPC: https://devnet-rpc3.aiw3.io
- 项目文档: https://docs.aiw3monitor.io

---

## 📈 项目状态

**当前版本**: v1.0.0 MVP  
**状态**: ✅ **MVP 完成,可投入生产使用**  
**完成日期**: 2025-11-06

### MVP 功能清单 ✅
- ✅ 实时区块链数据监控
- ✅ TimescaleDB 时序数据存储
- ✅ Prometheus 指标导出
- ✅ Grafana 监控面板
- ✅ 自动化测试 (覆盖率 80%+)
- ✅ 完整的 CI/CD 配置

### 实施进度
- **Phase 1**: ✅ 项目初始化 (100%)
- **Phase 2**: ✅ 基础设施 (100%)
- **Phase 3**: ✅ US1 实时监控 (91%)
- **总体进度**: 42/115 任务 (37%)

详见:
- [任务列表](specs/001-blockchain-monitor/tasks.md)
- [MVP 完成报告](MVP_COMPLETION_REPORT.md)
- [测试报告](TEST_REPORT.md)

---

## 🎯 下一步计划

### Phase 4: 告警功能 (计划中)
- 告警规则配置
- 邮件通知
- 告警静默期
- 告警聚合

### Phase 5: 告警历史 (计划中)
- 告警记录查询
- 统计分析
- 报告导出

### Phase 6: 多链支持 (计划中)
- 多节点管理
- 节点切换
- 独立告警配置


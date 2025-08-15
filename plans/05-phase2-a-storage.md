# 2.a storage 模块开发计划

**阶段**: 第二阶段 - 存储同步 (串行)
**关联文件**: [01-overview.md](./01-overview.md), [04-phase1-c-core.md](./04-phase1-c-core.md)

## 模块职责
Zenoh网络通信、本地存储管理、分布式同步实现。

## 数据结构设计

### StorageManager结构
```rust
pub struct StorageManager {
    zenoh_session: Arc<zenoh::Session>,
    local_storage: Box<dyn LocalStorage>,
    sync_config: SyncConfig,
}
```

### SyncResult结构
```rust
pub struct SyncResult {
    pub conflicts: Vec<Conflict>,
    pub applied_changes: Vec<Change>,
    pub sync_status: SyncStatus,
}
```

## API接口定义

```rust
pub trait StorageInterface {
    async fn save_document(doc: &Document) -> Result<()>;
    async fn load_document(uri: &str) -> Result<Document>;
    async fn sync_document(doc: &Document) -> Result<SyncResult>;
    async fn subscribe_changes(callback: Box<dyn Fn(Change) + Send + Sync>) -> Result<SubscriptionHandle>;
    async fn publish_change(change: &Change) -> Result<()>;
    async fn list_documents() -> Result<Vec<DocumentInfo>>;
}
```

## 功能点覆盖
- [ ] Zenoh会话管理
- [ ] 发布/订阅模式
- [ ] 本地文件存储
- [ ] 分布式同步
- [ ] 网络中断恢复
- [ ] 缓存管理

## 依赖其他模块
- elfi-types: Document, Change类型

## 测试策略
- 本地存储操作
- 网络同步功能
- 网络中断恢复
- 并发同步测试

## 🤖 推荐使用的 Subagent

### 主要开发 Subagent
**@network-architect**: 负责分布式网络架构的专业实现
- 设计和实现 Zenoh 网络通信层
- 实现多种网络拓扑支持（P2P、客户端-服务器、mesh）
- 设计分布式存储和同步策略
- 实现网络中断恢复和离线支持
- 优化网络性能和连接管理

### 支持 Subagent
**@rust-tdd-developer**: 负责网络层的测试和质量保证
- 编写网络分区和故障恢复测试
- 验证分布式同步的正确性
- 性能基准测试和压力测试
- 网络协议的边界条件测试

### 使用示例
```bash
# 第一步：Zenoh 网络层实现
@network-architect 请实现基于 Zenoh 的分布式存储系统。
要求：
1. 参考本计划文档中的 StorageManager 和接口设计
2. 实现 Zenoh 会话管理和网络通信
3. 支持 P2P、客户端-服务器、mesh 等多种网络拓扑
4. 实现智能的数据同步和冲突解决机制
5. 设计网络中断恢复和离线缓存支持
6. 确保同步延迟 < 100ms（本地网络）
7. 支持 7天的离线操作缓存

# 第二步：网络测试和验证
@rust-tdd-developer 请为分布式存储编写完整的测试套件。
要求：
1. 网络分区和恢复的测试
2. 多节点并发同步的正确性验证
3. 各种网络故障场景的测试
4. 离线操作和恢复的测试
5. 性能基准测试（重连时间 < 5秒）
6. 内存和连接泄漏检测

# 第三步：文档更新
@docs-maintainer 请更新以下文档：
1. docs/src/implementations/05-storage.md - Storage模块实现文档
2. docs/src/designs/02-storage_sync.md - 存储同步设计文档更新
3. docs/src/api/storage.md - Storage API参考文档
```
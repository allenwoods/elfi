# 阶段3: Core 模块开发计划

**阶段**: 阶段3 - 核心引擎 (串行)  
**关联文件**: [01-overview.md](./01-overview.md), [phase1-types.md](./phase1-types.md), [phase2-parser.md](./phase2-parser.md), [phase4-storage.md](./phase4-storage.md)

## 🤖 推荐 Subagent

**主要**: `@crdt-specialist` - 专门负责CRDT实现和事件溯源设计  
**辅助**: `@rust-tdd-developer` - 负责并发测试和性能测试

### 调用示例
```bash
@crdt-specialist 请开发 core 模块，实现基于Automerge的CRDT文档管理。
参考 docs/src/designs/01-data_modeling.md 中的CRDT设计，
实现文档生命周期管理、事件溯源和冲突解决机制。
确保与 types 模块的数据结构兼容。
```

## 模块职责
CRDT数据管理、文档生命周期管理、事件溯源实现。

## 数据结构设计

### DocumentManager结构
```rust
pub struct DocumentManager {
    documents: DashMap<String, AutomergeDoc>,
    sessions: DashMap<String, SessionHandle>,
    event_log: Arc<Mutex<EventLog>>,
}
```

### Main统一接口
```rust
pub struct Main {
    document_manager: Arc<DocumentManager>,
    session_manager: Arc<SessionManager>,
    storage: Box<dyn StorageInterface>,
}
```

## API接口定义

```rust
pub trait CoreInterface {
    async fn open_document(uri: &str) -> Result<DocumentHandle>;
    async fn create_document(config: CreateConfig) -> Result<DocumentHandle>;
    async fn add_block(doc_uri: &str, block_type: BlockType, name: Option<String>) -> Result<String>;
    async fn delete_block(doc_uri: &str, block_id: &str) -> Result<()>;
    async fn update_block(doc_uri: &str, block_id: &str, content: &str) -> Result<()>;
    async fn get_history(doc_uri: &str) -> Result<HistoryGraph>;
    async fn sync_document(doc_uri: &str) -> Result<SyncResult>;
}
```

## 功能点覆盖
- [ ] CRDT文档管理
- [ ] 事件溯源操作日志
- [ ] 文档生命周期管理
- [ ] 块级CRUD操作
- [ ] 历史版本管理
- [ ] 冲突检测和解决

## 依赖其他模块
- elfi-types: 所有数据类型
- elfi-storage: StorageInterface
- elfi-parser: ParserInterface

## 测试策略
- CRDT操作正确性
- 并发操作安全性
- 事件溯源完整性
- 性能基准测试

## 🤖 推荐使用的 Subagent

### 主要开发 Subagent
**@crdt-specialist**: 负责 CRDT 和事件溯源的专业实现
- 设计基于 Automerge 的 CRDT 文档管理系统
- 实现事件溯源和不可变操作日志
- 设计智能的冲突检测和解决策略
- 实现时间旅行和历史版本管理
- 优化 CRDT 操作的性能

### 支持 Subagent
**@rust-tdd-developer**: 负责测试驱动开发和质量保证
- 编写 CRDT 操作的并发测试
- 验证事件溯源的正确性
- 测试冲突解决的各种场景
- 性能基准测试和压力测试

### 使用示例
```bash
# 第一步：CRDT 系统设计和实现
@crdt-specialist 请实现 ELFI 的 CRDT 文档管理系统。
要求：
1. 参考本计划文档中的 DocumentManager 和 Main 接口设计
2. 基于 Automerge 实现 CRDT 数据结构
3. 实现完整的事件溯源系统，支持操作重放
4. 设计可插拔的冲突解决策略
5. 实现时间旅行功能，支持历史版本查看
6. 确保支持 10+ 并发用户同时编辑
7. 单文档内存使用 < 100MB

# 第二步：并发测试和验证
@rust-tdd-developer 请为 CRDT 系统编写完整的测试套件。
要求：
1. 多用户并发编辑的正确性测试
2. 各种冲突场景的解决验证
3. 事件溯源的完整性测试
4. 网络分区情况下的一致性测试
5. 性能基准测试（文档同步延迟 < 100ms）
6. 内存使用和泄漏检测

# 第三步：文档更新
@docs-maintainer 请更新以下文档：
1. docs/src/implementations/04-core.md - Core模块实现文档
2. docs/src/designs/01-data_modeling.md - CRDT设计文档更新
3. docs/src/api/core.md - Core API参考文档
```
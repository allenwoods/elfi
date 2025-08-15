# 阶段5a: Weave 模块开发计划

**阶段**: 阶段5 - 应用层 (并行)
**关联文件**: [01-overview.md](./01-overview.md), [phase3-core.md](./phase3-core.md), [phase4-storage.md](./phase4-storage.md), [phase5-tangle.md](./phase5-tangle.md)

## 模块职责
内容创作API、关系管理、IDE集成功能。

## 数据结构设计

### WeaveAPI结构
```rust
pub struct WeaveAPI {
    core: Arc<dyn CoreInterface>,
    file_watcher: Option<FileWatcher>,
    relation_manager: RelationManager,
}
```

### FileWatcher结构
```rust
pub struct FileWatcher {
    watcher: RecommendedWatcher,
    sync_config: WatchConfig,
    callback: Box<dyn Fn(FileEvent)>,
}
```

## API接口定义

```rust
pub trait WeaveInterface {
    async fn create_block(doc_uri: &str, block_type: &str, content: &str) -> Result<String>;
    async fn update_block_content(doc_uri: &str, block_id: &str, content: &str) -> Result<()>;
    async fn link_blocks(from_uri: &str, to_uri: &str, relation_type: &str) -> Result<()>;
    async fn unlink_blocks(from_uri: &str, to_uri: &str) -> Result<()>;
    async fn get_relations(doc_uri: &str) -> Result<Vec<Relation>>;
    async fn watch_files(config: WatchConfig) -> Result<WatchHandle>;
    async fn sync_with_files(doc_uri: &str, file_path: &str) -> Result<()>;
}
```

## 功能点覆盖
- [ ] 块内容创作和编辑
- [ ] 块间关系管理
- [ ] 文件监听和同步
- [ ] IDE双向集成
- [ ] 关系图查询
- [ ] 内容搜索功能

## 依赖其他模块
- elfi-core: CoreInterface
- elfi-types: 所有数据类型

## 测试策略
- 块创作和编辑功能
- 关系管理操作
- 文件监听功能
- IDE集成测试
# 阶段1: Types 模块开发计划

**阶段**: 阶段1 - 数据基础 (串行)  
**关联文件**: [01-overview.md](./01-overview.md), [phase2-parser.md](./phase2-parser.md), [phase2-core.md](./phase2-core.md)

## 🤖 推荐 Subagent

**主要**: `@rust-tdd-developer` - 负责数据结构设计和TDD开发流程  
**辅助**: `@docs-maintainer` - 负责API文档和使用示例生成

### 调用示例
```bash
@rust-tdd-developer 请开发 types 模块，定义ELFI的核心数据结构。
参考本文件中的数据结构设计要求，遵循TDD流程：
1. 先在 types/src/interface.rs 中定义公共接口
2. 在 types/tests/ 中编写单元测试
3. 实现 Document、Block、Relation 等核心类型
```

## 模块职责
核心数据结构定义，被所有其他模块共享使用。

## 数据结构设计

### Document结构
```rust
pub struct Document {
    pub id: String,
    pub blocks: Vec<Block>,
    pub metadata: DocumentMetadata,
}
```

### Block结构
```rust
pub struct Block {
    pub id: String,
    pub name: Option<String>,
    pub block_type: String,
    pub attributes: HashMap<String, Value>,
    pub content: BlockContent,
}
```

### Relation结构
```rust
pub struct Relation {
    pub from: String,
    pub to: String,
    pub relation_type: String,
    pub attributes: HashMap<String, Value>,
}
```

## API接口定义

```rust
pub trait TypeInterface {
    fn validate_block(block: &Block) -> Result<()>;
    fn serialize_document(doc: &Document) -> Result<String>;
    fn deserialize_document(content: &str) -> Result<Document>;
}
```

## 功能点覆盖
- [ ] 基础数据结构定义
- [ ] 序列化/反序列化
- [ ] 类型验证
- [ ] 错误类型定义

## 依赖其他模块
无（基础模块）

## 测试策略
- 数据结构创建和访问
- 序列化往返测试
- 边界条件验证

## 🤖 推荐使用的 Subagent

### 主要开发 Subagent
**@rust-tdd-developer**: 负责 TDD 开发流程和代码质量保证
- 先编写完整的测试套件
- 实现核心数据结构
- 确保测试覆盖率 > 80%
- 验证序列化/反序列化功能

### 支持 Subagent
**@docs-maintainer**: 负责文档同步和API文档生成
- 更新 `docs/src/implementations/02-types.md`
- 生成API参考文档
- 维护类型定义的文档说明

### 使用示例
```bash
# 第一步：TDD开发
@rust-tdd-developer 请按照TDD流程开发 elfi-types 模块。
要求：
1. 参考本计划文档中的数据结构设计
2. 先编写完整的测试套件覆盖所有功能点
3. 实现 Document、Block、Relation 核心类型
4. 确保序列化/反序列化正常工作
5. 测试覆盖率 > 80%

# 第二步：文档更新
@docs-maintainer 请更新以下文档：
1. docs/src/implementations/02-types.md - 实现文档
2. docs/src/api/types.md - API参考文档
3. 类型定义的使用示例和最佳实践
```
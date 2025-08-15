# types 模块文档

## 模块职责
参考: [plans/04-phase1-a-types.md](../../plans/04-phase1-a-types.md)

核心数据结构定义，被所有其他模块共享使用。

## API文档
[开发者文档区域 - 请更新]

### 主要类型

#### Document
- 文档的主要结构
- 包含块列表和元数据

#### Block  
- 4字段设计：id, name, type, attributes, content
- 支持不同内容类型

#### Relation
- 块间关系定义
- 支持跨文档引用

#### TypesError
- 统一的错误类型
- 包含验证、序列化等错误

## 使用示例
[开发者文档区域 - 请更新]

```rust
use types::{Document, Block, BlockContent};

// 创建文档
let doc = Document::new("my-doc".to_string());

// 创建块
let mut block = Block::new("block-1".to_string(), "markdown".to_string());
block.content = BlockContent::Text("Hello World".to_string());
```

## 测试覆盖
[开发者文档区域 - 请更新]

- [ ] Document创建和查找
- [ ] Block创建和验证
- [ ] Relation创建和验证
- [ ] 错误处理
- [ ] 序列化/反序列化

## 依赖管理

### 添加新依赖
```bash
# 进入types模块目录
cd types

# 添加运行时依赖
cargo add <package-name>

# 添加特定版本
cargo add <package-name>@<version>

# 添加带features的依赖
cargo add <package-name> --features feature1,feature2

# 添加开发依赖
cargo add --dev <package-name>
```

### 当前依赖
- serde: 序列化支持
- serde_json: JSON处理
- thiserror: 错误处理

### 更新依赖
```bash
# 更新单个依赖
cargo update -p <package-name>

# 更新所有依赖
cargo update
```

### 移除依赖
```bash
cargo rm <package-name>
```
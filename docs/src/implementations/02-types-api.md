# Types模块：API参考文档

本文档提供ELFI Types模块的完整API参考，包括所有公共接口、数据结构和使用示例。

## API概览

Types模块提供以下核心API：

| 类型 | 描述 | 主要功能 |
|------|------|----------|
| [`Document`](#document) | 文档容器 | 文档管理、块集合操作 |
| [`Block`](#block) | 内容块 | 块创建、内容管理、验证 |
| [`Relation`](#relation) | 关系对象 | 关系定义、属性管理 |
| [`TypeInterface`](#typeinterface) | 类型接口 | 可扩展的类型处理抽象 |
| [`TypesError`](#typeserror) | 错误类型 | 统一的错误处理 |

## Document

文档是ELFI系统中的顶级容器，管理一组相关的Block。

### 数据结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,                    // 文档唯一标识符
    pub blocks: Vec<Block>,           // 文档包含的所有块
    pub metadata: DocumentMetadata,   // 文档元数据
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub created_at: String,           // 创建时间 (RFC3339格式)
    pub updated_at: String,           // 更新时间 (RFC3339格式)
    pub version: u64,                 // 版本号
    pub attributes: HashMap<String, serde_json::Value>, // 自定义属性
}
```

### 构造函数

#### `Document::new(id: String) -> Self`

创建新的空文档。

**参数：**
- `id`: 文档的唯一标识符（建议使用UUID）

**返回：**
- 新创建的Document实例

**示例：**
```rust
use uuid::Uuid;
use elfi_types::Document;

let doc_id = Uuid::new_v4().to_string();
let document = Document::new(doc_id);

assert_eq!(document.blocks.len(), 0);
assert_eq!(document.metadata.version, 1);
```

### 块管理方法

#### `add_block(&mut self, block: Block) -> Result<(), TypesError>`

向文档添加新块。如果块名称已存在，返回错误。

**参数：**
- `block`: 要添加的Block实例

**返回：**
- `Ok(())`: 成功添加
- `Err(TypesError)`: 块名称重复或其他验证错误

**示例：**
```rust
use elfi_types::*;
use uuid::Uuid;

let mut doc = Document::new(Uuid::new_v4().to_string());

let block = Block::new(
    Uuid::new_v4().to_string(),
    "text".to_string()
).with_name("intro".to_string());

doc.add_block(block)?;
assert_eq!(doc.blocks.len(), 1);
```

#### `find_block(&self, block_id: &str) -> Option<&Block>`

根据ID查找块。

**参数：**
- `block_id`: 要查找的块ID

**返回：**
- `Some(&Block)`: 找到的块引用
- `None`: 未找到

#### `find_block_by_name(&self, name: &str) -> Option<&Block>`

根据名称查找块。

**参数：**
- `name`: 要查找的块名称

**返回：**
- `Some(&Block)`: 找到的块引用
- `None`: 未找到

#### `remove_block(&mut self, block_id: &str) -> bool`

移除指定ID的块。

**参数：**
- `block_id`: 要移除的块ID

**返回：**
- `true`: 成功移除
- `false`: 块不存在

### 验证方法

#### `validate(&self) -> Result<(), Vec<TypesError>>`

验证整个文档的有效性。

**返回：**
- `Ok(())`: 文档有效
- `Err(Vec<TypesError>)`: 包含所有验证错误的列表

**验证规则：**
- 文档ID不能为空
- 块名称不能重复
- 所有块必须通过验证

## Block

Block是ELFI文档的基本内容单元，使用4字段设计。

### 数据结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: String,                   // 块唯一标识符
    pub name: Option<String>,         // 可选的人类可读名称
    pub block_type: String,          // 块类型标识
    pub attributes: HashMap<String, serde_json::Value>, // 块属性
    pub content: BlockContent,       // 块内容
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockContent {
    Text(String),       // 文本内容
    Relations(String),  // 关系定义语法
    Binary(Vec<u8>),   // 二进制数据
}
```

### 构造函数

#### `Block::new(id: String, block_type: String) -> Self`

创建新块。

**参数：**
- `id`: 块的唯一标识符（必须是有效的UUID格式）
- `block_type`: 块类型（如："text", "code", "relations"等）

**返回：**
- 新创建的Block实例

**示例：**
```rust
use elfi_types::*;
use uuid::Uuid;

let block = Block::new(
    Uuid::new_v4().to_string(),
    "markdown".to_string()
);

assert!(block.name.is_none());
assert_eq!(block.block_type, "markdown");
```

### 建造者模式方法

#### `with_name(self, name: String) -> Self`

设置块名称。

**参数：**
- `name`: 块的名称

**返回：**
- 修改后的Block实例

#### `with_content(self, content: BlockContent) -> Self`

设置块内容。

**参数：**
- `content`: BlockContent枚举值

**返回：**
- 修改后的Block实例

#### `with_attributes(self, attributes: HashMap<String, serde_json::Value>) -> Self`

设置块属性。

**参数：**
- `attributes`: 属性映射

**返回：**
- 修改后的Block实例

#### `with_attribute(self, key: String, value: serde_json::Value) -> Self`

设置单个属性。

**参数：**
- `key`: 属性键
- `value`: 属性值

**返回：**
- 修改后的Block实例

**示例：**
```rust
use elfi_types::*;
use uuid::Uuid;

let block = Block::new(
    Uuid::new_v4().to_string(),
    "code".to_string()
)
.with_name("hello_world".to_string())
.with_content(BlockContent::Text("println!(\"Hello, World!\");".to_string()))
.with_attribute("language".to_string(), 
    serde_json::Value::String("rust".to_string()));

assert_eq!(block.name, Some("hello_world".to_string()));
```

### 验证方法

#### `validate(&self) -> Result<(), TypesError>`

验证块的有效性。

**返回：**
- `Ok(())`: 块有效
- `Err(TypesError)`: 验证错误

**验证规则：**
- ID不能为空且必须是有效的UUID格式
- 块类型不能为空
- 如果有名称，名称不能为空

## Relation

Relation表示块之间或块与外部实体之间的关系。

### 数据结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub from: String,                 // 源块ID或URI
    pub to: String,                   // 目标块ID或URI
    pub relation_type: String,        // 关系类型
    pub attributes: HashMap<String, serde_json::Value>, // 关系属性
}
```

### 构造函数

#### `Relation::new(from: String, to: String, relation_type: String) -> Self`

创建新关系。

**参数：**
- `from`: 源块ID或URI
- `to`: 目标块ID或URI
- `relation_type`: 关系类型（如："child_of", "references", "follows"等）

**返回：**
- 新创建的Relation实例

**示例：**
```rust
use elfi_types::Relation;

let relation = Relation::new(
    "block-1".to_string(),
    "block-2".to_string(),
    "child_of".to_string()
);

assert_eq!(relation.from, "block-1");
assert_eq!(relation.relation_type, "child_of");
```

### 建造者模式方法

#### `with_attributes(self, attributes: HashMap<String, serde_json::Value>) -> Self`

设置关系属性。

**参数：**
- `attributes`: 属性映射

**返回：**
- 修改后的Relation实例

#### `with_attribute(self, key: String, value: serde_json::Value) -> Self`

设置单个属性。

**参数：**
- `key`: 属性键
- `value`: 属性值

**返回：**
- 修改后的Relation实例

**示例：**
```rust
use elfi_types::Relation;

let relation = Relation::new(
    "task-1".to_string(),
    "task-2".to_string(),
    "depends_on".to_string()
)
.with_attribute("priority".to_string(), 
    serde_json::Value::String("high".to_string()))
.with_attribute("weight".to_string(), 
    serde_json::Value::Number(serde_json::Number::from_f64(0.8).unwrap()));
```

### 验证方法

#### `validate(&self) -> Result<(), TypesError>`

验证关系的有效性。

**返回：**
- `Ok(())`: 关系有效
- `Err(TypesError)`: 验证错误

**验证规则：**
- from字段不能为空
- to字段不能为空
- relation_type不能为空

## TypeInterface

TypeInterface是一个trait，定义了类型处理的标准接口，支持插件化扩展。

### Trait定义

```rust
pub trait TypeInterface: Send + Sync {
    fn validate_block(&self, block: &Block) -> Result<(), TypesError>;
    fn serialize_document(&self, doc: &Document) -> Result<String, TypesError>;
    fn deserialize_document(&self, content: &str) -> Result<Document, TypesError>;
    fn validate_relation(&self, relation: &Relation) -> Result<(), TypesError>;
}
```

### 方法说明

#### `validate_block(&self, block: &Block) -> Result<(), TypesError>`

验证指定的块。

#### `serialize_document(&self, doc: &Document) -> Result<String, TypesError>`

将文档序列化为字符串。

#### `deserialize_document(&self, content: &str) -> Result<Document, TypesError>`

从字符串反序列化文档。

#### `validate_relation(&self, relation: &Relation) -> Result<(), TypesError>`

验证指定的关系。

### 默认实现

#### `DefaultTypeInterface`

提供TypeInterface的默认实现。

**示例：**
```rust
use elfi_types::*;

let interface = DefaultTypeInterface::new();

// 验证块
let block = Block::new("test-id".to_string(), "text".to_string());
interface.validate_block(&block)?;

// 序列化文档
let doc = Document::new("doc-id".to_string());
let json = interface.serialize_document(&doc)?;

// 反序列化文档
let restored_doc = interface.deserialize_document(&json)?;
```

## TypesError

统一的错误类型，用于表示Types模块中的各种错误。

### 错误类型

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TypesError {
    EmptyField { field: String },
    InvalidUuid { value: String },
    DuplicateName { name: String },
    BlockValidation { message: String },
    RelationValidation { message: String },
    DocumentValidation { errors: Vec<String> },
    SerializationError { message: String },
    ValidationError { message: String },
}
```

### 错误说明

- **EmptyField**: 必需字段为空
- **InvalidUuid**: UUID格式无效
- **DuplicateName**: 名称重复
- **BlockValidation**: 块验证错误
- **RelationValidation**: 关系验证错误
- **DocumentValidation**: 文档验证错误
- **SerializationError**: 序列化/反序列化错误
- **ValidationError**: 通用验证错误

### 错误处理示例

```rust
use elfi_types::*;

fn handle_error(result: Result<(), TypesError>) {
    match result {
        Ok(()) => println!("操作成功"),
        Err(TypesError::EmptyField { field }) => {
            eprintln!("字段 '{}' 不能为空", field);
        }
        Err(TypesError::InvalidUuid { value }) => {
            eprintln!("无效的UUID: {}", value);
        }
        Err(TypesError::DuplicateName { name }) => {
            eprintln!("名称 '{}' 已存在", name);
        }
        Err(e) => {
            eprintln!("其他错误: {:?}", e);
        }
    }
}
```

## 完整使用示例

### 创建复杂文档

```rust
use elfi_types::*;
use uuid::Uuid;
use std::collections::HashMap;

fn create_technical_document() -> Result<Document, TypesError> {
    // 创建文档
    let mut doc = Document::new(Uuid::new_v4().to_string());
    
    // 设置文档属性
    doc.metadata.attributes.insert(
        "title".to_string(),
        serde_json::Value::String("技术设计文档".to_string())
    );
    
    // 添加介绍块
    let intro_block = Block::new(
        Uuid::new_v4().to_string(),
        "markdown".to_string()
    )
    .with_name("introduction".to_string())
    .with_content(BlockContent::Text(
        "# 项目介绍\n\n这是一个技术设计文档。".to_string()
    ))
    .with_attribute("level".to_string(), 
        serde_json::Value::Number(serde_json::Number::from(1)));
    
    doc.add_block(intro_block.clone())?;
    
    // 添加代码块
    let code_block = Block::new(
        Uuid::new_v4().to_string(),
        "code".to_string()
    )
    .with_name("example_code".to_string())
    .with_content(BlockContent::Text(
        "fn main() {\n    println!(\"Hello, ELFI!\");\n}".to_string()
    ))
    .with_attribute("language".to_string(), 
        serde_json::Value::String("rust".to_string()));
    
    doc.add_block(code_block.clone())?;
    
    // 添加关系块
    let relations_content = format!(r#"
child_of {}
references elf://external/rust-docs#getting-started
"#, intro_block.id);
    
    let relations_block = Block::new(
        Uuid::new_v4().to_string(),
        "relations".to_string()
    )
    .with_name("relationships".to_string())
    .with_content(BlockContent::Relations(relations_content));
    
    doc.add_block(relations_block)?;
    
    // 验证文档
    doc.validate()?;
    
    Ok(doc)
}

fn main() -> Result<(), TypesError> {
    let document = create_technical_document()?;
    
    // 使用接口进行序列化
    let interface = DefaultTypeInterface::new();
    let json = interface.serialize_document(&document)?;
    
    println!("Generated JSON: {}", json);
    
    // 验证反序列化
    let restored = interface.deserialize_document(&json)?;
    assert_eq!(document.id, restored.id);
    
    println!("文档创建和验证成功！");
    Ok(())
}
```

## 最佳实践

### 1. 错误处理

总是检查验证结果：

```rust
// 好的做法
let block = Block::new(id, block_type);
block.validate()?; // 立即验证

// 避免的做法
let block = Block::new(id, block_type);
// 没有验证就直接使用
```

### 2. 建造者模式

使用链式调用创建复杂对象：

```rust
let block = Block::new(id, "text".to_string())
    .with_name("section1".to_string())
    .with_content(BlockContent::Text("内容".to_string()))
    .with_attribute("priority".to_string(), 
        serde_json::Value::String("high".to_string()));
```

### 3. 性能考虑

对于大型文档，考虑使用引用而不是克隆：

```rust
// 高效的查找
if let Some(block) = document.find_block(&block_id) {
    // 使用 &Block，避免克隆
    process_block(block);
}
```

### 4. 并发安全

Types模块的所有类型都实现了`Send + Sync`，可以安全地在多线程环境中使用：

```rust
use std::sync::Arc;
use std::thread;

let document = Arc::new(create_document()?);
let document_clone = Arc::clone(&document);

thread::spawn(move || {
    // 安全地在另一个线程中使用文档
    let block_count = document_clone.blocks.len();
    println!("Block count: {}", block_count);
});
```

---

**API文档版本**: elfi-types v0.1.0  
**生成时间**: 2025-01-15  
**文档状态**: 完整 - 反映实际实现  

更多详细信息请参考生成的Rust文档：`target/doc/types/index.html`
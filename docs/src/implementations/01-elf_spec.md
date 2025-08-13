# .elf 文件语法规范

本文档定义了 `.elf` 文件格式的完整语法规范和解析策略。`.elf` 格式是一种人类可读、版本控制友好且易于解析的结构化文档格式。

## 1. 语法设计原则

### 1.1. 核心理念

`.elf` 文件本质上是**分块的结构化文档**，每个块都有自己的元数据和内容：

- **显式分块**：通过 `---` 分隔符明确划分内容块
- **元数据驱动**：每个块都有 YAML 前置元数据  
- **类型化内容**：通过 `type` 字段显式声明块的内容类型
- **结构化关系**：通过 `parent` 等元数据建立块之间的层级关系
- **版本控制友好**：纯文本格式，便于 Git 等工具进行差异比较

### 1.2. 与 Markdown 的关系

`.elf` 格式在基础文本处理上与 Markdown 高度相似，但在以下方面有重要扩展：

- **结构化元数据**：每个内容块都有明确的类型和属性
- **跨文档引用**：支持引用其他 `.elf` 文档中的内容
- **内容转换**：通过 Recipe 系统支持多种输出格式
- **协作增强**：内置对并发编辑和版本历史的支持

## 2. 文档结构规范

### 2.1. 文件级结构

一个 `.elf` 文件由一系列**块 (Blocks)** 组成，块之间由标准 `---` 分隔符隔开：

```elf
---
id: f47ac10b-58cc-4372-a567-0e02b2c3d479
type: markdown
name: introduction
---
# 项目介绍

这是一个基于CRDT的协作文档系统。

---
id: a1b2c3d4-5e6f-7890-1234-56789abcdef0
type: code
name: setup-code
metadata:
  parent: f47ac10b-58cc-4372-a567-0e02b2c3d479
  language: bash
---
npm install elfi-core

---
id: 2e8f9a3b-1c4d-5a6b-7c8d-9e0f1a2b3c4d
type: link
name: shared-utils
metadata:
  parent: f47ac10b-58cc-4372-a567-0e02b2c3d479
---
target: elf://shared-lib/utils/helpers#string-utils
ref_type: include
display_text: 共享工具函数
```

### 2.2. 块结构组成

每个**块**有两个不同的部分：

1. **元数据部分** (YAML Frontmatter)：由 `---` 包围的 YAML 配置
2. **内容部分**：元数据结束后的所有文本内容

### 2.3. 块分隔符规则

- **分隔符**：`---`（三个连字符）
- **位置**：独占一行，前后可有空白字符
- **作用**：分隔相邻的块，同时标识元数据部分的开始和结束

## 3. 元数据部分规范

### 3.1. 必需字段

#### `id` (字符串)
```yaml
id: f47ac10b-58cc-4372-a567-0e02b2c3d479
```
- **作用**：块的全局唯一标识符（系统内部的真实身份）
- **格式**：**必须使用标准 UUID 格式**
- **约束**：全局唯一，系统自动生成
- **用途**：CRDT 操作的目标标识、跨文档引用、历史追踪

#### `type` (字符串)
```yaml
type: markdown  # 或 code, link, recipe 等
```
- **作用**：声明块的内容类型，决定解析和渲染方式
- **标准类型**：
  - `markdown`: 标准 Markdown 文本内容
  - `code`: 程序代码（需要 `language` 元数据）
  - `link`: 跨文档引用（使用特殊内容格式）
  - `recipe`: 内容转换配方（YAML 配置）
  - `metadata`: 文档级元数据（通常是第一个块）
- **扩展性**：支持自定义类型，由插件系统处理

### 3.2. 可选字段

#### `name` (字符串)
```yaml
name: helper-functions
```
- **作用**：人类可读的块名称（便于记忆的别名）
- **约束**：同一文档内必须唯一（如果指定）
- **用途**：跨文档引用目标、内部导航、Recipe 选择器
- **格式**：推荐使用 `kebab-case` 格式（如 `intro-section`, `utils-block`）

#### `metadata` (对象)
```yaml
metadata:
  parent: parent-block-id
  language: python
  interactive: true
  tags: ["export", "utility"]
  description: "工具函数集合"
  owner: "alice"
  merge_method: "manual"
```

**标准元数据属性**：
- `parent`: 父块 ID，用于构建层级结构
- `language`: 代码语言（适用于 code 类型）
- `interactive`: 是否需要交互式渲染（Tangle 层使用）
- `tags`: 标签数组，用于分类和 Recipe 选择
- `description`: 块的描述信息
- `owner`: 区块所有者（协作权限控制）
- `merge_method`: 合并策略 (`crdt` | `manual`)

## 4. 内容部分规范

### 4.1. 通用内容规则

内容部分包含块的原始文本，紧跟在元数据部分的结束 `---` 之后，延伸到下一个块分隔符或文件末尾。

### 4.2. 特殊块类型的内容格式

#### Link Block 语法

Link Block 使用特殊的键值对语法：

```elf
---
id: ref-utils
type: link
name: reference-to-utils
---
target: elf://my-project/components/shared-utils#helper-functions
ref_type: include
display_text: 共享工具函数
description: 项目中的通用辅助函数库
```

**Link Block 内容字段**：
- `target`: 目标 URI（必需）
- `ref_type`: 引用类型 - `include` | `reference` | `embed`
- `display_text`: 显示文本（可选）
- `description`: 引用描述（可选）

#### Recipe Block 语法

Recipe Block 使用 YAML 配置：

```elf
---
id: export-recipe
type: recipe
name: project-documentation
---
name: "project-export"
version: "1.0"
description: "导出项目文档"

references:
  - source: "elf://shared-lib/templates#doc-template"
    target: "doc_template"
    cache_policy: "on_change"

selector:
  types: ["markdown", "code"]
  tags: ["export"]
  names: ["introduction", "setup-*"]

transform:
  - type: "filter"
    action: "include"
    template: "# {{name}}\n\n{{content}}\n"
  - type: "concat"
    action: "merge"
    template: "{{content}}\n---\n"

error_handling:
  on_missing_reference: "placeholder"
  on_circular_reference: "error"
  
output:
  format: "markdown"
  filename: "{{project_name}}-docs.md"
  header: "# {{project_name}} 文档\n\n"
  footer: "\n\n生成时间：{{timestamp}}"
```

## 5. URI 引用格式

### 5.1. 标准 ELF URI 格式

```
elf://[user/]repo/doc[#block-name]
```

**组成部分**：
- `user/`: 用户或组织名（可选，默认当前用户）
- `repo`: 仓库名（必需）
- `doc`: 文档名（必需，不含 .elf 扩展名）
- `#block-name`: 块名称（可选，默认引用整个文档）

**示例**：
```
elf://my-project/main                    # 引用整个文档
elf://my-project/components#utils        # 引用特定块
elf://alice/shared-lib/helpers#strings   # 跨用户引用
```

### 5.2. 相对引用格式

```
./doc#block-name          # 同仓库相对引用
#block-name               # 同文档内引用
../other-repo/doc         # 跨仓库相对引用（同用户）
```

## 6. 解析策略和实现

### 6.1. Tree-sitter 语法定义

使用 Tree-sitter 创建结构化解析器：

```javascript
// grammar.js (简化版)
module.exports = grammar({
  name: 'elf',
  
  rules: {
    source_file: $ => repeat($.block),
    
    block: $ => seq(
      $.metadata_section,
      optional($.content_section)
    ),
    
    metadata_section: $ => seq(
      '---',
      $.yaml_content,
      '---'
    ),
    
    content_section: $ => repeat1($._content_line),
    
    yaml_content: $ => /[^-]([^-]|-[^-]|--[^-])*/,
    
    _content_line: $ => /.*/
  }
});
```

### 6.2. 解析器实现策略

```rust
// core/src/parser/mod.rs
pub struct ElfParser {
    tree_sitter_parser: tree_sitter::Parser,
}

impl ElfParser {
    pub fn new() -> Result<Self, ParseError> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_elf::language())?;
        Ok(Self { tree_sitter_parser: parser })
    }
    
    /// 将 .elf 文本解析为 CRDT 文档
    pub fn parse_to_doc(&mut self, text: &str) -> Result<automerge::AutoCommit, ParseError> {
        // 1. Tree-sitter 解析为 CST
        let tree = self.tree_sitter_parser.parse(text, None)
            .ok_or(ParseError::TreeSitterFailed)?;
        
        // 2. 遍历 CST 提取块结构
        let root_node = tree.root_node();
        let mut doc = automerge::AutoCommit::new();
        
        // 3. 初始化文档结构
        let blocks_id = doc.put_object(automerge::ROOT, "blocks", automerge::ObjType::List)?;
        
        // 4. 处理每个块
        for block_node in root_node.children(&mut tree.walk()) {
            if block_node.kind() == "block" {
                let block = self.parse_block(text, &block_node)?;
                self.add_block_to_doc(&mut doc, &blocks_id, block)?;
            }
        }
        
        Ok(doc)
    }
    
    fn parse_block(&self, source: &str, node: &tree_sitter::Node) -> Result<ParsedBlock, ParseError> {
        let mut metadata_text = None;
        let mut content_text = None;
        
        for child in node.children(&mut node.walk()) {
            match child.kind() {
                "metadata_section" => {
                    metadata_text = Some(child.utf8_text(source.as_bytes())?);
                }
                "content_section" => {
                    content_text = Some(child.utf8_text(source.as_bytes())?);
                }
                _ => {}
            }
        }
        
        let metadata_str = metadata_text.ok_or(ParseError::MissingMetadata)?;
        let metadata = self.parse_yaml_metadata(metadata_str)?;
        
        Ok(ParsedBlock {
            id: metadata.id,
            block_type: metadata.block_type,
            name: metadata.name,
            metadata: metadata.metadata,
            content: content_text.unwrap_or("").to_string(),
        })
    }
}
```

### 6.3. YAML 元数据验证

```rust
#[derive(Debug, Deserialize)]
struct BlockMetadata {
    id: String,
    #[serde(rename = "type")]
    block_type: String,
    name: Option<String>,
    metadata: Option<serde_json::Value>,
}

impl BlockMetadata {
    fn validate(&self) -> Result<(), ValidationError> {
        // 验证 UUID 格式
        uuid::Uuid::parse_str(&self.id)
            .map_err(|_| ValidationError::InvalidUuid(self.id.clone()))?;
        
        // 验证块类型
        match self.block_type.as_str() {
            "markdown" | "code" | "link" | "recipe" | "metadata" => {}
            _ => return Err(ValidationError::UnknownBlockType(self.block_type.clone())),
        }
        
        // 类型特定验证
        if self.block_type == "code" {
            if let Some(meta) = &self.metadata {
                if !meta.get("language").is_some() {
                    return Err(ValidationError::MissingLanguage);
                }
            }
        }
        
        Ok(())
    }
}
```

## 7. 与 Markdown 的兼容性

### 7.1. 兼容的 Markdown 语法

在 `type: markdown` 的块中，支持所有标准 Markdown 语法：

```elf
---
id: markdown-content
type: markdown
---
# 标题

**粗体文本** 和 *斜体文本*

- 列表项 1
- 列表项 2

```python
# 代码块也在 markdown 内正常工作
print("Hello World")
```

[链接](https://example.com)

| 表格 | 支持 |
|------|------|
| 是   | 完整 |
```

### 7.2. 扩展语法

#### 块内引用语法
在 Markdown 内容中可以使用特殊语法引用其他块：

```markdown
参见 {{ref:block-name}} 中的实现细节。

插入代码：
{{include:setup-code}}
```

#### 动态内容标记
```markdown
这部分内容由 Recipe 动态生成：
{{recipe:build-summary}}
```

## 8. 错误处理和验证

### 8.1. 语法验证规则

**结构验证**：
- 每个块必须有有效的 YAML 元数据部分
- `id` 和 `type` 字段必须存在
- `name` 在同一文档内必须唯一（如果指定）
- 块分隔符 `---` 必须独占一行

**类型验证**：
- `code` 块的 `metadata.language` 应为有效的语言标识符
- `link` 块的 `target` 必须是有效的 URI 格式
- `recipe` 块的内容必须是有效的 YAML 配置
- `parent` 引用必须指向存在的块 ID

**引用完整性**：
- Link Block 的目标 URI 应该可以解析
- Recipe 中的外部引用应该可达
- 不允许循环的 `parent` 引用

### 8.2. 错误类型定义

```rust
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Tree-sitter 解析失败")]
    TreeSitterFailed,
    
    #[error("YAML 元数据解析错误: {0}")]
    YamlError(#[from] serde_yaml::Error),
    
    #[error("块 '{block_id}' 缺少必需的 '{field}' 字段")]
    MissingRequiredField { block_id: String, field: String },
    
    #[error("无效的 UUID 格式: {0}")]
    InvalidUuid(String),
    
    #[error("未知的块类型: {0}")]
    UnknownBlockType(String),
    
    #[error("重复的块名称: {0}")]
    DuplicateBlockName(String),
    
    #[error("无效的 URI 引用: {0}")]
    InvalidUri(String),
    
    #[error("循环的 parent 引用: {0}")]
    CircularParentReference(String),
}

impl ParseError {
    pub fn user_message(&self) -> String {
        match self {
            ParseError::MissingRequiredField { block_id, field } => 
                format!("块 '{}' 缺少必需的 '{}' 字段", block_id, field),
            ParseError::InvalidUuid(uuid) => 
                format!("无效的 UUID 格式: {}。请使用标准 UUID 格式。", uuid),
            ParseError::UnknownBlockType(block_type) => 
                format!("未知的块类型: {}。支持的类型: markdown, code, link, recipe", block_type),
            _ => self.to_string(),
        }
    }
    
    pub fn suggestions(&self) -> Vec<&'static str> {
        match self {
            ParseError::MissingRequiredField { field, .. } if field == "id" => vec![
                "为块添加唯一的 UUID 标识符",
                "使用 'uuidgen' 命令生成新的 UUID",
                "确保 id 字段在 YAML 元数据中存在"
            ],
            ParseError::MissingRequiredField { field, .. } if field == "type" => vec![
                "指定块类型: markdown, code, link, recipe",
                "确保 type 字段在 YAML 元数据中存在",
                "检查 YAML 语法是否正确"
            ],
            ParseError::InvalidUri(_) => vec![
                "检查 URI 格式: elf://[user/]repo/doc[#block-name]",
                "确认目标仓库和文档存在",
                "验证块名称拼写是否正确"
            ],
            _ => vec![]
        }
    }
}
```

## 9. 最佳实践

### 9.1. 命名约定

**块命名**：
- 使用描述性名称：`user-authentication` 而不是 `block1`
- 代码块包含语言信息：`setup-bash-script`
- Recipe 块描述用途：`export-documentation`

**元数据使用**：
- 为代码块始终指定 `language`
- 为公共块提供 `description`
- 使用有意义的 `tags` 便于 Recipe 选择
- 合理使用 `parent` 构建层级结构

### 9.2. 结构组织

**层级结构**：
- 使用 `parent` 字段构建清晰的层级结构
- 避免过深的嵌套（建议不超过 4 层）
- 同级块在物理上相邻排列

**内容分离**：
- 将不同类型的内容分到不同的块
- 使用 Link Block 引用外部内容而不是复制
- Recipe 块独立于内容块

### 9.3. 协作友好

**版本控制优化**：
- 每行内容尽量保持简短
- 避免在单行中混合多个概念
- 块的顺序尽量保持稳定

**冲突预防**：
- 为可能并发编辑的区块设置 `merge_method: "manual"`
- 使用清晰的 `owner` 字段标识责任人
- 通过 `tags` 标识协作边界

## 验证清单

### 语法正确性
- [ ] 所有块都有有效的 YAML 元数据
- [ ] 必需字段 `id` 和 `type` 存在
- [ ] UUID 格式符合标准
- [ ] 块分隔符格式正确

### 语义完整性  
- [ ] 块名称在文档内唯一
- [ ] Parent 引用指向存在的块
- [ ] 代码块指定了语言类型
- [ ] Link Block 的 URI 格式有效

### 协作友好性
- [ ] 块有合理的层级结构
- [ ] 使用描述性的命名
- [ ] 关键块设置了所有者
- [ ] Tags 用于内容分类

### 解析性能
- [ ] 文档大小合理（建议 < 1MB）
- [ ] 块数量适中（建议 < 1000个）
- [ ] 引用深度有限（建议 < 10层）
- [ ] YAML 元数据结构简单

这个语法规范确保了 `.elf` 格式既保持人类可读性，又具备结构化文档系统所需的元数据能力和协作特性。
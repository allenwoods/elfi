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

### 1.2. 扩展性设计原则 ⚠️

**重要：elfi 系统的设计哲学是"结构与语义分离"**

#### 系统职责边界
- **elfi 负责**：
  - 文件结构解析（块分割、YAML 解析）
  - CRDT 数据同步和版本控制
  - 命令行接口和基础操作
  - 网络通信和存储抽象
- **elfi 不负责**：
  - 具体类型的业务语义解释
  - 特定属性的验证和处理逻辑
  - 内容格式的渲染和展示
  - 领域特定的冲突解决策略

#### 用户定义优先原则
- **类型系统**：完全由用户和项目定义，elfi 只是透明传递
- **属性名称**：任意字符串，含义由应用层解释
- **关系类型**：开放式设计，支持任意项目特定关系
- **内容格式**：elfi 只保证文本存储，格式解释由插件处理

#### 插件化扩展机制
- **类型处理器**：通过插件系统注册特定类型的处理逻辑
- **冲突解决器**：可插拔的冲突解决策略
- **渲染引擎**：可扩展的内容渲染机制
- **验证器**：可选的内容验证插件

这种设计确保 elfi 保持轻量和通用性，避免被任何特定领域的需求绑定。

### 1.3. 与 Markdown 的关系

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
attributes:
  parent: f47ac10b-58cc-4372-a567-0e02b2c3d479
  language: bash
---
npm install elfi-core

---
id: 2e8f9a3b-1c4d-5a6b-7c8d-9e0f1a2b3c4d
type: relations
name: document-relations
attributes:
  owner: alice
  merge_method: manual
---
setup-code -> introduction [child_of] {}
introduction -> elf://shared-lib/utils/helpers#string-utils [references] {display_text: "共享工具函数"}
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

### 3.1. 核心字段

每个Block都有且仅有以下4个核心字段：

#### `id` (字符串，必需)
```yaml
id: f47ac10b-58cc-4372-a567-0e02b2c3d479
```
- **作用**：块的全局唯一标识符（系统内部的真实身份）
- **格式**：**必须使用标准 UUID 格式**
- **约束**：全局唯一，系统自动生成
- **用途**：CRDT 操作的目标标识、跨文档引用、历史追踪

#### `type` (字符串，必需)
```yaml
type: markdown  # 或 code, relations, recipe 等 - 完全由用户定义
```
- **作用**：声明块的内容类型，用于上层应用的处理分发
- **⚠️ 重要说明**：**elfi 系统本身不定义任何具体类型**，以下仅为常用约定示例
- **常用约定类型**（非系统强制）：
  - `markdown`: Markdown 文本内容（用户约定）
  - `code`: 程序代码（用户约定）
  - `relations`: 关系管理（用户约定，可用任意名称）
  - `recipe`: 转换配方（用户约定）
  - `metadata`: 元数据（用户约定）
- **设计原则**：
  - **类型完全自由**：用户可以定义任意字符串作为类型
  - **语义由应用定义**：elfi 只负责存储和传递，不解释类型含义
  - **插件机制处理**：具体类型的业务逻辑通过插件系统实现

#### `name` (字符串，可选)
```yaml
name: helper-functions
```
- **作用**：人类可读的块名称（便于记忆的别名）
- **约束**：同一文档内必须唯一（如果指定）
- **用途**：跨文档引用目标、内部导航、Recipe 选择器
- **格式**：推荐使用 `kebab-case` 格式（如 `intro-section`, `utils-block`）

#### `attributes` (对象，可选)
```yaml
attributes:
  # 结构化属性
  parent: parent-block-id
  tags: ["export", "utility"]
  description: "工具函数集合"
  
  # 协作属性  
  owner: "alice"
  merge_method: "manual"  # crdt | manual
  contributors: ["alice", "bob"]
  
  # 类型特定属性
  language: python          # code 类型
  interactive: true         # 交互式渲染
  
  # 用户自定义属性（示例）
  speaker: "Alice"          # conversation 类型
  timestamp: "2024-01-15T14:00:00Z"
  role: "Product Manager"
  author: "Bob"
  derived_from: ["block-1", "block-2"]
  meeting_type: "技术讨论"
```

**⚠️ 重要说明**：**所有属性名称和含义都完全由用户定义**，elfi 系统不强制任何特定属性。

**常用约定属性**（仅供参考，非系统要求）：
- `parent`: 父块 ID，用于构建层级结构（用户约定）
- `tags`: 标签数组，用于分类和过滤（用户约定）
- `description`: 块的描述信息（用户约定）
- `owner`: 区块所有者（协作约定）
- `contributors`: 贡献者列表（协作约定）
- `merge_method`: 合并策略（协作约定，值可以是任意字符串）

**项目特定属性示例**（完全由用户定义）：
- **代码项目**: `language`, `author`, `test_coverage`
- **对话记录**: `speaker`, `timestamp`, `topic`, `role`
- **内容管理**: `derived_from`, `source`, `references`
- **会议管理**: `meeting_type`, `estimated_duration`, `participants`
- **版本控制**: `version`, `created`, `modified`, `status`
- **业务流程**: `priority`, `assignee`, `deadline`, `category`

**灵活性原则**：
- **属性名自由**：可以使用任意字符串作为属性名
- **值类型自由**：支持字符串、数组、对象、数字等任意JSON类型
- **语义自定义**：属性的业务含义完全由项目定义
- **处理可选**：上层应用可选择处理或忽略任意属性

**设计原则**：
- **命名约定**: 使用 `snake_case` 格式，避免特殊字符
- **数据类型**: 支持字符串、数组、对象、布尔值、数字
- **嵌套限制**: 建议attributes嵌套深度不超过 3 层，保持解析性能
- **大小限制**: 单个 attributes 对象建议不超过 1KB

## 4. 内容部分规范

### 4.1. 通用内容规则

内容部分包含块的原始文本，紧跟在元数据部分的结束 `---` 之后，延伸到下一个块分隔符或文件末尾。

### 4.2. 特殊块类型的内容格式

#### Relations Block 语法

Relations Block 是一种特殊类型的块，专门用于管理文档中所有块间关系。**关系信息存储在内容部分**：

```elf
---
id: doc-relations
type: relations
name: document-relations
attributes:
  description: "管理文档内所有块间关系"
  owner: "alice"
  merge_method: "manual"
---
# 文档内关系
setup-code -> introduction [child_of] {}
introduction -> summary [parent_of] {weight: 1.0}

# 跨文档引用 
introduction -> elf://shared-lib/utils/helpers#string-utils [references] {display_text: "共享工具函数"}
api-design -> elf://team-notes/meetings/2024-01-15#decision [derived_from] {meeting_id: "tech-discussion"}
```

**设计说明**：
- **attributes部分**: 存储Relations Block本身的属性和所有权信息
- **内容部分**: 使用简洁语法描述所有块间关系

**Relations语法格式**：
```
source -> target [relation_type] {properties}
```

**组成部分**：
- `source`: 源块名称或ID
- `target`: 目标块名称、ID或URI
- `relation_type`: 关系类型（见下文）
- `properties`: 关系属性（JSON对象格式，可选）

**⚠️ 重要说明**：**关系类型完全由用户定义**，elfi 系统不限制关系类型。

**常用约定关系类型**（仅为建议，非系统强制）：
- `parent_of` / `child_of`: 层级关系（用户约定）
- `references`: 一般引用关系（用户约定）
- `includes`: 内容包含关系（用户约定）
- `derived_from`: 派生关系（用户约定）
- `implements`: 实现关系（用户约定）
- `depends_on`: 依赖关系（用户约定）

**项目自定义关系示例**：
- **软件开发**: `tests`, `documents`, `reviews`, `replaces`
- **内容管理**: `summarizes`, `expands_on`, `contradicts`, `updates`
- **业务流程**: `approves`, `blocks`, `triggers`, `completes`
- **学术研究**: `cites`, `supports`, `refutes`, `builds_upon`

**关系灵活性**：
- **类型自由**：可以使用任意字符串作为关系类型
- **属性自由**：关系属性 `{properties}` 可以包含任意JSON数据
- **语义开放**：关系的具体含义由项目和应用定义

**跨文档引用格式**：
```
local-block -> elf://[user/]repo/doc#block-name [relation_type] {props}
```

**Relations Block特性**：
- **所有权模型**: 通过`owner`属性指定关系管理者
- **Manual合并**: 使用`merge_method: manual`避免关系冲突
- **集中管理**: 一个文档中的所有关系统一管理
- **URI支持**: 原生支持跨文档引用

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
            attributes: metadata.attributes,
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
    attributes: Option<serde_json::Value>,
}

impl BlockMetadata {
    fn validate(&self) -> Result<(), ValidationError> {
        // 验证 UUID 格式
        uuid::Uuid::parse_str(&self.id)
            .map_err(|_| ValidationError::InvalidUuid(self.id.clone()))?;
        
        // 注意：elfi 系统不验证具体的块类型，任何字符串都是有效的
        // 以下代码仅为示例，实际实现中应该接受任意类型
        // match self.block_type.as_str() {
        //     "markdown" | "code" | "relations" | "recipe" | "metadata" => {}
        //     _ => return Err(ValidationError::UnknownBlockType(self.block_type.clone())),
        // }
        
        // elfi 只验证类型是非空字符串
        if self.block_type.is_empty() {
            return Err(ValidationError::EmptyBlockType);
        }
        
        // 注意：elfi 系统不执行类型特定验证
        // 以下代码仅为示例，实际的验证逻辑应由应用层和插件实现
        // if self.block_type == "code" {
        //     if let Some(attrs) = &self.attributes {
        //         if !attrs.get("language").is_some() {
        //             return Err(ValidationError::MissingLanguage);
        //         }
        //     }
        // }
        
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

**语义验证**（完全可选，由应用层决定）：
- `code` 块的 `attributes.language`：如果项目约定使用此属性，应为项目认可的语言标识符
- `relations` 块的内容：如果使用关系语法，应符合项目定义的格式
- `recipe` 块的内容：如果项目约定使用 YAML，应符合项目的配置规范
- `parent` 引用：如果项目使用层级结构，应指向存在的块 ID

**⚠️ 重要**：elfi 系统本身不执行这些验证，具体验证逻辑由应用层和插件实现。

**引用完整性**：
- Relations Block 中的目标 URI 应该可以解析
- Recipe 中的外部引用应该可达
- 不允许循环的 `parent` 引用
- Relations Block 中不允许循环引用

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
            ParseError::EmptyBlockType => 
                "块类型不能为空字符串".to_string(),
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
                "指定任意块类型（如: markdown, code, custom_type 等）",
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
- **必需字段**: 为代码块始终指定 `language`
- **描述信息**: 为公共块提供清晰的 `description`
- **分类标记**: 使用有意义的 `tags` 便于 Recipe 选择和内容过滤
- **层级结构**: 合理使用 `parent` 构建文档层次
- **协作追踪**: 使用 `owner` 和 `contributors` 明确责任
- **自定义扩展**: 根据用例需要添加领域特定属性（如对话的 `speaker`, `timestamp`）

**Metadata 最佳实践**：
- **语义化命名**: 使用描述性的属性名，如 `meeting_type` 而非 `type2`
- **数据类型一致**: 同类属性在不同区块间保持相同格式（如时间戳格式）
- **适度使用**: 避免过多冗余信息，重点记录对后续处理有价值的数据
- **Recipe友好**: 考虑属性在模板系统中的可用性

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
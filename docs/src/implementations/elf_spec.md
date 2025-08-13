# ELF 语法规范

本文档定义了 `.elf` 文件格式的完整语法规范。鉴于 ELF 格式在基础文本处理上与 Markdown 高度相似，本规范重点说明 ELF 格式的特殊语法扩展和与标准 Markdown 的差异之处。

## 1. 基础原则

`.elf` 文件本质上是**分块的 Markdown**，每个块都有自己的元数据和内容。与标准 Markdown 文档的主要区别：

- **显式分块**：通过 `---` 分隔符明确划分内容块
- **元数据驱动**：每个块都有 YAML 前置元数据
- **类型化内容**：通过 `type` 字段显式声明块的内容类型
- **结构化关系**：通过 `parent` 等元数据建立块之间的关系

## 2. 文档结构

### 2.1. 文件级结构

```elf
---
id: f47ac10b-58cc-4372-a567-0e02b2c3d479
name: introduction
type: markdown
---
# 项目介绍

这是一个基于CRDT的协作文档系统。

---
id: a1b2c3d4-5e6f-7890-1234-56789abcdef0
name: setup-code
type: code
metadata:
  parent: f47ac10b-58cc-4372-a567-0e02b2c3d479
  language: bash
---
npm install elfi-core

---
id: 2e8f9a3b-1c4d-5a6b-7c8d-9e0f1a2b3c4d
name: shared-utils
type: link
metadata:
  parent: f47ac10b-58cc-4372-a567-0e02b2c3d479
---
target: elf://shared-lib/utils/helpers#string-utils
ref_type: include
display_text: 共享工具函数

---
id: 3f9a8b2c-7d6e-4f5a-8b9c-0d1e2f3a4b5c
name: build-recipe
type: recipe
metadata:
  parent: f47ac10b-58cc-4372-a567-0e02b2c3d479
---
name: "project-export"
selector:
  types: ["code", "markdown"]
  tags: ["export"]
transform:
  - type: "concat"
    template: "{{content}}\n\n"
output:
  format: "markdown"
  filename: "README.md"
```

### 2.2. 块分隔符

- **分隔符**：`---`（三个连字符）
- **位置**：独占一行，前后可有空白字符
- **作用**：分隔相邻的块，同时也标识元数据部分的开始和结束

## 3. 元数据部分

### 3.1. 必需字段

#### `id` (字符串)
```yaml
id: f47ac10b-58cc-4372-a567-0e02b2c3d479
```
- **作用**：块的全局唯一标识符（系统内部的真实身份）
- **格式**：**必须使用标准 UUID 格式**
- **约束**：全局唯一，系统自动生成

#### `type` (字符串)
```yaml
type: markdown  # 或 code, link, recipe 等
```
- **作用**：声明块的内容类型
- **标准类型**：
  - `markdown`: 标准 Markdown 文本
  - `code`: 程序代码
  - `link`: 跨文档引用
  - `recipe`: 内容转换配方
  - 其他自定义类型

### 3.2. 可选字段

#### `name` (字符串)
```yaml
name: helper-functions
```
- **作用**：人类可读的块名称（便于记忆的别名）
- **约束**：同一文档内必须唯一（如果指定）
- **用途**：用于跨文档引用、内部导航、Recipe选择器
- **格式**：推荐使用 `kebab-case` 格式（如 `intro-section`, `utils-block`）

#### `metadata` (对象)
```yaml
metadata:
  parent: parent-block-id
  language: python
  interactive: true
  tags: ["export", "utility"]
```
- **作用**：存储块的扩展属性
- **标准属性**：
  - `parent`: 父块ID，用于构建层级结构
  - `language`: 代码语言（适用于code类型）
  - `interactive`: 是否需要交互式渲染
  - `tags`: 标签数组，用于分类和选择
  - `description`: 块的描述信息

## 4. 特殊块类型

### 4.1. Link Block 语法

Link Block 的内容部分使用特殊的键值对语法：

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

### 4.2. Recipe Block 语法

Recipe Block 的内容部分使用 YAML 配置：

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

### 5.1. 标准 URI 格式

```
elf://[user/]repo/doc[#block-name]
```

**组成部分**：
- `user/`: 用户或组织名（可选）
- `repo`: 仓库名（必需）
- `doc`: 文档名（必需）
- `#block-name`: 块名称（可选，默认引用整个文档）

### 5.2. 相对引用

```
./doc#block-name          # 同仓库相对引用
#block-name               # 同文档内引用
../other-repo/doc         # 跨仓库相对引用
```

## 6. 与 Markdown 的兼容性

### 6.1. 兼容的 Markdown 语法

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
# 代码块也在markdown内正常工作
print("Hello World")
```

[链接](https://example.com)
```

### 6.2. 扩展语法

#### 块内引用
在 Markdown 内容中可以使用特殊语法引用其他块：

```markdown
参见 {{ref:block-name}} 中的实现细节。

插入代码：
{{include:setup-code}}
```

#### 动态内容标记
```markdown
这部分内容由Recipe动态生成：
{{recipe:build-summary}}
```

## 7. 语法验证规则

### 7.1. 结构验证
- 每个块必须有有效的 YAML 元数据部分
- `id` 和 `type` 字段必须存在
- `name` 在同一文档内必须唯一（如果指定）
- 块分隔符 `---` 必须独占一行

### 7.2. 类型验证
- `code` 块的 `metadata.language` 应为有效的语言标识符
- `link` 块的 `target` 必须是有效的 URI 格式
- `recipe` 块的内容必须是有效的 YAML 配置
- `parent` 引用必须指向存在的块 ID

### 7.3. 引用完整性
- Link Block 的目标 URI 应该可以解析
- Recipe 中的外部引用应该可达
- 不允许循环的 `parent` 引用

## 8. 错误处理

### 8.1. 解析错误
```
错误: 块 'block-1' 缺少必需的 'type' 字段
位置: 第 3 行
建议: 在元数据中添加 type 字段
```

### 8.2. 引用错误
```
警告: 无法解析引用 'elf://missing/doc#block'
块: 'ref-block-1'
处理: 使用占位符内容
```

### 8.3. 类型错误
```
错误: Recipe 块 'recipe-1' 的YAML配置无效
原因: 'selector' 字段格式错误
位置: 第 15-20 行
```

## 9. 最佳实践

### 9.1. 命名约定
- 使用描述性的块名称：`user-authentication` 而不是 `block1`
- 代码块包含语言信息：`setup-bash-script`
- Recipe 块描述用途：`export-documentation`

### 9.2. 结构组织
- 使用 `parent` 字段构建清晰的层级结构
- 通过 `tags` 实现横向分类
- 将相关的块在物理上相邻排列

### 9.3. 元数据使用
- 为代码块始终指定 `language`
- 为公共块提供 `description`
- 使用有意义的 `tags` 便于 Recipe 选择

这个规范确保了 `.elf` 格式既保持了 Markdown 的人类可读性，又具备了结构化文档系统所需的元数据驱动能力。
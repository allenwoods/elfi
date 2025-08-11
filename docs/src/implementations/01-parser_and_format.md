# 实现：01 - 解析器与 .elf 格式

本文档规定了 `.elf` 文件的初始纯文本格式，以及将此格式解析为基于 CRDT 的内存模型的策略。

## 1. `.elf` 纯文本格式

该格式旨在实现人类可读、对版本控制友好且易于解析。它由一系列**块 (Blocks)** 组成，块之间由标准 `---` 分隔符隔开。此设计直接受到所提供的 `example.elf.md` 的启发。

一个**块**有两个不同的部分：
1.  **元数据部分** (YAML Frontmatter)
2.  **内容部分**

### 块结构示例：

```
---
id: block-C
type: code
metadata:
  parent: block-B
  language: python
---
import pandas as pd
penguins = sns.load_dataset("penguins")
penguins.head()
```

### 元数据部分

-   **语法**：一个有效的 YAML 对象，由 `---` 包围，位于块的开头。
-   **必填字段**：
    -   `id` (String)：块的唯一标识符。虽然任何字符串都有效，但建议为新块使用 UUID 以防止冲突。
    -   `type` (String)：定义块类型的字符串字面量（例如 `"markdown"`, `"code"`）。这决定了内容如何被渲染和处理。
-   **可选字段**：
    -   `metadata` (Object)：一个用于存储任意元数据的嵌套 YAML 对象。这为可扩展性提供了命名空间，并且是存储语义信息（如以下内容）的必需位置：
        -   `parent` (String)：父块的 `id`，用于构建逻辑层级。
        -   `language` (String)：`code` 块的语言（例如 `"python"`, `"rust"`）。
        -   `interactive` (Boolean)：一个供 Tangle 使用的标志，以确定一个块是否应作为交互式孤岛被“激活”。

### 内容部分

内容部分包含块的原始文本。它紧跟在元数据部分的结束 `---` 之后，并一直延伸到下一个块分隔符 (`---`) 或文件末尾。

## 2. 解析策略 (`elfi-parser`)

`elfi-parser` crate 负责将 `.elf` 文本格式转换为 `automerge` 文档实例。

### 2.1. Tree-sitter 语法

将为 `tree-sitter` 创建一个 `grammar.js` 文件。它将定义以下结构：
-   一个 `source_file` 由一个或多个 `block` 节点组成。
-   一个 `block` 节点由一个 `metadata_section` 和一个 `content_section` 组成。
-   `metadata_section` 将被识别为由 `---` 包围的文本。
-   `content_section` 是剩余的文本。

这种方法可以高效且容错地将文件解析为具体语法树 (CST)。

### 2.2. CST 到 CRDT 的转换

解析器将提供一个主函数：
`pub fn parse_to_doc(text: &str) -> Result<automerge::AutoCommit, ElfiError>`

转换过程如下：
1.  输入的 `text` 被 Tree-sitter 解析为 CST。
2.  解析器遍历 CST 的顶级 `block` 节点。
3.  对于每个 `block` 节点：
    a. 提取其 `metadata_section` 子节点的文本内容，并使用 YAML 解析器（例如 `serde_yaml`）进行解析。
    b. 在一个新的 `automerge` 事务中创建一个新的 map 对象，代表该块。
    c. 将解析出的 YAML 字段（`id`, `type`, `metadata`）插入到 Automerge map 中。
    d. `content` 字段被创建为一个 `automerge::Text` 对象，并使用 `content_section` 子节点的文本进行初始化。
    e. 这个新的块 map 被附加到 Automerge 文档中的顶级 `blocks` 列表中。
4.  返回最终填充好的 `automerge::AutoCommit` 文档。

## 3. 测试要点 (Testing Points)

- **单元测试 (Unit Tests)**:
    - **有效文件解析**:
        - 测试包含单个块、多个块以及嵌套块（通过 `parent` 元数据）的有效 `.elf` 文件。
        - 验证解析后的 `automerge` 文档结构是否与源文件完全对应。
    - **无效文件处理**:
        - **格式错误的 YAML**: 测试元数据部分包含无效 YAML 的文件，验证解析器是否会返回一个可识别的 `Error`。
        - **缺少必填字段**: 测试缺少 `id` 或 `type` 字段的块，验证是否会返回错误。
        - **文件编码**: 测试不同编码（如 UTF-16）或包含无效字符的文件，确保解析器不会崩溃。
    - **边缘情况**:
        - 测试空文件。解析器应返回一个空的 `automerge` 文档。
        - 测试只包含分隔符 (`---`) 的文件。
        - 测试内容部分为空的块。
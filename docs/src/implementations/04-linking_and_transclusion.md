# 实现：04 - 链接与转译 (Linking and Transclusion)

本文档为 "文档即 App" 的核心场景提供实现思路。它定义了一种机制，允许一个 `.elf` 文件引用并嵌入另一个 `.elf` 文件中的内容，实现内容的动态组合和复用。

## 1. `link` 块类型

为了实现内容的转译，我们引入一个新的块类型： `link`。

### 块结构示例

```yaml
---
id: link-block-1
type: link
metadata:
  parent: some-parent-block
---
elfi://doc-uuid-abc/block-id-xyz
```

- **`type: link`**: 明确指出这是一个链接块。
- **内容**: 链接块的内容是一个遵循特定 URI 方案的字符串，指向目标块。

## 2. Elfi URI 方案

我们定义一个 `elfi://` URI 方案来定位网络中的任何一个块。

- **格式**: `elfi://<document_id>/<block_id>`
- **`<document_id>`**: 目标文档的唯一标识符 (例如，UUID)。
- **`<block_id>`**: 目标文档中目标块的 `id`。

## 3. 解析与渲染逻辑

当 `elfi` 解释器或 Tangle UI 遇到一个 `link` 块时，它必须执行以下“解析 (resolving)”过程：

1.  **提取 URI**: 从块内容中解析出 `elfi://` URI。
2.  **获取目标文档**: 使用 `Repo` 实例，根据 `document_id` 获取对应的 `DocHandle`。这可能需要从网络或本地缓存加载文档。
3.  **获取目标块**: 从目标 `DocHandle` 中查找具有指定 `block_id` 的块。
4.  **内容替换**: 在渲染时，`link` 块本身不会被渲染。取而代之的是，渲染器会递归地渲染目标块的内容。

这个过程允许一个主 `.elf` 文件像一个“骨架”一样，将来自不同 `.elf` 文件的“器官”组合成一个完整的应用。

## 4. 测试要点 (Testing Points)

- **单元测试 (Unit Tests)**:
    - **URI 解析**: 编写一个函数 `parse_elfi_uri(uri: &str) -> Result<(String, String), Error>` 并为其编写单元测试，确保它可以正确解析有效 URI 并拒绝无效 URI。
    - **链接解析逻辑**: 在 `elfi-core` 中，测试解析 `link` 块的函数。
        - **成功解析**: 模拟一个场景，其中目标文档和块都存在，并验证是否返回了正确的内容。
        - **目标文档未找到**: 测试当 `document_id` 无效时，系统能否优雅地处理错误（例如，返回一个错误提示块）。
        - **目标块未找到**: 测试当 `block_id` 在目标文档中不存在时，系统如何处理。
        - **循环引用**: 设计一个测试，其中 `doc-A` 链接到 `doc-B`，而 `doc-B` 又链接回 `doc-A`。解析器必须能够检测到这种循环引用并阻止无限递归。
- **集成测试 (Integration Tests)**:
    - 创建 `main.elf` 和 `component.elf`。
    - 在 `main.elf` 中创建一个 `link` 块指向 `component.elf` 中的一个块。
    - 运行一个导出或预览命令，验证 `main.elf` 的输出是否正确地包含了 `component.elf` 的内容。
    - 修改 `component.elf` 中的块内容，然后再次预览 `main.elf`，验证内容是否自动更新。

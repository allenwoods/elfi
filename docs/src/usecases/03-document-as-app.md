# 测试场景 03: 文档即 App (Document as App)

本测试旨在验证 `elfi` 的内容组合与转译（transclusion）能力，这是构建复杂应用的基础。

- **核心目标**: 证明一个主 `.elf` 文件可以引用并动态地展示（转译）其他 `.elf` 文件中的内容块。

## 关联的实现文档

- `implementations/04-linking_and_transclusion.md`: 详细定义了 `link` 块类型、`elfi://` URI 方案以及解析逻辑。
- `implementations/02-core_logic.md`: `Repo` 需要能够根据 URI 获取其他文档的 `DocHandle`。

## 测试流程设计

1.  **准备 (Preparation)**
    -   创建两个 `.elf` 文档实例：
        -   `component.elf`: 包含一个可复用的 `code` 块，ID 为 `reusable-code`，内容为一个简单的 Python 函数 `def hello(): return "Hello from component!"`。
        -   `main.elf`: 包含一个 `link` 类型的块，其内容是一个指向 `component.elf` 中代码块的 Elfi URI。例如：`elfi://<component_doc_id>/reusable-code`。（`<component_doc_id>` 是 `component.elf` 的唯一标识符）。

2.  **执行 (Execution)**
    -   使用一个能够解析链接的 `elfi` 渲染器或导出器来处理 `main.elf`。例如，运行一个命令：
        ```bash
        elfi export main.elf --format resolved-json --resolve-links
        ```
        这里的 `--resolve-links` 是一个假设的标志，告诉导出器执行转译操作。

3.  **验证 (Verification)**
    -   **初次转译**: 检查 `main.elf` 导出的结果。确认 `link` 块没有被原样输出，而是被 `component.elf` 中 `reusable-code` 块的实际内容（即 Python 函数字符串）所替换。
    -   **动态更新**: 修改 `component.elf` 中 `reusable-code` 块的内容，例如将函数修改为 `def hello(): return "Hello, updated world!"`。
    -   再次执行第二步中的导出命令。
    -   检查 `main.elf` 的新导出结果，确认它反映了 `component.elf` 中更新后的内容。
    -   **错误处理**: 测试一个指向不存在的块或文档的 `link`，验证系统能否优雅地处理错误（例如，渲染一个错误提示信息而不是崩溃）。

## 成功标准

-   `main.elf` 的导出结果总是能正确地、动态地反映 `component.elf` 中被链接内容的最新状态。
-   链接解析器能够正确处理有效链接和无效链接，表现出良好的鲁棒性。

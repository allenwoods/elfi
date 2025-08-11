# 实现：03 - 命令行界面 (`elfi-cli`)

本文档规定了 `elfi-cli` 二进制 crate 所需的命令、参数和功能。CLI 作为离线文件管理、验证和转换的工具。

我们将使用 `clap` crate 及其 “derive” 功能来解析参数。

## 命令

### `elfi new <PATH>`

在指定路径创建一个新的、最小化的 `.elf` 文件。

-   **参数**：
    -   `<PATH>`：应创建新文件的路径。
-   **行为**：
    -   生成一个单一的、空的 `markdown` 块。
    -   为该块的 `id` 分配一个新的 `UUID`。
    -   将生成的文本写入指定文件。

### `elfi validate <PATH>`

验证 `.elf` 文件的结构和语法。

-   **参数**：
    -   `<PATH>`：要验证的 `.elf` 文件的路径。
-   **行为**：
    -   读取文件内容。
    -   通过 `elfi-parser` 运行它。
    -   如果解析成功，则打印成功消息并以代码 0 退出。
    -   如果解析失败，则打印详细的错误消息（例如，YAML 解析错误、Tree-sitter 语法错误）并以非零代码退出。

### `elfi export <PATH>`

解析一个 `.elf` 文件并将其当前状态导出为不同格式。

-   **参数**：
    -   `<PATH>`：`.elf` 文件的路径。
-   **选项**：
    -   `--format <FORMAT>`：输出格式。默认为 `json`。
        -   `json`：将完整的 Automerge 文档导出为单个 JSON 对象。
        -   `raw-json`：以与 Tangle-API 兼容的格式导出文档（参见 `02-core_logic.md`）。
-   **行为**：
    -   将 `.elf` 文件解析为内存中的 Automerge 文档。
    -   将文档序列化为指定格式。
    -   将结果打印到标准输出。

## 4. 测试要点 (Testing Points)

- **单元测试 (Unit Tests) / 集成测试 (Integration Tests)**:
    - **`elfi new`**:
        - 运行 `elfi new test.elf`。
        - 验证 `test.elf` 文件是否被创建。
        - 读取 `test.elf` 的内容，并使用 `elfi-parser` 验证其是否为一个有效的、包含单个块的 `.elf` 文件。
    - **`elfi validate`**:
        - 对一个有效的 `.elf` 文件运行 `elfi validate`，验证其是否以退出码 0 成功退出。
        - 对一个无效的（例如，YAML 语法错误）`.elf` 文件运行 `elfi validate`，验证其是否以非零退出码失败，并检查 `stderr` 中是否包含有意义的错误信息。
    - **`elfi export`**:
        - 创建一个测试用的 `.elf` 文件。
        - 运行 `elfi export --format json test.elf`，捕获 `stdout`，验证其是否为有效的 JSON。
        - 运行 `elfi export --format raw-json test.elf`，验证其输出是否符合 `Tangle API` 的 JSON 结构。
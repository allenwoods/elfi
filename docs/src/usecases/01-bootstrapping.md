# 测试场景 01: 自举 (Bootstrapping)

本测试旨在验证 `elfi` 的核心开发循环是否可以基于 `.elf` 文件本身进行。

- **核心目标**: 证明 `elfi` 可以可靠地管理和处理其自身的 Rust 源代码，实现“用 `elfi` 开发 `elfi`”。

## 关联的实现文档

- `implementations/02-core_logic.md`: 涉及文档状态的管理。
- `implementations/03-cli.md`: 涉及 `export` 命令，用于将代码块内容提取到文件系统。

## 测试流程设计

1.  **准备 (Preparation)**
    -   创建一个名为 `elfi-dev.elf` 的测试文件。
    -   在此文件中，创建一个 `code` 类型的块 (例如，ID 为 `dev-block-rust`)。
    -   将 `elfi-core` 项目中的一小部分真实、有效的 Rust 源代码（例如，一个简单的工具函数或一个结构体定义）复制到该代码块的内容中。
    -   在 `elfi` 项目的 `src/` 目录下，创建一个临时的 Rust 文件，例如 `src/tangled_code.rs`，并将其在 `lib.rs` 中通过 `mod tangled_code;` 引入。

2.  **执行 (Execution)**
    -   通过一个模拟的客户端或测试脚本，对 `elfi-dev.elf` 文件中的 `dev-block-rust` 块进行一次简单的编辑。例如，在函数中增加一行注释 `// This is a test comment.`。
    -   执行 `elfi` 的命令行工具，使用一个（假设的）`tangle` 命令，该命令专门用于将代码块导出并覆盖到文件系统中：
        ```bash
        elfi tangle elfi-dev.elf --block-id dev-block-rust --output-file src/tangled_code.rs
        ```

3.  **验证 (Verification)**
    -   检查 `src/tangled_code.rs` 文件的内容，确认它已经被更新，并且包含了第二步中增加的注释。
    -   在 `elfi` 项目的根目录下，运行完整的测试套件：
        ```bash
        cargo test
        ```

## 成功标准

-   `src/tangled_code.rs` 文件的内容与 `.elf` 文件中代码块的最新内容完全一致。
-   `cargo test` 命令成功通过，没有任何编译错误或测试失败。这证明了从 `.elf` 文件中提取并集成到项目中的代码是有效的，并且整个开发->导出->编译的流程是可靠的。

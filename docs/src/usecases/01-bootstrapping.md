# 测试场景 01: 自举 (Bootstrapping)

本测试旨在验证 `elfi` 的核心开发循环是否可以基于 `.elf` 文件本身进行。

- **核心目标**: 证明 `elfi` 可以可靠地管理和处理其自身的 Rust 源代码，实现"用 `elfi` 开发 `elfi`"。

## 📁 关联的测试文件

**[elfi-dev.elf](./elfi-dev.elf)** - 自举测试的完整 `.elf` 文档

这个文件包含了本测试场景的所有必要组件：
- `dev-block-rust`: 包含真实 Rust 源代码的区块
- `code-export-recipe`: Recipe 配置，用于导出代码到文件系统  
- `test-documentation`: 测试说明和进度追踪
- `changelog`: 详细的变更记录

## 关联的实现文档

- `implementations/02-core_logic.md`: 涉及文档状态的管理。
- `implementations/03-cli.md`: 涉及 `export` 命令和 Recipe 系统，用于将代码区块内容提取到文件系统。
- `implementations/04-recipe_system.md`: Recipe 系统的详细设计和实现。

## 测试流程设计

1.  **准备 (Preparation)**
    -   创建一个名为 `elfi-dev.elf` 的测试文件。
    -   在此文件中，创建以下区块：
        -   一个 `code` 类型的区块 (ID 为 `dev-block-rust`)，包含真实的 Rust 源代码
        -   一个 `recipe` 类型的区块 (ID 为 `code-export-recipe`)，用于定义代码导出规则
    -   在 `elfi` 项目的 `src/` 目录下，创建一个目标文件 `src/bootstrapped_code.rs`，并在 `lib.rs` 中通过 `mod bootstrapped_code;` 引入。

2.  **执行 (Execution)**
    -   通过一个模拟的客户端或测试脚本，对 `elfi-dev.elf` 文件中的 `dev-block-rust` 区块进行一次简单的编辑。例如，在函数中增加一行注释 `// This is a test comment.`。
    -   使用 Recipe 系统导出代码块到文件系统：
        ```bash
        # 使用自定义的code-export Recipe导出代码
        elfi export --recipe=code-export-recipe ./src/
        ```
        这个Recipe配置将会：
        - 选择 `code` 类型的区块
        - 将内容导出为 `src/bootstrapped_code.rs` 文件
        - 保持原有的 Rust 代码格式

3.  **验证 (Verification)**
    -   检查 `src/bootstrapped_code.rs` 文件的内容，确认它已经被更新，并且包含了第二步中增加的注释。
    -   验证Recipe系统的正确性：检查导出的文件与 `.elf` 文件中的区块内容完全一致。
    -   在 `elfi` 项目的根目录下，运行完整的测试套件：
        ```bash
        cargo test
        ```
    -   可选验证：使用 `elfi watch` 命令启动双向同步，验证在 IDE 中修改 `src/bootstrapped_code.rs` 后，更改能够自动同步回 `.elf` 文件。

## 成功标准

-   `src/bootstrapped_code.rs` 文件的内容与 `.elf` 文件中代码区块的最新内容完全一致。
-   Recipe 系统能够正确解析和导出代码区块，保持格式和结构。
-   `cargo test` 命令成功通过，没有任何编译错误或测试失败。这证明了从 `.elf` 文件中提取并集成到项目中的代码是有效的。
-   整个开发 -> Recipe导出 -> 编译的流程证明是可靠的，验证了 "**用 elfi 开发 elfi**" 的核心目标。
-   可选：双向同步机制 (`elfi watch`) 正常工作，允许在传统 IDE 中编辑并自动同步回 `.elf` 文件。

## Recipe 配置示例

为了实现上述测试，`code-export-recipe` 区块的内容应该包含类似以下的 YAML 配置：

```yaml
name: code-export-recipe
version: 1.0
description: 导出 Rust 代码区块到文件系统

# 选择器：只处理 code 类型的区块
selector:
  types: [code]
  filters:
    - lang: "rust"

# 转换规则：保持原有格式
transform:
  - type: code
    action: copy
    preserve_format: true

# 输出配置
output:
  format: file-per-block
  path_template: "src/{block_name}.rs"
  filename_mapping:
    "dev-block-rust": "bootstrapped_code.rs"
```

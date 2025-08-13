# 测试场景 02: 对话即文档 (Conversation as Document)

本测试旨在验证 `elfi` 的实时协作和 CRDT 冲突解决能力。

- **核心目标**: 证明多个用户可以并发地对同一个文档进行编辑，即使在出现冲突的情况下，所有用户的文档状态最终也能自动、正确地收敛到完全一致的状态。

## 📁 关联的测试文件

**[conversation.elf](./conversation.elf)** - 实时协作测试的完整 `.elf` 文档

这个文件包含了协作测试的所有组件：
- `initial-content`: 基础内容区块，用于并发修改测试
- `test-block-a` 和 `test-block-b`: 分别为 Client-A 和 Client-B 准备的测试区块
- `conflict-test-area`: 专门用于测试冲突解决的区域
- `collaboration-log`: 协作过程的详细日志记录
- `verification-checklist`: 测试验证清单
- `test-script-config`: 自动化测试的配置信息

## 关联的实现文档

- `implementations/02-core_logic.md`: 描述了 `Repo`、`DocHandle` 以及本地/远程变更的工作流程。
- `GEMINI.md` 中关于 Zenoh 的部分：描述了用于同步操作的底层网络协议。

## 测试流程设计

1.  **准备 (Preparation)**
    -   在测试环境中启动一个 Zenoh 服务端实例。
    -   在测试脚本中，创建两个（或更多）并行的 `Repo` 实例，我们称之为 `Client-A` 和 `Client-B`。这两个实例将模拟两个不同的协作者。
    -   让 `Client-A` 和 `Client-B` 都打开（`get_or_create_handle`）同一个文档 URL，例如 `elfi/test/conversation.elf`。
    -   `Client-A` 首先创建一个初始块，例如一个 `markdown` 块，内容为 "Initial content."，并等待 `Client-B` 同步到这个状态。

2.  **执行 (Execution)**
    -   **场景 A: 无冲突并发编辑 (Non-conflicting concurrent edits)**
        -   `Client-A` 在文档的开头添加一个新块 (ID: `block-A`)。
        -   几乎在同一时间，`Client-B` 在文档的末尾添加一个新块 (ID: `block-B`)。
    -   **场景 B: 冲突并发编辑 (Conflicting concurrent edits)**
        -   `Client-A` 和 `Client-B` 同时修改初始块中同一行文本。例如，都将 "Initial content." 修改为不同的内容。
        -   `Client-A` 修改为: "My initial content."
        -   `Client-B` 修改为: "Our initial content."

3.  **验证 (Verification)**
    -   在执行每个场景后，给予一小段等待时间，以确保所有 CRDT 操作都通过 Zenoh 网络传播完毕。
    -   从 `Client-A` 和 `Client-B` 的 `DocHandle` 中分别导出完整的 `automerge` 文档状态。
    -   比较这两个文档状态。最可靠的方法是比较 `automerge` 文档的哈希值（heads）。

## 成功标准

-   在场景 A 和 B 之后，`Client-A` 和 `Client-B` 的文档哈希值必须完全相同。这证明了它们的文档状态已经收敛一致。
-   在场景 B（冲突）之后，检查合并后的文本内容。它应该符合 `automerge` 的预期合并行为（通常是字符级的交错合并，结果可能是 "My Our initial content." 或类似的确定性结果），证明冲突被自动解决了。

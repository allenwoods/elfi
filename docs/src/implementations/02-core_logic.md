# 实现：02 - 核心逻辑 (`elfi-core`)

本文档详细介绍了 `elfi-core` 库的内部数据结构和工作流程，该库负责协调数据模型、同步和 API 层。

## 1. 核心数据结构

为了在 `async` 环境中有效管理并发和状态，我们将使用 `tokio` 和几种并发数据结构。

### `Repo` 结构体

`Repo` 是应用程序的主要入口点。它管理网络会话和所有活动的文档句柄。

```rust
use dashmap::DashMap;
use std::sync::Arc;
use zenoh::Session;

pub struct Repo {
    zenoh_session: Arc<Session>,
    doc_handles: DashMap<String, Arc<DocHandle>>,
}
```
- `zenoh_session`：一个共享的 Zenoh 会话引用，用于所有网络操作。
- `doc_handles`：一个线程安全的哈希映射 (`DashMap`)，将文档的 URL（作为字符串）映射到其对应的 `DocHandle`。

### `DocHandle` 结构体

`DocHandle` 管理单个文档的状态和同步。

```rust
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use automerge::AutoCommit;

pub struct DocHandle {
    doc_url: String,
    doc: Arc<Mutex<AutoCommit>>,
    subscribers: broadcast::Sender<()>, // 通知监听器任何变更
}
```
- `doc_url`：文档的唯一标识符。
- `doc`：文档的内存表示，包裹在 `tokio::sync::Mutex` 中以确保安全的并发访问。
- `subscribers`：一个 `tokio::sync::broadcast` 通道。每当文档状态发生变化时（无论是本地编辑还是远程更新），都会在此通道上发送一条消息。UI 层将订阅此通道的接收端。

## 2. 核心工作流程

### 本地变更工作流程 (`handle.change()`)

此工作流程描述了当用户修改文档时发生的情况。

1.  **获取锁**：应用程序调用 `handle.change()`。第一步是异步获取 `DocHandle` 的 `Mutex<AutoCommit>` 上的锁。
2.  **执行闭包**：执行用户提供的闭包。它接收一个对 `AutoCommit` 文档的可变引用，并执行其编辑（例如，插入文本、更改元数据）。
3.  **获取变更**：当闭包完成且锁被释放后，我们调用 `doc.get_last_local_changes()` 来检索刚刚进行的二进制编码的变更。
4.  **本地广播**：调用 `subscribers.send(())` 以立即通知任何本地监听器（如 UI）文档已更新。
5.  **远程发布**：将二进制变更发布到适当的 Zenoh 键空间（例如 `/elfi/docs/{doc_url}/ops`），供远程对等方接收。

### 远程变更工作流程 (Zenoh 订阅任务)

每个 `DocHandle` 将生成一个专用的 `tokio` 任务来监听来自网络的传入变更。

1.  **订阅**：该任务订阅文档的 Zenoh 键空间。
2.  **监听**：它异步等待传入的消息。
3.  **获取锁**：当收到包含远程变更的消息时，该任务获取 `DocHandle` 的 `Mutex<AutoCommit>` 上的锁。
4.  **应用变更**：使用 `doc.load_incremental(&remote_changes)` 将变更应用到本地模型。
5.  **本地广播**：调用 `subscribers.send(())` 以通知本地监听器更新，从而触发 UI 刷新。

## 3. 冲突解决工作流程 (Conflict Resolution Workflow)

`automerge` 本身通过 CRDT 算法确保数据最终会收敛，但这种收敛可能是“语义盲目”的。为了实现 `designs` 文档中定义的块级语义冲突解决，`elfi-core` 在接收到远程变更后必须执行一个更复杂的流程。

此工作流程在上述“远程变更工作流程”的第 4 步和第 5 步之间进行：

**4a. 检测冲突**: 在调用 `doc.load_incremental()` 之后，`elfi-core` 必须检查刚刚被修改的属性是否存在并发写入。
   - 遍历刚刚应用的变更集（changeset），找出所有被修改的块及其属性路径。
   - 对于每个被修改的属性，调用 `doc.get_conflicts(path)`。

**4b. 策略分发**: 如果 `get_conflicts` 返回了多个值，证明存在冲突。此时，系统根据块的 `type` 执行不同策略：
   - **`type: "code"`**: 这是最需要语义处理的场景。
     - **不自动合并**: 系统不会尝试自动合并代码。
     - **标记冲突**: 系统会在该块的 `metadata` 中设置一个标志，例如：`handle.change(doc => { doc.block_mut(id).metadata_mut().put("conflict", true); })`。
     - **暴露冲突版本 (可选)**: 系统可以将冲突的几个版本的值也存入元数据中，供 UI 层读取和展示。
   - **`type: "markdown"`**: 对于文本，`automerge` 的 `Text` CRDT 自身的交错合并行为通常是可接受的。因此，默认情况下我们信任其合并结果，不进行额外处理。
   - **其他类型**: 可根据需要定义其他策略。

**4c. UI 层响应**:
   - Tangle/UI 层通过订阅文档变更来监听 `metadata` 的变化。
   - 当检测到 `metadata.conflict == true` 时，UI 负责将该块高亮显示，并提供一个合并工具（例如，一个并排的 diff 视图），让用户来做出最终的裁决。
   - 用户完成手动合并后，UI 会调用 `handle.change()` 将最终确认的内容写回块中，并移除 `conflict` 标志。这次新的写入操作会成为所有先前冲突版本的后代，从而在 CRDT 的历史中正式解决该冲突。

## 4. Tangle API JSON 结构 (`get_document_state`)

为了便于在解耦的 UI 中进行渲染，`get_document_state` 函数将返回一个具有可预测的、受 Lexical 启发的结构的 JSON 对象。

```json
{
  "root_block_ids": ["block-A", "block-D"], // 顶级块 (父级为 null)
  "blocks": {
    "block-A": {
      "id": "block-A",
      "type": "markdown",
      "content": "# 分析帕尔默群岛企鹅数据...",
      "parent": null,
      "children": ["block-B"]
    },
    "block-B": {
      "id": "block-B",
      "type": "markdown",
      "content": "## 1. 数据加载...",
      "parent": "block-A",
      "children": ["block-C"]
    },
    "block-C": {
      "id": "block-C",
      "type": "code",
      "content": "import pandas as pd...",
      "parent": "block-B",
      "children": [],
      "metadata": {
        "language": "python"
      }
    }
    // ... 其他块
  }
}
```
这种结构允许 UI 框架轻松地重建文档的层级结构，并按 ID 访问每个块的内容和元数据。

## 4. 层级结构管理 API (Hierarchy Management API)

为了方便地操作文档的逻辑树结构，`DocHandle` 需要提供以下辅助函数。这些函数是对 `automerge` 文档的封装，使其更易于使用。

- `get_children(block_id: &str) -> Vec<String>`: 返回指定 `block_id` 的所有直接子块的 ID 列表。它通过扫描所有块并匹配其 `metadata.parent` 字段来实现。
- `get_parent(block_id: &str) -> Option<String>`: 返回指定块的父块 ID。
- `move_block(block_id: &str, new_parent_id: &str)`: 将一个块移动到新的父块下。这实际上是修改目标块的 `metadata.parent` 字段。

## 5. 测试要点 (Testing Points)

- **单元测试 (Unit Tests)**:
    - **`Repo` 管理**:
        - 测试 `Repo::get_or_create_handle` 能否为新的 URL 创建 `DocHandle`，并为已存在的 URL 返回相同的实例。
    - **`DocHandle` 状态**:
        - **本地变更**: 调用 `handle.change()` 后，验证 `automerge` 文档状态是否已更新，以及是否广播了通知。
        - **远程变更**: 调用 `handle.load_incremental()` 后，验证状态是否更新，以及是否广播了通知。
    - **并发控制**:
        - 在多个 `tokio` 任务中同时对同一个 `DocHandle` 调用 `change()`，验证 `Mutex` 是否能防止数据竞争，并确保最终状态是一致的。
    - **层级结构 API**:
        - 创建一个具有三层嵌套的文档结构。
        - 调用 `get_children` 和 `get_parent`，验证返回的 ID 是否正确。
        - 调用 `move_block` 将一个子树移动到另一个节点下，然后再次验证层级结构的正确性。
    - **JSON 序列化**:
        - 使用一个包含所有特性（嵌套、元数据、不同块类型）的文档调用 `get_document_state`，并将其输出与一个预先定义好的、正确的 JSON 字符串进行比较。
    - **冲突处理**:
        - 模拟两个客户端并发修改同一个 `code` 块的 `content`。
        - 在同步后，验证该块的 `metadata` 中是否出现了 `conflict: true` 标志。
        - 验证 `doc.get_conflicts()` 是否能返回两个不同的代码版本。
- **集成测试 (Integration Tests)**:
    - **端到端同步**:
        - 设置两个 `Repo` 实例，模拟两个客户端。
        - 客户端 A 创建一个文档并进行一些更改。
        - 验证客户端 B 能否通过 Zenoh 订阅接收到这些变更并应用它们，使得两个客户端的文档状态最终完全一致。
        - **冲突解决**: 让两个客户端同时修改同一个块的同一行文本。验证 `automerge` 是否能自动合并变更，并且两个客户端最终收敛到相同的状态。
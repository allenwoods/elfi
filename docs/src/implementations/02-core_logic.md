# Implementation: 02 - Core Logic (`elfi-core`)

This document details the internal data structures and workflows for the `elfi-core` library, which orchestrates the data model, synchronization, and API layers.

## 1. Core Data Structures

To manage concurrency and state effectively in an `async` environment, we will use `tokio` and several concurrent data structures.

### `Repo` Struct

The `Repo` is the main entry point for an application. It manages the network session and all active document handles.

```rust
use dashmap::DashMap;
use std::sync::Arc;
use zenoh::Session;

pub struct Repo {
    zenoh_session: Arc<Session>,
    doc_handles: DashMap<String, Arc<DocHandle>>,
}
```
- `zenoh_session`: A shared reference to the active Zenoh session for all network operations.
- `doc_handles`: A thread-safe hash map (`DashMap`) mapping a document's URL (as a String) to its corresponding `DocHandle`.

### `DocHandle` Struct

A `DocHandle` manages the state and synchronization for a single document.

```rust
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use automerge::AutoCommit;

pub struct DocHandle {
    doc_url: String,
    doc: Arc<Mutex<AutoCommit>>,
    subscribers: broadcast::Sender<()>, // Notifies listeners of any change
}
```
- `doc_url`: The unique identifier for the document.
- `doc`: The in-memory representation of the document, wrapped in a `tokio::sync::Mutex` to ensure safe concurrent access.
- `subscribers`: A `tokio::sync::broadcast` channel. A message is sent on this channel whenever the document state changes, whether from a local edit or a remote update. UI layers will subscribe to the receiver end of this channel.

## 2. Core Workflows

### Local Change Workflow (`handle.change()`)

This workflow describes what happens when a user modifies a document.

1.  **Acquire Lock**: The application calls `handle.change()`. The first step is to asynchronously acquire a lock on the `DocHandle`'s `Mutex<AutoCommit>`.
2.  **Execute Closure**: The user-provided closure is executed. It receives a mutable reference to the `AutoCommit` document and performs its edits (e.g., inserting text, changing metadata).
3.  **Get Changes**: When the closure finishes and the lock is released, we call `doc.get_last_local_changes()` to retrieve the binary-encoded changes that were just made.
4.  **Broadcast Locally**: `subscribers.send(())` is called to immediately notify any local listeners (like a UI) that the document has been updated.
5.  **Publish Remotely**: The binary changes are published to the appropriate Zenoh key-space (e.g., `/elfi/docs/{doc_url}/ops`) for remote peers to receive.

### Remote Change Workflow (Zenoh Subscription Task)

Each `DocHandle` will spawn a dedicated `tokio` task to listen for incoming changes from the network.

1.  **Subscribe**: The task subscribes to the document's Zenoh key-space.
2.  **Listen**: It asynchronously awaits incoming messages.
3.  **Acquire Lock**: When a message containing remote changes is received, the task acquires a lock on the `DocHandle`'s `Mutex<AutoCommit>`.
4.  **Apply Changes**: The changes are applied to the local model using `doc.load_incremental(&remote_changes)`.
5.  **Broadcast Locally**: `subscribers.send(())` is called to notify local listeners of the update, triggering UI refreshes.

## 3. Tangle API JSON Structure (`get_document_state`)

To facilitate rendering in a decoupled UI, the `get_document_state` function will return a JSON object with a predictable, Lexical-inspired structure.

```json
{
  "root_block_ids": ["block-A", "block-D"], // Top-level blocks (parent is null)
  "blocks": {
    "block-A": {
      "id": "block-A",
      "type": "markdown",
      "content": "# Analysing Palmer Archipelago Penguin Data...",
      "parent": null,
      "children": ["block-B"]
    },
    "block-B": {
      "id": "block-B",
      "type": "markdown",
      "content": "## 1. Data Loading...",
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
    // ... other blocks
  }
}
```
This structure allows a UI framework to easily reconstruct the document's hierarchy and access the content and metadata for each block by its ID.

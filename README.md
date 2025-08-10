# ELFI: Event-sourcing Literate File Interpreter

This document provides a summary of the `elfi` project, based on its technical design document. It serves as a persistent context for the Gemini/Claude agent to understand the project's goals, architecture, and terminology.

## Project Vision

**`elfi` (Event-sourcing Literate File Interpreter)** is a new literate programming paradigm built around the `.elf` file format. It is designed from the ground up for native, decentralized collaboration to overcome the limitations of existing tools like Jupyter Notebooks, LaTeX, and Org-mode.

The goal is to create a powerful, transparent, and efficient medium for human-computer collaboration in software engineering and scientific research.

## Core Principles

- **Parser-First & Human-Readable:** `.elf` is a pure text format that is both human-friendly and easily parsable into a rich, structured in-memory data model. It is designed to be friendly to version control systems like Git.
- **Native Collaboration:** The data model is built on **Conflict-Free Replicated Data Types (CRDTs)**, enabling seamless concurrent and offline editing while guaranteeing eventual consistency.
- **Event Sourcing:** A document is not a static file but a replayable, immutable log of all operations (changes). This provides ultimate transparency, auditability, and a powerful version history.
- **Decentralized & Network-Agnostic:** The architecture supports various network topologies (P2P, client-server, mesh) and is not dependent on a single central server, ensuring data sovereignty.

## System Architecture

The `elfi` system is composed of several distinct layers, implemented in a modular Rust kernel.

### 1. Data Model (CRDT Core)

- **Foundation:** Built on **Event Sourcing** and an **Automerge-inspired CRDT** model. Unlike performance-focused CRDTs (like Yjs), it retains the full, immutable operation history.
- **Structure:** A document is a flat list of "Block" objects. Each block is a CRDT Map (`id`, `type`, `content`, `metadata`).
- **Hierarchy:** Rich hierarchical structures (e.g., chapters, sections) are represented using an **Adjacency List Model**. A `parent` ID is stored in a block's `metadata`, creating logical nesting on top of a flat data structure. This simplifies concurrent move operations.
- **Conflict Resolution:** Implements a block-aware, semantic conflict resolution strategy. Instead of relying on default CRDT behavior, it can use strategies like 3-way merge for code or expose conflicts to the UI for manual resolution.

### 2. Storage & Sync (Zenoh Network)

- **Protocol:** Uses **Eclipse Zenoh** as a unified pub/sub, distributed query, and storage network.
- **Mechanism:** CRDT operations are published as messages to a unique key-space for each document (e.g., `/elf/docs/<doc_uuid>/ops`). Clients subscribe to this key-space for real-time updates and query it to fetch historical operations.
- **Persistence:** Decoupled via Zenoh's storage backend plugins. Operation logs can be stored in filesystems, RocksDB, InfluxDB, or any custom database without changing the core `elfi` logic.

### 3. Weave API (Content Creation Layer)

- **Purpose:** A high-level API for primary content contributors (programmers, writers).
- **Model:** Implements a **Repository (`Repo`) model**, abstracting away the complexities of networking and storage. Developers interact with a `DocHandle` to modify documents.
- **Functionality:** Provides methods like `change()`, `subscribe()`, `getHistory()`, and `viewAt()` (for time travel), as well as helpers for navigating the logical hierarchy (`getParent`, `getChildren`).

### 4. Tangle API (Interactive Rendering Layer)

- **Purpose:** An API for UI clients that render interactive views of `.elf` documents.
- **Model:** Based on the **Islands Architecture**. The document is rendered as static HTML (the "ocean") with interactive components ("islands") that are hydrated selectively.
- **State Management:** Creates a clear boundary between:
    - **Local/Transient UI State:** Managed inside each island (e.g., a slider's current position during drag).
    - **Global/Persistent State:** The CRDT document, which acts as the shared state bus.
- **Pinning:** Provides a `pinBlockState()` API for an island to commit a significant state change back to the CRDT, making it persistent and shared with collaborators.

### 5. `elfi` Interpreter (Rust Kernel)

- **Implementation:** A headless kernel written in **Rust** for performance and safety.
- **Core Dependencies:**
    - **`tree-sitter`:** For parsing the `.elf` text format into an AST.
    - **`automerge`:** For the in-memory CRDT data model.
    - **`zenoh`:** For all networking and synchronization.
- **Structure:** A Cargo workspace containing modular crates:
    - `elfi-core`: The main library exposing the Weave and Tangle APIs.
    - `elfi-parser`: The Tree-sitter grammar and parsing logic.
    - `elfi-cli`: A command-line tool for offline file management.
    - `elfi-ffi`: A C-compatible FFI layer for bindings to other languages and WebAssembly.
- **Code Execution:** Includes a module to interact with external code execution kernels via the Jupyter protocol.

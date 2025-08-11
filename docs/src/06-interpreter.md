# `.elf` 的解释器实现

## 6.1. 项目概述：elfi——一个基于Rust的无头内核

`.elf`系统的核心引擎被定义为一个名为`elfi`（Event-sourcing Literate File Interpreter）的**无头（headless）内核**。它是一个独立的系统，封装了文档的解析、协作、持久化和执行的核心逻辑，并通过定义良好的API与各种前端和工具进行交互 。之所以选择**Rust**作为实现语言，是基于对性能、内存安全以及生态系统协同性的战略考量。我们依赖的核心组件——Automerge（CRDT实现）、Zenoh（网络同步）和Tree-sitter（解析）——都拥有高质量的、性能卓越的Rust核心库 [1, 4, 24, 25]。将`elfi`本身定义为一个Rust项目，可以最大限度地减少跨语言调用的开销，确保整个系统的健壮性和效率。

`elfi`内核的设计是模块化的，可以被编译为多种目标产物：

-   一个**静态库（`lib.rs`）**，可以被其他Rust应用或通过FFI（Foreign Function Interface）被其他语言（如Python, C#, Swift）的应用嵌入。
-   一个**命令行工具（`main.rs`）**，用于对`.elf`文件进行离线操作，如验证、转换、合并等。
-   一个**WebAssembly（Wasm）模块**，使其核心逻辑能够在浏览器或其他Wasm兼容环境中运行。

## 6.2. 核心依赖与集成策略

`elfi`内核的实现将建立在三个 foundational Rust crates 之上，它们共同构成了系统的技术基石。

-   **解析 (`tree-sitter`)**: 内核将使用`tree-sitter`的Rust绑定 [26] 来处理`.elf`的纯文本格式。我们将为`.elf`设计一个明确的语法，并利用Tree-sitter的工具链来生成一个高效且容错的解析器。当内核加载一个`.elf`文件时，Tree-sitter解析器首先将其转换为一个抽象语法树（AST）。随后，内核会遍历这个AST，根据树的结构来初始化第二章中定义的Automerge CRDT文档模型，**这包括正确解析并设置每个块的`metadata`（含`parent`字段）**。
-   **数据模型 (`automerge`)**: 内核将直接嵌入官方的`automerge` Rust crate [4, 27] 作为所有文档在内存中的权威表示。所有对文档的修改、合并、历史查询等操作都将通过`automerge`库的API来完成。内核负责将来自网络的CRDT操作应用到内存中的文档实例，并将本地产生的更改序列化以供网络层发送。
-   **同步 (`zenoh`)**: 内核将集成官方的`zenoh` Rust crate [24, 28] 来处理所有的网络通信和存储交互。内核的配置将包括Zenoh的配置，例如连接的路由器地址、选择的持久化后端等。它将使用Zenoh的API来发布本地生成的CRDT操作，并订阅来自远程协作者的操作流。

## 6.3. 项目结构：一个模块化的Cargo工作空间

为了有效管理这个多方面系统的复杂性，并遵循Rust社区的最佳实践 [29]，`elfi`项目将被组织成一个**Cargo工作空间（workspace）**，其根目录包含一个虚拟清单（virtual manifest） [30, 31]。这种结构允许多个相关的crate共享同一个`Cargo.lock`文件和`target`目录，确保了依赖的一致性和编译效率。

| **Crate 名称** | **描述**                                                     | **主要依赖**                                              |
| -------------- | ------------------------------------------------------------ | --------------------------------------------------------- |
| `elfi-core`    | 核心库crate，包含所有核心逻辑、数据结构，并暴露公共的Weave和Tangle API。这是整个系统的核心。 | `automerge`, `zenoh`, `elfi-parser`, `thiserror`, `tokio` |
| `elfi-parser`  | 一个专门的库crate，包含为`.elf`格式编写的Tree-sitter语法，以及将文本解析为AST并将其转换为初始CRDT状态的逻辑。 | `tree-sitter`                                             |
| `elfi-cli`     | 一个二进制crate，提供一个命令行工具，用于离线检查、创建、转换和管理`.elf`文件。 | `elfi-core`, `clap`, `anyhow`                             |
| `elfi-ffi`     | 一个`cdylib` crate，为`elfi-core`暴露一个C兼容的外部函数接口（FFI）。这为绑定到其他语言（如Python, Node.js）或编译到WebAssembly提供了基础。 | `elfi-core`                                               |

## 6.4. `elfi-core` 的核心接口与模块

`elfi-core` crate是整个系统的核心，其内部模块结构和暴露的公共API经过精心设计，以清晰地分离关注点。

### 6.4.1. 模块结构

```
// elfi-core/src/lib.rs

// 公共API模块
pub mod doc;      // 对应Weave层API，用于内容创作和协作
pub mod render;   // 对应Tangle层API，用于渲染和执行
pub mod error;    // 定义库的公共错误类型

// 内部实现模块
mod sync;     // 管理Zenoh会话和网络同步逻辑
mod store;    // 管理内存中的Automerge文档实例集合
mod exec;     // 管理代码执行环境的交互（如Jupyter协议）
```

### 6.4.2. 暴露的接口

-   **Weave API (`pub mod doc`)**:

    -   这个模块将暴露与第四章定义的Weave API相对应的Rust结构体和方法。
    -   `pub struct Repo`: 管理与Zenoh网络的连接和所有文档句柄。
    -   `pub struct DocHandle`: 单个文档的句柄，提供`change`, `subscribe`, `get_history`等方法。
    -   这个API是为需要深度集成、直接读写文档内容的工具（如IDE插件、原生桌面编辑器）设计的。

-   **Tangle API (`pub mod render`)**:

    -   这个模块将暴露与第五章定义的Tangle API相对应的函数，主要面向需要渲染交互式视图的前端或UI客户端。
    -   `pub async fn get_document_state(doc_url: &str) -> Result<serde_json::Value, ElfiError>`: 获取文档的JSON渲染快照。
    -   `pub async fn pin_block_state(doc_url: &str, block_id: &str, metadata_patch: serde_json::Value) -> Result<(), ElfiError>`: 实现“钉合”机制。
    -   `pub fn execute_code_block(doc_url: &str, block_id: &str) -> impl Stream<Item = Result<ExecutionEvent, ElfiError>>`:
        -   此函数将与`exec`模块交互，该模块负责管理与外部代码执行内核的通信。
        -   `exec`模块将使用一个现有的Jupyter协议客户端库（如`jupyter-protocol` [32, 33]）来连接到一个标准的Jupyter内核（如IPython）。
        -   它将代码块的内容通过Jupyter消息协议发送给内核执行，并将内核返回的输出（`stdout`, `display_data`等）封装成`ExecutionEvent`枚举，通过异步流返回给调用者。

-   **错误处理 (`pub mod error`)**:

    -   这个模块将定义整个库的公共错误类型。遵循Rust库设计的最佳实践，我们将使用`thiserror` crate来创建一个结构化的、详尽的`enum ElfiError` [34, 35]。

    ```
    use thiserror::Error;
    
    #
    pub enum ElfiError {
        #[error("Network error: {0}")]
        Network(#[from] zenoh::Error),
    
        #
        DataModel(#[from] automerge::AutomergeError),
    
        #[error("Parsing error: {0}")]
        Parse(String), // 来自elfi-parser的错误
    
        #
        DocumentNotFound(String),
    
        #[error("Execution error: {0}")]
        Execution(String),
    }
    ```

    -   这种设计为`elfi-core`的消费者提供了强大的能力，使其可以编程方式地匹配和处理不同类型的故障，这对于构建健壮的应用程序至关重要。与之相对，`elfi-cli`则会使用`anyhow`来包装来自`elfi-core`的`ElfiError`，并添加上下文信息，以便向最终用户提供清晰的错误报告 [36]。

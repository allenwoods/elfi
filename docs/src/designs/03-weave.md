# Weave：编写 `.elf`  文件

## 4.1. 设计原则：为内容创作者提供一个仓库模型

在建立了基于CRDT的数据模型和基于Zenoh的通信网络之后，我们需要一个高层次、对开发者友好的API来封装底层的复杂性。Weave层正是为此而设计。它面向文档内容的**主要贡献者**——作家、研究员、程序员和技术文档工程师——提供了一套用于编写和协作编辑文档“静态内容”（如文本和代码）的接口。

直接操作CRDT和发布/订阅协议对于应用开发者来说过于底层和复杂。因此，Weave层的核心设计原则是采用**仓库（Repository）模型**。该模型借鉴了`automerge-repo` [1, 13] 和Yjs Providers  等“全家桶”式协作框架的设计模式，将网络、存储和文档状态管理打包成一个易于使用的抽象层。

这种模型有效地将**文档管理**（发现、加载、同步）的关注点与**文档修改**的关注点分离开来。开发者通过一个简单的、面向对象的API与系统交互，而CRDT状态转换、序列化、网络消息收发等复杂机制则被完全封装。其核心组件包括：

-   **Repo（仓库）**：作为应用程序中管理协作状态的中心对象。它负责初始化和维护与Zenoh网络的连接，管理存储适配器，并持有一系列当前活跃的文档。
-   **DocHandle（文档句柄）**：作为单个`.elf`文档的引用或代理。开发者不直接持有文档对象本身，而是通过句柄来操作。句柄负责管理文档的整个生命周期，包括从网络或本地存储加载、处理进出的同步消息、触发更新事件，并提供修改文档的方法。

## 4.2. Weave API 规范

Weave API的设计旨在简洁和直观，同时充分利用底层全历史CRDT模型的强大能力，提供类似Git的版本控制功能和对层级结构的操作能力。

| **函数签名**                                              | **描述**                                                     |
| --------------------------------------------------------- | ------------------------------------------------------------ |
| `repo.create() -> DocHandle`                              | 创建一个新的、空的`.elf`文档。此操作会在本地生成一个初始的CRDT状态，并为其分配一个唯一的URL。它会立即返回一个指向这个新文档的句柄。 |
| `repo.load(docUrl: String) -> DocHandle`                  | 根据一个唯一的文档URL（该URL映射到Zenoh的键空间）加载一个已存在的文档。此函数会返回一个句柄，并开始在后台异步地从网络和/或本地存储中获取文档的历史操作并构建其状态。 |
| `handle.change(callback: (doc: MutableDoc) => void)`      | 对文档进行所有修改的唯一入口。框架会将一个可变的、代理版本的文档状态传入`callback`中。开发者在此函数内部对代理对象进行的所有修改都会被记录下来，并在函数执行完毕后原子性地打包成CRDT操作广播出去。 |
| `handle.subscribe(callback: (doc: ImmutableDoc) => void)` | 为文档句柄注册一个监听器。每当该文档的状态因本地修改或接收到远程更新而发生变化时，注册的回调函数就会被调用，并传入新的、不可变的文档状态。这是UI层响应数据变化、进行重新渲染的主要机制。 |
| `handle.getHistory() -> HistoryGraph`                     | 返回构成文档历史的完整的、有向无环图（DAG）结构的操作日志。这使得高级的版本可视化和分析成为可能。 |
| `handle.viewAt(heads: ChangeHash) -> ImmutableDoc`        | 根据一组历史变更哈希，返回一个该文档在那个特定时间点的只读视图。这是实现“时间旅行”和版本回退功能的基础 。 |
| `handle.getParent(blockId: String) -> Option<Block>`      | 获取指定块的父块（如果存在）。                               |
| `handle.getChildren(blockId: String) -> Vec<Block>`       | 获取指定块的所有直接子块。                                   |
| `handle.getTree() -> Tree<Block>`                         | 将整个文档的扁平块列表重构为一个树状结构并返回。             |
| `handle.createLink(target: String, ref_type: LinkType) -> Block` | 创建一个Link Block，指向指定的URI目标。                  |
| `handle.resolveLink(blockId: String) -> Promise<Block>`   | 解析Link Block的目标内容，返回引用的区块。          |
| `handle.validateLinks() -> Vec<LinkValidation>`           | 验证文档中所有Link Block的引用完整性。              |
| `handle.createRecipe(config: RecipeConfig) -> Block`      | 创建一个Recipe区块，包含指定的转换配置。           |
| `handle.executeRecipe(blockId: String) -> Promise<Output>` | 执行Recipe并返回转换结果。                          |
| `handle.validateRecipe(blockId: String) -> RecipeValidation` | 验证Recipe配置的语法和语义正确性。               |

新增的层级操作API（`getParent`, `getChildren`, `getTree`）封装了遍历CRDT列表并检查`parent`元数据字段的逻辑。它们为上层应用提供了一个自然的、面向树的编程模型，而无需关心底层的扁平存储实现。

## 4.3. 块级类型的Weave接口

为了提供更丰富和类型安全的编辑体验，Weave API并不会将所有块的内容都暴露为通用的文本。相反，在`handle.change`回调中访问块对象时，其`.content`属性会根据块的`type`提供不同的、专门化的API。

这种设计通过多态性为不同类型的内容提供了最自然的编辑原语，极大地提升了开发者的体验和代码的安全性。

-   **对于 `type: "markdown"` 或 `type: "code"` 的块**： 当开发者访问这类块的`content`属性时，他们会得到一个实现了**Text API**的对象。这个API提供了细粒度的文本操作方法，这些方法会直接映射到底层的Text CRDT操作。

    ```
    handle.change(doc => {
      const codeBlock = doc.blocks; // 假设这是一个code块
      //.content 提供了文本操作接口
      codeBlock.content.insert(0, "function hello() {\n");
      codeBlock.content.delete(20, 1); 
      codeBlock.content.insert(20, "}");
    });
    ```

-   **对于其他自定义类型的块**： 系统可以为其他类型的块提供专门的接口。例如，一个`type: "counter"`的块，其内容可能是一个CRDT计数器。

    ```
    handle.change(doc => {
      const counterBlock = doc.blocks; // 假设这是一个counter块
      //.content 提供了计数器操作接口
      counterBlock.content.increment(5);
    });
    ```

## 4.4. 在Weave层处理冲突

Weave API的设计旨在将底层CRDT的冲突信息清晰地暴露给应用程序，并提供一个明确的工作流来让用户参与语义冲突的解决。这与默默地丢弃冲突信息的模型形成了鲜明对比。

### 4.4.1. 冲突发现API

Weave层将提供一个专门的API来查询特定属性上的并发写入冲突。

-   **`handle.getConflicts(path: Path) -> Map<OpId, Value> | undefined`**:
    -   **描述**: 此函数接收一个路径（`Path`），该路径指向文档状态树中的一个特定属性（例如 `['blocks', 0, 'content']`）。
    -   **返回值**: 如果该属性上存在由并发操作引起的多个值，函数将返回一个`Map`对象。这个`Map`的键是导致冲突的每个操作的唯一ID（`OpId`），值是该操作写入的具体值（`Value`）。如果该属性没有冲突，则返回`undefined`。
    -   **依据**: 这个API设计直接源于Automerge等全历史CRDT提供的能力，它们在合并时不会丢弃任何信息，而是保留所有并发写入的值，并通过类似`getConflicts`的接口供上层查询 [1, 7]。

### 4.4.2. 冲突解决工作流

结合`getConflicts` API，应用程序可以实现一个强大的、用户驱动的冲突解决流程：

1.  **检测与呈现**：

    -   应用程序通过`handle.subscribe`监听文档的任何变化。
    -   在回调函数中，当检测到文档状态更新（特别是合并了远程更改后），应用程序可以主动调用`handle.getConflicts`来检查关键属性是否存在冲突。
    -   例如，在处理一个代码块时，UI代码可以检查`handle.getConflicts(['blocks', blockIndex, 'content'])`。
    -   如果返回了一个包含多个值的`Map`，UI层可以将这些不同的版本呈现给用户。例如，并排显示两个版本的代码，并询问用户：“Alice和Bob同时修改了这段代码，请选择要保留的版本，或手动合并它们。”

2.  **解决与提交**：

    -   一旦用户做出了选择（例如，选择了Bob的版本，或者手动编辑了一个合并后的新版本），应用程序将获取这个最终的、被认可的内容。
    -   然后，应用程序会发起一次**新的 `handle.change` 操作**，将这个最终内容写入到之前发生冲突的属性路径上。

    ```
    // 假设用户选择了Bob写入的值
    const finalContent = bobVersion; 
    
    handle.change(doc => {
      doc.blocks[blockIndex].content.replaceAll(finalContent);
    });
    ```

    -   这次新的`change`操作会在CRDT的因果历史中创建一个新的节点，该节点将所有先前冲突的“头”（heads）作为其父节点。这就在语义上解决了冲突，并确保所有协作者的文档状态都会收敛到这个由用户确认的、唯一的最终状态。

这个工作流清晰地划分了职责：CRDT负责在合并期间**无损地保存所有信息**；Weave API负责将这些冲突信息**透明地暴露给应用程序**；最终，应用程序和用户负责运用**领域知识和语义理解**来做出最终的裁决。

## 4.5. 使用外部工具进行编辑

### 4.5.1. IDE集成架构

Weave层支持与传统IDE的双向集成，允许开发者使用熟悉的编辑器：

- **导出机制**：将`.elf`文档导出为传统项目结构
- **文件监听**：实时监控导出文件的变更
- **同步条件**：确定哪些修改可以安全地同步回`.elf`
- **冲突处理**：处理外部编辑与内部变更的冲突

### 4.5.2. `elfi watch`命令集成

基于quickstart.md中定义的实际命令，IDE集成通过`elfi watch`实现：

```bash
# 启动文件监听服务
elfi watch --project my-project --export-dir ./exported/

# 监听特定类型的区块
elfi watch --types code,markdown --sync-mode bidirectional
```

#### 双向同步机制

`elfi watch`提供双向同步能力：

1. **导出同步**：`.elf`文档变更时自动更新导出的源文件
2. **导入同步**：源文件变更时自动同步回`.elf`文档
3. **冲突检测**：检测并处理双向修改产生的冲突
4. **实时反馈**：在IDE中显示同步状态和冲突提示

### 4.5.3. 同步条件与验证

为确保数据完整性，IDE同步需要满足以下条件：

1. **单区块映射**：被修改的文件必须对应单个区块的完整导出
2. **结构一致性**：文件路径和名称必须与导出时保持完全一致
3. **时间窗口**：修改必须在合理时间窗口内发生，避免过期同步
4. **完整性检查**：文件内容必须通过语法和结构验证
5. **Recipe依赖**：如果区块被Recipe引用，需要检查依赖完整性

### 4.5.4. 监听与同步实现

```rust
// elfi watch 命令的核心实现
impl WatchService {
    async fn handle_file_change(&self, path: PathBuf, content: String) {
        // 1. 验证文件是否为有效的同步目标
        if let Some(block_mapping) = self.get_block_mapping(&path) {
            // 2. 检查同步条件
            if self.validate_sync_conditions(&block_mapping, &content) {
                // 3. 检查Recipe依赖影响
                let affected_recipes = self.find_dependent_recipes(&block_mapping.block_id);
                
                // 4. 应用变更到对应区块
                self.apply_change_to_block(block_mapping.block_id, content).await;
                
                // 5. 重新执行受影响的Recipe
                for recipe_id in affected_recipes {
                    self.queue_recipe_execution(recipe_id).await;
                }
            } else {
                // 6. 报告同步冲突
                self.report_sync_conflict(&path, &content);
            }
        }
    }
    
    async fn start_watch_mode(&self, config: WatchConfig) {
        // 启动文件监听
        let watcher = notify::recommended_watcher(move |res| {
            // 处理文件系统事件
        }).unwrap();
        
        // 同时启动Zenoh订阅以接收远程变更
        self.subscribe_to_document_changes().await;
    }
}


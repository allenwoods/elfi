# Weave：编写 `.elf`  文件

## API设计边界说明 ⚠️

**重要：Weave API 设计为通用接口，不依赖特定的块类型或业务逻辑。**

### Weave API 职责范围
- **文档操作**：提供统一的文档创建、加载、修改接口
- **CRDT 抽象**：封装底层 CRDT 操作，提供直观的编程模型
- **历史管理**：支持时间旅行、差异查看等版本控制功能
- **冲突处理框架**：提供冲突检测和解决的通用机制
- **关系操作**：提供通用的关系创建、查询、验证接口

### Weave API 不负责的范围
- **类型语义解释**：不定义 `markdown`、`code` 等类型的具体行为
- **内容格式验证**：不验证代码语法、Markdown 格式等
- **特定冲突策略**：不内置领域特定的冲突解决逻辑
- **渲染逻辑**：不负责内容的可视化展示

### 插件化扩展原则
文档中提到的类型处理（如 `markdown`、`relations`）都是通过插件系统实现：

- **类型处理器**：插件可以注册特定类型的 `content` 处理逻辑
- **冲突解决器**：插件可以提供自定义的冲突解决策略
- **验证器**：插件可以实现类型特定的内容验证
- **转换器**：插件可以提供内容格式转换功能

这确保 Weave API 保持通用性，同时支持无限扩展。

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
| `handle.createRelation(source: String, target: String, relType: String, props?: Object) -> void` | 在Relations Block中创建新的关系。 |
| `handle.removeRelation(source: String, target: String) -> void` | 从Relations Block中移除指定关系。 |
| `handle.getRelations(blockId?: String) -> Vec<Relation>` | 获取所有关系或指定块的相关关系。 |
| `handle.validateRelations() -> Vec<RelationValidation>` | 验证Relations Block中所有关系的完整性。 |
| `handle.createRecipe(config: RecipeConfig) -> Block`      | 创建一个Recipe区块，包含指定的转换配置。           |
| `handle.executeRecipe(blockId: String) -> Promise<Output>` | 执行Recipe并返回转换结果。                          |
| `handle.validateRecipe(blockId: String) -> RecipeValidation` | 验证Recipe配置的语法和语义正确性。               |

新增的层级操作API（`getParent`, `getChildren`, `getTree`）封装了遍历CRDT列表并检查`parent` attributes字段的逻辑。Relations管理API（`createRelation`, `removeRelation`, `getRelations`, `validateRelations`）提供了统一的关系管理接口，支持跨文档引用和复杂关系操作。它们为上层应用提供了一个自然的、面向关系的编程模型，而无需关心底层的扁平存储实现。

## 4.3. 块级类型的Weave接口

⚠️ **重要说明**：以下描述的类型特定接口都是**插件系统提供的示例**，不是 Weave API 的内置功能。

为了提供更丰富的编辑体验，插件可以扩展 Weave API，为不同的用户定义类型提供专门化的接口。在`handle.change`回调中，插件可以根据块的`type`提供不同的内容操作API。

这种插件化设计通过多态性为不同类型的内容提供了最自然的编辑原语，同时保持了 Weave API 的通用性。

-   **用户约定类型 `"markdown"` 或 `"code"`（插件提供）**： 当项目使用文本类型的块时，插件可以提供 **Text API** 扩展：

    ```
    // 插件扩展的示例，不是 Weave API 内置功能
    handle.change(doc => {
      const codeBlock = doc.blocks[0]; // 假设项目约定这是code块
      // 插件提供的 .content 文本操作接口
      codeBlock.content.insert(0, "function hello() {\n");
      codeBlock.content.delete(20, 1); 
      codeBlock.content.insert(20, "}");
      
      // 标准的 .attributes 属性访问（Weave API 内置）
      codeBlock.attributes.language = "javascript";
      codeBlock.attributes.author = "alice";
    });
    ```

-   **用户约定类型 `"relations"`（插件提供）**： 项目如果使用关系管理，插件可以提供专门的关系操作API：

    ```
    // 插件扩展的示例，不是 Weave API 内置功能
    handle.change(doc => {
      const relationsBlock = doc.blocks.find(b => b.type === "relations");
      // 插件提供的关系操作接口
      relationsBlock.content.addRelation("source", "target", "references", {display_text: "链接"});
      
      // 标准的 .attributes 操作（Weave API 内置）
      relationsBlock.attributes.owner = "alice";
      relationsBlock.attributes.merge_method = "manual";
    });
    ```

-   **项目自定义类型（插件提供）**： 插件可以为任意自定义类型提供专门的接口：

    ```
    // 用户自定义类型的插件示例
    handle.change(doc => {
      const customBlock = doc.blocks[0]; // 假设项目定义了custom_type
      // 插件提供的自定义操作接口
      customBlock.content.customMethod(params);
      
      // 标准的 .attributes 操作（Weave API 内置）
      customBlock.attributes.project_specific_attr = "value";
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

1.  **检测与呈现**：应用程序通过订阅机制监听文档状态变化，主动检测关键属性的冲突情况，并通过直观的UI界面向用户呈现不同的并发修改版本，帮助用户理解冲突的性质和范围。

2.  **解决与提交**：用户做出决策后，应用程序通过新的修改操作将最终内容提交到CRDT中。这种机制在因果历史中创建一个新节点，将所有冲突的分支合并为一个一致的状态，确保所有协作者都收敛到用户确认的最终结果。

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

### 4.5.5. 监听服务架构设计

`elfi watch`命令的核心设计基于以下架构原理：

**双向同步机制**：同时监听文件系统变更和远程文档变更，实现真正的双向同步
**条件验证策略**：确保只有满足安全条件的修改才能同步，避免数据不一致
**依赖传播处理**：自动检测和处理Recipe依赖关系，确保相关内容的一致性
**冲突优雅处理**：提供清晰的冲突报告和解决建议，保持用户工作流的连续性


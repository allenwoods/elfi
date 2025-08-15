# 使用场景概览

本章节包含的文档描述了 `elfi` 的三个核心使用场景。这些场景不仅是功能的展示，更体现了一种能力上的递进关系，共同构成了 `elfi` 的核心价值主张。

## 📁 测试文件总览

每个使用场景都配备了完整的 `.elf` 测试文件：

| 使用场景 | 文档描述 | 测试文件 | 核心功能验证 |
|---------|---------|----------|-------------|
| **对话即文档** | [02-conversation-as-document.md](./02-conversation-as-document.md) | [conversation.elf](./conversation.elf) | CRDT实时协作与冲突解决 |
| **自举** | [01-bootstrapping.md](./01-bootstrapping.md) | [elfi-dev.elf](./elfi-dev.elf) | 代码管理与Recipe导出系统 |
| **文档即App** | [03-document-as-app.md](./03-document-as-app.md) | [main.elf](./main.elf) + [component.elf](./component.elf) | 跨文档引用与动态组合 |

三个场景之间的关系如下：

### 1. [对话即文档](./02-conversation-as-document.md) - 基础能力

这是 `elfi` 的基石。它首先确立了系统的核心技术：基于 CRDT 和 Zenoh 的实时、无冲突、多用户协作能力。没有这个基础，其他一切都无从谈起。这个场景回答了最基本的问题：“我们能否让多个人在同一个文档上无缝协作？”

- **关键词**: 协作，同步，冲突解决，CRDT，基础架构。

### 2. [自举](./01-bootstrapping.md) - 应用与验证

这是对基础能力的第一个重要应用和验证。它将“文档”的概念从简单的文本扩展到了高度结构化的“代码”。通过使用 `elfi` 来管理和编辑其自身的源代码，我们证明了其协作核心的鲁棒性和实用性足以应对软件工程这样复杂的任务。这个场景回答了：“我们的协作能力是否强大到足以支撑我们自己的开发？”

- **关键词**: 结构化数据，代码管理，元编程，自我迭代。

### 3. [文档即 App](./03-document-as-app.md) - 组合与创造

这是最高阶的应用场景，它建立在前两个场景的能力之上。一旦我们能够可靠地协作编辑复杂的、结构化的文档（如代码），下一步自然就是将这些独立的文档动态地组合起来，创造出比各部分总和更强大的东西——一个完整的、动态的应用。通过 Recipe 系统的跨文档引用能力，`elfi` 将文档从静态的信息载体转变为动态的功能模块。

- **关键词**: Recipe驱动的内容组合，跨文档引用，动态组合，模块化，应用构建。


总之，这三个场景从**"实现协作"**到**"应用协作"**，再到**"升华协作"**，清晰地展示了 `elfi` 的设计哲学和发展路径。特别是 Recipe 系统作为核心的内容组合和转换引擎，不仅支持了场景2的代码管理需求，更是实现场景3中动态内容组合的关键技术。

## 🤖 开发指引与 Subagent 使用

### 推荐的 Subagent 配置

以下是针对每个开发阶段的专业 subagent 推荐：

| Subagent | 专业领域 | 主要职责 | 适用模块 |
|----------|----------|----------|----------|
| **rust-tdd-developer** | TDD开发 | 测试驱动开发、单元测试、代码质量保证 | types, 通用测试 |
| **crdt-specialist** | CRDT专家 | 事件溯源、冲突解决、Automerge集成 | core |
| **parser-expert** | 解析器专家 | Tree-sitter语法、.elf解析、错误处理 | parser |
| **network-architect** | 网络架构师 | Zenoh集成、分布式同步、P2P网络 | storage |
| **api-designer** | API设计师 | 接口设计、内容创作API、关系管理 | weave, tangle, recipe, extension |
| **integration-tester** | 集成测试师 | 端到端测试、性能基准、故障模拟 | 集成测试 |
| **cli-ux-specialist** | CLI专家 | 用户体验、命令行设计、配置管理 | cli |
| **docs-maintainer** | 文档维护师 | 技术文档、API文档、用户指南 | 文档维护 |

### 🎯 快速Subagent选择指南

```mermaid
graph LR
    A[开发任务] --> B{任务类型}
    B -->|数据结构/接口| C[@rust-tdd-developer]
    B -->|.elf解析| D[@parser-expert]
    B -->|CRDT/事件溯源| E[@crdt-specialist] 
    B -->|网络同步| F[@network-architect]
    B -->|API设计| G[@api-designer]
    B -->|CLI工具| H[@cli-ux-specialist]
    B -->|测试集成| I[@integration-tester]
    B -->|文档维护| J[@docs-maintainer]
```

### 💬 Subagent 调用示例

#### 开始新模块开发
```bash
# 开发基础数据结构
@rust-tdd-developer 请开发 types 模块，定义ELFI的核心数据结构。
参考 plans/04-phase1-a-types.md 中的设计要求，遵循TDD流程：
1. 先在 types/src/interface.rs 中定义公共接口
2. 在 types/tests/ 中编写单元测试
3. 实现 Document、Block、Relation 等核心类型

# 实现CRDT功能
@crdt-specialist 请基于 types 模块的Interface，
实现 core 模块的CRDT功能。参考 plans/04-phase1-c-core.md
中的CRDT设计要求和 docs/src/designs/01-data_modeling.md。
```

#### 用例驱动的开发
```bash
# 基于对话即文档场景开发
@crdt-specialist 请实现 conversation.elf 测试文件中展示的
多用户实时协作功能。重点解决CRDT冲突解决和Zenoh同步。

# 基于自举场景开发
@api-designer 请实现 elfi-dev.elf 中的Recipe系统，
支持代码导出和文件双向同步功能。

# 基于文档即App场景开发
@api-designer 请实现跨文档引用功能，支持 main.elf 
引用 component.elf 的动态组合渲染。
```

#### 集成测试和质量保证
```bash
# 端到端测试
@integration-tester 请基于三大核心用例设计端到端测试：
1. 验证 conversation.elf 的多用户协作场景
2. 验证 elfi-dev.elf 的自举开发场景  
3. 验证 main.elf + component.elf 的跨文档组合场景

# 文档同步
@docs-maintainer 请根据已完成的 core 模块实现，
更新 docs/src/implementations/02-core.md 文档。
```

### ⚡ 快速开发工作流

#### 🚀 开始新任务的3步法

```bash
# 1️⃣ 确定 Subagent - 查看上面的选择指南

# 2️⃣ 调用 Subagent - 使用具体的模块计划文档
@{专业subagent} 请开发 {module} 模块，
参考计划文档 plans/{phase}-{module}.md 中的具体要求。

# 3️⃣ 验证集成 - 确保与用例场景的兼容性
@integration-tester 请验证 {module} 与 {usecase}.elf 的集成效果。
```

### 🔄 常见开发场景

#### 场景1: 新功能开发
基于三个核心用例的需求，优先开发支持用例场景的功能模块。

#### 场景2: 集成调试  
当模块间集成出现问题时，使用 `@integration-tester` 分析和设计测试验证。

#### 场景3: 性能优化
当系统性能不满足用例要求时，使用对应的专业subagent进行优化。

#### 场景4: 文档更新
实现完成后，使用 `@docs-maintainer` 同步更新相关文档。

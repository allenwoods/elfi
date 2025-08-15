# ELFI：事件溯源的文学化文件解释器

本文档基于其技术设计文档，对 `elfi` 项目进行概述。它将作为 Gemini/Claude 代理的持久化上下文，以帮助其理解项目的目标、架构和术语。

## 📖 开发者文档

### 🚀 新开发者入门
如果你是第一次参与ELFI项目，请按顺序阅读：

1. **[ROADMAP.md](ROADMAP.md)** - 项目进度概览和当前里程碑
2. **[DEVELOPMENT.md](DEVELOPMENT.md)** - 环境配置、TDD工作流程和开发规范  
3. **[plans/01-overview.md](plans/01-overview.md)** - 项目架构、模块职责和开发策略
4. **[docs/src/usecases/00-overview.md](docs/src/usecases/00-overview.md)** - 核心用例和Subagent使用指南

### 📚 详细文档
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - 贡献者指南，包含快速开始、项目结构、开发命令等
- **[docs/README.md](docs/README.md)** - 文档构建和维护指南
- **[scripts/README.md](scripts/README.md)** - 自动化脚本使用说明

### 用户文档
- **[快速入门](docs/src/02-quickstart.md)** - 完整工作流演示和核心功能介绍
- **[命令速查表](docs/src/03-cheatsheet.md)** - 所有命令的参考手册

### 技术设计文档
- **[.elf文件格式规范](docs/src/implementations/01-elf_spec.md)** - 文件语法、Block结构、Relations语法定义
- **[数据建模](docs/src/designs/01-data_modeling.md)** - CRDT架构、Block设计、Relations Block、冲突解决策略
- **[存储同步](docs/src/designs/02-storage_sync.md)** - Zenoh网络架构和同步机制
- **[Weave API](docs/src/designs/03-weave.md)** - 内容创作API、关系管理、IDE集成
- **[Tangle API](docs/src/designs/04-tangle.md)** - 交互渲染和Recipe系统
- **[解释器](docs/src/designs/05-interpreter.md)** - Rust内核实现
- **[Extension系统](docs/src/designs/06-extension.md)** - 插件架构设计和生态系统

**重要笔记：**
- **文档结构**: 所有需要通过 `mdbook` 编译的文档源文件（`.md` 文件）都**必须**放置在 `docs/src/` 目录下。`docs/src/SUMMARY.md` 文件定义了最终文档的目录结构。

## 命令设计原则

为保证CLI命令的一致性和可发现性，请遵循以下原则：

### 命令组织原则
- **所有命令必须在[命令速查表](docs/src/03-cheatsheet.md)中列出** - 这是唯一的命令参考文档
- **优先使用二级命令组织相关功能** - 将功能相关的操作归类到同一个一级命令下
- **使用URI统一支持文档和区块操作** - 所有接受目标的命令应使用URI格式而非简单的ID
- **避免无必要的新一级命令** - 优先考虑将新功能作为现有命令的子命令

### URI格式规范
- **完整URI**: `elf://[user/]repo/doc[/block-id]`
- **相对引用**: `./doc/block-id` (同仓库) | `#block-id` (同文档)
- **示例**: 
  - `elf://my-project/doc` - 整个文档
  - `elf://my-project/doc/block-001` - 特定区块
  - `./doc/block-001` - 相对路径

### 命令分类
- **Extension管理**: 使用 `elfi extension <subcommand>` 统一管理
- **权限管理**: 使用 `elfi permission <subcommand>` 统一管理  
- **同步操作**: 使用 `elfi sync [subcommand]` 统一管理
- **核心工作流**: 保留为一级命令（open, add, link, export等）

## 开发规范

### TDD开发流程

**所有开发必须遵循Test-Driven Development流程：**

1. **先写测试，后写实现** - 每个功能开发前必须先编写对应的单元测试
2. **真实接口，Mock依赖** - 测试中使用真实的模块实现，依赖其他模块时使用Interface + Mock
3. **Interface优先原则** - 如果依赖的模块Interface不存在，测试应报错提醒对应开发者实现
4. **迭代开发** - 重复"运行测试→实现功能→运行测试"直到所有测试通过

### 模块边界和接口约定

**严格的模块职责分离：**
- 每个模块只负责自己的核心功能，不得实现其他模块的功能
- 模块间交互必须通过Interface trait，不允许直接依赖具体实现
- 未实现的依赖必须抛出`NotImplemented` error，不能自己实现替代方案

**数据结构共享原则：**
- 所有核心数据结构定义在`elfi-types` crate中
- 禁止在不同模块中重复定义相似的数据结构
- 新增数据类型必须在types模块中统一定义

### 代码质量要求

**测试覆盖率标准：**
- 单元测试覆盖率必须 > 80%
- 所有公共API必须有对应测试
- 边界条件和错误情况必须有测试覆盖

**代码提交规范：**
```bash
# 每次提交前必须运行
just test      # 单元测试
just lint      # 代码格式检查
just typecheck # 类型检查
```

**提交信息格式：**
```
feat(module): 简短描述

- 详细变更内容
- 相关测试覆盖

🤖 Generated with Claude Code
```

### 文档同步要求

**实现完成后的文档更新顺序：**
1. `docs/src/designs/{module}.md` - 设计文档
2. `docs/src/implementations/{module}.md` - 实现文档  
3. `docs/src/03-cheatsheet.md` - 命令参考
4. 如有新的开发注意事项，更新此CLAUDE.md文件

### 避免Over-Engineering原则

- **恰好够用**: 只实现当前需求，不预设未来功能
- **组合优于创新**: 优先通过现有模块组合实现功能
- **集成测试验证**: 通过三大用例的集成测试验证功能完整性

### 错误处理统一规范

```rust
// 使用统一的错误类型
use elfi_types::error::ElfiError;

// 模块未实现时的标准错误
return Err(ElfiError::NotImplemented { 
    module: "elfi-storage".to_string() 
});

// 依赖模块错误的包装
dependency_result.map_err(|e| ElfiError::Dependency { 
    source: e.into() 
})?;
```

### 依赖管理规范

**⚠️ 重要警告: 依赖版本变更管控**

所有依赖相关的修改必须严格管控：

1. **禁止随意修改依赖版本** - 任何对 `Cargo.toml` 中依赖版本的修改都可能导致：
   - API破坏性变更影响现有代码
   - 版本冲突导致编译失败  
   - 安全漏洞或性能回退
   - 跨平台兼容性问题

2. **依赖变更流程**:
   ```
   提议变更 → 技术讨论 → 影响评估 → 测试验证 → 团队确认 → 实施变更
   ```

3. **必须讨论的变更类型**:
   - 主版本升级 (如 `1.x` → `2.x`)
   - 核心依赖变更 (automerge, zenoh, tree-sitter, tokio)
   - 新增依赖库
   - 移除现有依赖

4. **例外情况**:
   - 安全补丁的patch版本更新 (如 `1.0.1` → `1.0.2`)
   - 开发依赖的小版本更新 (dev-dependencies)

5. **变更文档要求**:
   - 详细说明变更原因和预期收益
   - 列出潜在的破坏性变更
   - 提供回滚方案
   - 更新 `plans/03-dependencies.md` 文档

**当前稳定依赖版本** (参考 `plans/03-dependencies.md`):
- automerge = "0.5" (API开发中，需谨慎)
- zenoh = "1.5" (最新稳定版)  
- tree-sitter = "0.25" (最新版本)
- tokio = "1.0" (长期稳定版)

### 性能和安全基线

**性能指标：**
- 文档同步延迟 < 100ms
- 单文档内存使用 < 100MB  
- 支持并发用户数 > 10

**安全要求：**
- 不在代码中硬编码任何密钥或敏感信息
- 不在日志中输出用户敏感数据
- 所有网络通信使用安全协议

## 项目愿景

**`elfi` (Event-sourcing Literate File Interpreter)** 是一种全新的文学化编程范式，围绕 `.elf` 文件格式构建。它从零开始设计，旨在实现原生的、去中心化的协作，以克服现有工具（如 Jupyter Notebooks、LaTeX 和 Org-mode）的局限性。

其目标是为软件工程和科学研究中的人机协作，创造一个强大、透明且高效的媒介。

## 核心原则

- **解析器优先 & 人类可读：** `.elf` 是一种纯文本格式，既对人类友好，又易于解析为丰富的、结构化的内存数据模型。它的设计对 Git 等版本控制系统非常友好。
- **原生协作：** 数据模型建立在**无冲突复制数据类型 (CRDTs)** 之上，支持无缝的并发和离线编辑，同时保证最终一致性。
- **事件溯源：** 文档不是一个静态文件，而是一个可重放的、不可变的所有操作（变更）日志。这提供了极高的透明度、可审计性和强大的版本历史。
- **去中心化 & 网络无关：** 该架构支持各种网络拓扑（P2P、客户端-服务器、网状），且不依赖于单一中央服务器，确保了数据主权。
- **通用性 & 语义分离：** elfi-core 是一个通用的 CRDT 框架，不定义任何具体的块类型或业务逻辑。所有类型语义、属性含义、关系定义都完全由用户和项目自定义。
- **插件化扩展：** 通过插件系统实现类型处理、冲突解决、内容验证等功能，确保核心系统保持轻量和可扩展。

## 系统架构概览

`elfi` 系统采用分层架构，在一个模块化的 Rust 内核中实现。详细的技术设计请参考相应的设计文档。

### 1. 数据层：CRDT 驱动的文档模型

- **事件溯源 & 全历史保留：** 保留完整的不可变操作历史，支持时间旅行和精确差异追踪
- **Block 抽象：** 文档由简化的 4 字段 Block 构成，具体结构和语法参见 [.elf 文件格式规范](docs/src/implementations/01-elf_spec.md)
- **Relations 管理：** 通过专门的 Relations Block 统一管理所有块间关系，支持跨文档引用
- **扁平化 + 逻辑层级：** 底层数据扁平存储，通过邻接列表模型构建逻辑层级结构
- **插件化冲突解决：** 块级语义冲突解决策略，由插件系统根据类型提供具体实现

*详见 [数据建模设计文档](docs/src/designs/01-data_modeling.md)*

### 2. 网络层：Zenoh 分布式通信

- **协议无关：** 基于 Eclipse Zenoh 实现统一的发布/订阅、查询和存储网络
- **去中心化：** 支持 P2P、客户端-服务器、网状等多种网络拓扑
- **存储解耦：** 通过 Zenoh 存储插件支持多种持久化后端
- **实时同步：** CRDT 操作通过消息发布实现实时协作

*详见 [存储同步设计文档](docs/src/designs/02-storage_sync.md)*

### 3. API 层：双重抽象的编程接口

#### Weave API - 内容创作
- **仓库模型：** 提供 Git 风格的文档管理抽象
- **通用接口：** 不依赖具体块类型，通过插件扩展类型特定功能
- **关系管理：** 统一的关系创建、查询、验证接口
- **IDE 集成：** 双向文件同步，支持传统开发工具

*详见 [Weave API 设计文档](docs/src/designs/03-weave.md)*

#### Tangle API - 交互渲染
- **孤岛架构：** 静态 HTML + 选择性激活的交互组件
- **状态分离：** 本地 UI 状态与全局 CRDT 状态的清晰边界
- **Recipe 系统：** 用户自定义的内容转换和导出配置

*详见 [Tangle API 设计文档](docs/src/designs/04-tangle.md)*

### 4. 实现层：Rust 内核

- **模块化设计：** elfi-core、elfi-parser、elfi-cli、elfi-ffi 等独立 crate
- **核心依赖：** tree-sitter (解析)、automerge (CRDT)、zenoh (网络)
- **跨平台兼容：** FFI 层支持多语言绑定和 WebAssembly
- **性能优先：** 无头内核设计，专注于数据处理和同步效率

*详见 [解释器设计文档](docs/src/designs/05-interpreter.md)*

## 系统边界与设计哲学

### elfi-core 职责范围
**✅ elfi 负责：**
- 文档结构的解析和CRDT数据同步
- 网络通信和分布式存储抽象
- 命令行接口和基础文档操作
- 插件系统的框架和扩展机制

**❌ elfi 不负责：**
- 具体块类型的业务语义定义
- 内容格式的验证和处理逻辑
- 特定领域的冲突解决策略
- 内容的渲染和可视化展示

### 扩展性原则
- **类型系统开放：** 所有类型、属性、关系完全由用户定义
- **插件化处理：** 类型处理器、冲突解决器、验证器等通过插件实现
- **语义分离：** 结构管理与业务逻辑严格分离，保持核心系统通用性

---

**Agent Instructions:**

请总是使用中文进行回复，但使用英文的技术词汇。
- 请统一使用mermaid图来表示，即时ascii图等其它形式可能更好看，但mermaid使得我们可以准确无误地传达设计理念
# API 参考

本章节提供 ELFI 系统的完整 API 参考文档。

## 自动生成的 Rust API 文档

> **注意**: 以下链接指向自动生成的 Rust API 文档。在本地开发时，请先运行 `just docs-api` 或 `cargo doc --no-deps` 生成文档。

### 核心模块

#### [types](../../target/doc/types/index.html) - 核心数据结构
- **[Block](../../target/doc/types/block/struct.Block.html)**: 基础内容单元，支持多种内容类型
- **[Document](../../target/doc/types/document/struct.Document.html)**: 文档容器，管理一组相关的块
- **[Relation](../../target/doc/types/relation/struct.Relation.html)**: 描述块之间关系的对象
- **[TypeInterface](../../target/doc/types/interface/trait.TypeInterface.html)**: 可扩展的类型处理接口

#### [parser](../../target/doc/parser/index.html) - 解析器模块
- **[ElfParser](../../target/doc/parser/elf_parser/struct.ElfParser.html)**: .elf 文件解析器
- **[ParserInterface](../../target/doc/parser/interface/trait.ParserInterface.html)**: 解析器接口定义

#### [core](../../target/doc/elfi_core/index.html) - 核心功能
CRDT 操作和共享核心功能

#### 其他模块
- **[storage](../../target/doc/storage/index.html)**: 存储抽象层
- **[weave](../../target/doc/weave/index.html)**: 内容创作 API  
- **[tangle](../../target/doc/tangle/index.html)**: 交互渲染 API
- **[cli](../../target/doc/cli/index.html)**: 命令行接口
- **[extension](../../target/doc/extension/index.html)**: 插件系统
- **[recipe](../../target/doc/recipe/index.html)**: Recipe 系统

## 本地访问方式

如果上述链接无法访问，请使用以下方式：

### 方法 1: 直接打开文件
```bash
# 生成 API 文档
just docs-api

# 在浏览器中打开
open target/doc/types/index.html
```

### 方法 2: 启动本地服务器
```bash
# 在项目根目录运行
python3 -m http.server 8000

# 然后访问: http://localhost:8000/target/doc/types/index.html
```

## 集成测试文档

详细的集成测试和使用示例请参考：
- [自举开发用例](./usecases/01-bootstrapping.md)
- [对话即文档用例](./usecases/02-conversation-as-document.md)  
- [文档即应用用例](./usecases/03-document-as-app.md)

## 贡献指南

如需改进 API 文档：

1. **添加文档注释**: 在 `.rs` 文件中使用 `///` 添加函数和结构体文档
2. **添加示例代码**: 使用 ```` ```rust```` 代码块提供使用示例
3. **重新生成**: 运行 `just docs-api` 重新生成文档
4. **测试链接**: 确保文档中的链接有效

详见 [贡献指南](../CONTRIBUTING.md)。
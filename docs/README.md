# ELFI 文档

此目录包含 ELFI (Event-sourcing Literate File Interpreter) 项目的文档。

## 快速开始

### 新开发者

如果您是第一次设置此项目：

```bash
# 选项 1: 使用 just (推荐)
just init

# 选项 2: 手动设置
cargo install mdbook mdbook-mermaid
mdbook-mermaid install .
```

### 开发命令

```bash
# 本地启动文档服务 (文件变更时自动重新加载)
just serve
# 或
mdbook serve --open

# 构建文档
just build
# 或
mdbook build

# 生成合并后的 Markdown 文件
just merge
# 或
./merge_markdown.sh
```

## 项目结构

```
docs/
├── book.toml          # mdbook 配置
├── Cargo.toml         # 开发工具依赖
├── justfile           # 任务运行器
├── src/               # Markdown 源文件
│   ├── SUMMARY.md     # 书籍结构
│   ├── designs/       # 设计文档
│   ├── implementations/ # 实现文档
└── README.md          # 本文件
```

## 特性

- **Mermaid 图表**: 支持在 markdown 中使用 mermaid 图表
- **实时重新加载**: 开发过程中自动刷新
- **智能合并**: 将所有文档合并成一个单独的文件
- **代码高亮**: 支持多种语言的语法高亮
- **交叉引用**: 内部链接和导航

## Cargo 与 uv 对比

| 特性 | Cargo | uv/Python |
|---------|-------|-----------|
| 依赖文件 | `Cargo.toml` | `pyproject.toml` |
| 安装依赖 | `cargo install <tool>` | `uv sync` |
| 任务运行器 | `just` or `cargo run` | `uv run` |
| 锁定文件 | `Cargo.lock` | `uv.lock` |

**主要区别：**
- Cargo 工具通常是**全局**安装的 (`cargo install`)
- Python 工具通常是**按项目**安装的 (`uv add --dev`)
- 对于文档工具，通常首选全局安装

## 添加依赖

对于文档工具，请更新 `justfile` 或使用 `cargo-make`：

```toml
# 在 Cargo.toml 中 - 仅用于文档
[package.metadata.docs.tools]
mdbook = "0.4"
mdbook-mermaid = "0.15"
```

然后运行：
```bash
just install-tools
```

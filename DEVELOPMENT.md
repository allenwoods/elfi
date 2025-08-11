# ELFI 开发环境配置指南

欢迎加入 ELFI (Event-sourcing Literate File Interpreter) 项目！本指南将帮助你快速配置开发环境。

## 🚀 快速开始

### 自动配置（推荐）

```bash
# 克隆项目
git clone <repository-url>
cd elfi

# 运行自动配置脚本
./scripts/setup-dev.sh

# 开始开发
cd docs && just serve
```

### 手动配置

如果你想了解每个步骤或自动脚本遇到问题，请按照下面的详细说明进行。

## 📋 先决条件

### 核心工具

| 工具 | 版本要求 | 用途 | 安装方式 |
|------|----------|------|----------|
| **Rust** | >= 1.70 | 核心开发语言 | [rustup.rs](https://rustup.rs/) |
| **Git** | >= 2.20 | 版本控制 | [git-scm.com](https://git-scm.com/) |
| **just** | >= 1.0 | 任务运行器 | `cargo install just` |

### 文档工具

| 工具 | 用途 | 安装命令 |
|------|------|----------|
| **mdbook** | 文档生成 | `cargo install mdbook` |
| **mdbook-mermaid** | 图表支持 | `cargo install mdbook-mermaid` |

### 可选工具

| 工具 | 用途 | 安装方式 |
|------|------|----------|
| **uv** | Python 包管理（如需要） | `curl -LsSf https://astral.sh/uv/install.sh \| sh` |
| **VSCode** | 推荐 IDE | [code.visualstudio.com](https://code.visualstudio.com/) |

## 🔧 详细配置步骤

### 1. 安装 Rust 开发环境

<details>
<summary><strong>macOS</strong></summary>

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 验证安装
rustc --version
cargo --version
```

</details>

<details>
<summary><strong>Linux (Ubuntu/Debian)</strong></summary>

```bash
# 更新包列表
sudo apt update

# 安装依赖
sudo apt install -y curl build-essential

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 验证安装
rustc --version
cargo --version
```

</details>

<details>
<summary><strong>Windows</strong></summary>

1. 访问 [rustup.rs](https://rustup.rs/) 下载安装程序
2. 运行 `rustup-init.exe`
3. 按照提示完成安装
4. 重启终端并验证：
   ```powershell
   rustc --version
   cargo --version
   ```

</details>

### 2. 安装任务运行器

```bash
# 安装 just (类似 make 但更现代)
cargo install just

# 验证安装
just --version
```

### 3. 安装文档工具

```bash
# 安装文档生成工具
cargo install mdbook mdbook-mermaid

# 验证安装
mdbook --version
mdbook-mermaid --version
```

### 4. 配置项目

```bash
# 进入文档目录
cd docs

# 配置 mermaid 支持
mdbook-mermaid install .

# 构建文档测试
just build

# 启动开发服务器
just serve
```

如果一切顺利，浏览器会自动打开 http://localhost:3000 显示文档。

## 📁 项目结构

```
elfi/
├── DEVELOPMENT.md          # 本文件
├── README.md              # 项目简介
├── Cargo.toml            # Rust 项目配置
├── docs/                 # 📚 文档目录
│   ├── Cargo.toml       # 文档工具依赖
│   ├── justfile         # 文档任务脚本
│   ├── book.toml        # mdbook 配置
│   ├── src/             # markdown 源文件
│   └── merge_markdown.sh # 文档合并脚本
├── src/                  # 🦀 Rust 源代码
│   ├── elfi-core/       # 核心库
│   ├── elfi-parser/     # 解析器
│   ├── elfi-cli/        # 命令行工具
│   └── elfi-ffi/        # FFI 绑定
├── scripts/              # 🔧 开发脚本
└── tests/               # 🧪 测试文件
```

## 🛠️ 常用开发命令

### 文档开发

```bash
# 进入文档目录
cd docs

# 启动文档服务器（自动重载）
just serve

# 构建文档
just build

# 生成合并的 markdown
just merge

# 清理构建产物
just clean
```

### 代码开发

```bash
# 运行测试
cargo test

# 格式化代码
cargo fmt

# 代码检查
cargo clippy

# 构建项目
cargo build --release

# 运行 CLI
cargo run --bin elfi-cli -- --help
```

## 🔍 故障排除

### 常见问题

<details>
<summary><strong>Rust 安装失败</strong></summary>

**问题**: `curl: command not found` 或网络连接问题

**解决方案**:
- macOS: `xcode-select --install`
- Linux: `sudo apt install curl`
- 使用代理: `export https_proxy=http://your-proxy:port`
- 手动下载: 访问 [forge.rust-lang.org](https://forge.rust-lang.org/infra/channel-based-releases.html)

</details>

<details>
<summary><strong>mdbook 构建失败</strong></summary>

**问题**: `Summary parsing failed` 或文件重复

**解决方案**:
1. 检查 `docs/src/SUMMARY.md` 格式
2. 确保没有重复的文件路径
3. 验证所有引用的文件存在

```bash
# 验证 SUMMARY.md
cd docs && mdbook test
```

</details>

<details>
<summary><strong>mermaid 图表不显示</strong></summary>

**问题**: mermaid 代码块显示为纯文本

**解决方案**:
1. 确保已运行 `mdbook-mermaid install .`
2. 检查 `book.toml` 配置
3. 重新构建: `just clean && just build`

</details>

<details>
<summary><strong>权限问题</strong></summary>

**问题**: `Permission denied` 执行脚本

**解决方案**:
```bash
# 给脚本执行权限
chmod +x scripts/setup-dev.sh
chmod +x docs/merge_markdown.sh
```

</details>

## 🌐 IDE 配置推荐

### Visual Studio Code

推荐安装以下扩展：

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml", 
    "yzhang.markdown-all-in-one",
    "bierner.markdown-mermaid",
    "skellock.just"
  ]
}
```

### 配置文件

创建 `.vscode/settings.json`:

```json
{
  "rust-analyzer.cargo.buildScripts.enable": true,
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "markdown.extension.toc.levels": "2..6"
}
```

## 🤝 贡献指南

### 开发工作流

1. **Fork 项目** 并克隆到本地
2. **创建功能分支**: `git checkout -b feature/amazing-feature`
3. **进行开发** 并定期提交
4. **运行测试**: `cargo test` 和 `just build`
5. **提交 PR** 并请求代码审查

### 代码规范

- 使用 `cargo fmt` 格式化代码
- 运行 `cargo clippy` 检查代码质量
- 添加适当的文档注释
- 为新功能编写测试

### 文档规范

- 使用中文编写文档，但保留技术术语的英文
- 为复杂概念添加 mermaid 图表
- 更新相关的 README 和示例

## 📞 获取帮助

如果遇到问题，可以：

1. 查看项目 [Issues](https://github.com/your-org/elfi/issues)
2. 阅读 [FAQ](https://github.com/your-org/elfi/wiki/FAQ)
3. 在 [Discussions](https://github.com/your-org/elfi/discussions) 提问
4. 查看 `docs/` 目录下的详细文档

---

**Ready to hack!** 🎉

运行 `just --list` 查看所有可用的开发任务。
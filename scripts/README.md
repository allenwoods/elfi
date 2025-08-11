# ELFI 开发脚本

这个目录包含了用于配置和维护 ELFI 开发环境的脚本。

## 🚀 快速开始

### 自动配置

选择适合你操作系统的脚本：

**macOS / Linux:**
```bash
./scripts/setup-dev.sh
```

**Windows PowerShell:**
```powershell
.\scripts\setup-dev.ps1
```

**Windows (管理员权限):**
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
.\scripts\setup-dev.ps1
```

## 📝 脚本说明

### `setup-dev.sh` (macOS/Linux)

**功能:**
- 自动检测操作系统
- 安装 Rust 工具链
- 安装开发工具 (just, mdbook, mdbook-mermaid)
- 配置文档环境
- 创建 VSCode 配置文件
- 验证安装结果

**选项:**
```bash
# 基本安装
./setup-dev.sh

# 检查网络连接
ping -c 1 google.com && ./setup-dev.sh
```

**支持的系统:**
- macOS (通过 Homebrew)
- Ubuntu/Debian (通过 apt)
- CentOS/RHEL (通过 yum)
- 其他 Linux 发行版 (手动提示)

### `setup-dev.ps1` (Windows)

**功能:**
- 安装 Chocolatey 包管理器
- 安装 Git 和 VSCode
- 安装 Rust 工具链
- 配置开发工具
- 创建 IDE 配置

**参数:**
```powershell
# 基本安装
.\setup-dev.ps1

# 跳过 Rust 安装 (如果已安装)
.\setup-dev.ps1 -SkipRust

# 离线模式 (跳过网络检查)
.\setup-dev.ps1 -Offline
```

**权限要求:**
- 建议以管理员身份运行
- 需要 PowerShell 5.0+

## 🛠️ 安装的工具

### 必需工具

| 工具 | 版本 | 用途 |
|------|------|------|
| **Rust** | >= 1.70 | 核心开发语言 |
| **cargo** | 随 Rust | 包管理器 |
| **just** | >= 1.0 | 任务运行器 |
| **mdbook** | >= 0.4 | 文档生成 |
| **mdbook-mermaid** | >= 0.15 | 图表支持 |

### 可选工具

| 工具 | 用途 | 安装方式 |
|------|------|----------|
| **Git** | 版本控制 | 系统包管理器 |
| **VSCode** | IDE | 官网下载 |
| **Chocolatey** | Windows 包管理 | 脚本自动安装 |

## 🔧 手动配置

如果自动脚本失败，可以手动执行以下步骤：

### 1. 安装 Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. 安装开发工具
```bash
cargo install just mdbook mdbook-mermaid
```

### 3. 配置文档
```bash
cd docs
mdbook-mermaid install .
mdbook build
```

### 4. 验证安装
```bash
rustc --version
cargo --version
just --version
mdbook --version
```

## 🐛 故障排除

### 网络问题
```bash
# 设置代理
export https_proxy=http://your-proxy:port
export http_proxy=http://your-proxy:port

# 使用国内镜像 (中国用户)
export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
```

### 权限问题
```bash
# 给脚本执行权限
chmod +x scripts/setup-dev.sh

# Windows PowerShell 执行策略
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### 依赖问题

**Linux:**
```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config libssl-dev

# CentOS/RHEL
sudo yum groupinstall "Development Tools"
```

**macOS:**
```bash
xcode-select --install
```

### 常见错误

**"command not found: cargo"**
- 重启终端或运行: `source ~/.cargo/env`
- 检查 PATH: `echo $PATH | grep cargo`

**"Permission denied"**
- 检查脚本权限: `ls -la scripts/`
- 给予执行权限: `chmod +x scripts/setup-dev.sh`

**"Summary parsing failed"**
- 检查 `docs/src/SUMMARY.md` 格式
- 确保引用的文件存在

## 📞 获取帮助

如果脚本运行遇到问题：

1. 查看详细错误信息
2. 检查网络连接
3. 手动执行失败的步骤
4. 提交 Issue 到项目仓库

## 🔄 更新脚本

脚本会随项目更新，获取最新版本：

```bash
git pull origin main
./scripts/setup-dev.sh
```

---

**自动化让开发更简单！** 🎉
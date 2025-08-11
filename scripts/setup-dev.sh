#!/bin/bash

# ELFI 开发环境自动配置脚本
# 支持 macOS, Linux, Windows (Git Bash)

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# 图标
CHECKMARK="✅"
CROSS="❌"
ARROW="🔄"
ROCKET="🚀"

echo -e "${PURPLE}"
cat << "EOF"
╔════════════════════════════════════════╗
║       ELFI 开发环境配置脚本              ║
║   Event-sourcing Literate File        ║
║           Interpreter                  ║
╚════════════════════════════════════════╝
EOF
echo -e "${NC}"

# 检测操作系统
detect_os() {
    case "$OSTYPE" in
        darwin*)  OS="macOS" ;;
        linux*)   OS="Linux" ;;
        msys*|cygwin*|mingw*) OS="Windows" ;;
        *)        OS="Unknown" ;;
    esac
    echo -e "${BLUE}🖥️  检测到操作系统: $OS${NC}"
}

# 检查命令是否存在
command_exists() {
    command -v "$1" &> /dev/null
}

# 安装 Rust
install_rust() {
    echo -e "\n${YELLOW}${ARROW} 检查 Rust 安装状态...${NC}"
    
    if command_exists rustc && command_exists cargo; then
        RUST_VERSION=$(rustc --version | cut -d' ' -f2)
        echo -e "${GREEN}${CHECKMARK} Rust 已安装 (版本: $RUST_VERSION)${NC}"
        return 0
    fi
    
    echo -e "${YELLOW}${ARROW} 安装 Rust...${NC}"
    
    if [[ "$OS" == "Windows" ]]; then
        echo -e "${YELLOW}请手动访问 https://rustup.rs/ 下载并安装 Rust${NC}"
        echo -e "${YELLOW}安装完成后重新运行此脚本${NC}"
        exit 1
    else
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
    fi
    
    if command_exists rustc && command_exists cargo; then
        echo -e "${GREEN}${CHECKMARK} Rust 安装成功${NC}"
    else
        echo -e "${RED}${CROSS} Rust 安装失败${NC}"
        exit 1
    fi
}

# 安装系统依赖
install_system_deps() {
    echo -e "\n${YELLOW}${ARROW} 安装系统依赖...${NC}"
    
    case "$OS" in
        "macOS")
            if command_exists brew; then
                echo -e "${GREEN}${CHECKMARK} Homebrew 已安装${NC}"
            else
                echo -e "${YELLOW}${ARROW} 安装 Homebrew...${NC}"
                /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
            fi
            ;;
        "Linux")
            if command_exists apt; then
                echo -e "${YELLOW}${ARROW} 更新包列表...${NC}"
                sudo apt update
                echo -e "${YELLOW}${ARROW} 安装构建工具...${NC}"
                sudo apt install -y curl build-essential pkg-config libssl-dev
            elif command_exists yum; then
                echo -e "${YELLOW}${ARROW} 安装构建工具...${NC}"
                sudo yum groupinstall -y "Development Tools"
                sudo yum install -y curl openssl-devel
            else
                echo -e "${YELLOW}请手动安装 curl 和构建工具${NC}"
            fi
            ;;
        "Windows")
            echo -e "${YELLOW}确保已安装 Visual Studio Build Tools${NC}"
            ;;
    esac
}

# 安装 Rust 工具
install_rust_tools() {
    echo -e "\n${YELLOW}${ARROW} 安装 Rust 开发工具...${NC}"
    
    local tools=("just" "mdbook" "mdbook-mermaid")
    
    for tool in "${tools[@]}"; do
        if command_exists "$tool"; then
            echo -e "${GREEN}${CHECKMARK} $tool 已安装${NC}"
        else
            echo -e "${YELLOW}${ARROW} 安装 $tool...${NC}"
            cargo install "$tool"
            
            if command_exists "$tool"; then
                echo -e "${GREEN}${CHECKMARK} $tool 安装成功${NC}"
            else
                echo -e "${RED}${CROSS} $tool 安装失败${NC}"
            fi
        fi
    done
}

# 配置文档环境
setup_docs() {
    echo -e "\n${YELLOW}${ARROW} 配置文档环境...${NC}"
    
    if [[ ! -d "docs" ]]; then
        echo -e "${RED}${CROSS} 找不到 docs 目录，请确保在项目根目录运行此脚本${NC}"
        exit 1
    fi
    
    cd docs
    
    # 配置 mermaid
    if [[ -f "book.toml" ]]; then
        echo -e "${YELLOW}${ARROW} 配置 mdbook-mermaid...${NC}"
        mdbook-mermaid install .
        echo -e "${GREEN}${CHECKMARK} mermaid 配置完成${NC}"
    else
        echo -e "${RED}${CROSS} 找不到 book.toml${NC}"
    fi
    
    # 测试构建
    echo -e "${YELLOW}${ARROW} 测试文档构建...${NC}"
    if mdbook build; then
        echo -e "${GREEN}${CHECKMARK} 文档构建成功${NC}"
    else
        echo -e "${RED}${CROSS} 文档构建失败${NC}"
    fi
    
    cd ..
}

# 创建 IDE 配置
setup_ide_config() {
    echo -e "\n${YELLOW}${ARROW} 创建 IDE 配置文件...${NC}"
    
    # VSCode 配置
    mkdir -p .vscode
    
    # 扩展推荐
    cat > .vscode/extensions.json << 'EOF'
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml",
    "yzhang.markdown-all-in-one",
    "bierner.markdown-mermaid",
    "skellock.just",
    "vadimcn.vscode-lldb"
  ]
}
EOF

    # 设置
    cat > .vscode/settings.json << 'EOF'
{
  "rust-analyzer.cargo.buildScripts.enable": true,
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.rustfmt.rangeFormatting.enable": true,
  "editor.formatOnSave": true,
  "editor.rulers": [100],
  "markdown.extension.toc.levels": "2..6",
  "files.watcherExclude": {
    "**/target/**": true,
    "**/docs/book/**": true
  }
}
EOF

    # 调试配置
    cat > .vscode/launch.json << 'EOF'
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug elfi-cli",
      "cargo": {
        "args": ["build", "--bin=elfi-cli", "--package=elfi-cli"],
        "filter": {
          "name": "elfi-cli",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
EOF

    echo -e "${GREEN}${CHECKMARK} VSCode 配置文件已创建${NC}"
}

# 验证安装
verify_installation() {
    echo -e "\n${YELLOW}${ARROW} 验证安装...${NC}"
    
    local errors=0
    
    # 检查必要工具
    local required_tools=("rustc" "cargo" "just" "mdbook" "mdbook-mermaid")
    
    for tool in "${required_tools[@]}"; do
        if command_exists "$tool"; then
            local version=$($tool --version 2>/dev/null | head -n1)
            echo -e "${GREEN}${CHECKMARK} $tool: $version${NC}"
        else
            echo -e "${RED}${CROSS} $tool: 未找到${NC}"
            ((errors++))
        fi
    done
    
    return $errors
}

# 显示后续步骤
show_next_steps() {
    echo -e "\n${GREEN}${ROCKET} 配置完成！${NC}"
    echo -e "\n${BLUE}📋 后续步骤:${NC}"
    echo -e "   1. ${YELLOW}重新加载终端或运行:${NC} source ~/.bashrc (或 ~/.zshrc)"
    echo -e "   2. ${YELLOW}启动文档服务器:${NC} cd docs && just serve"
    echo -e "   3. ${YELLOW}运行测试:${NC} cargo test"
    echo -e "   4. ${YELLOW}查看所有任务:${NC} just --list"
    echo ""
    echo -e "${BLUE}🔗 有用的链接:${NC}"
    echo -e "   • 📖 文档: ${YELLOW}http://localhost:3000${NC}"
    echo -e "   • 🦀 Rust 学习: ${YELLOW}https://rustlings.cool/${NC}"
    echo -e "   • 📚 mdBook 指南: ${YELLOW}https://rust-lang.github.io/mdBook/${NC}"
    echo ""
    echo -e "${GREEN}Happy coding! 🎉${NC}"
}

# 主函数
main() {
    detect_os
    
    echo -e "\n${BLUE}开始配置开发环境...${NC}"
    
    install_system_deps
    install_rust
    install_rust_tools
    setup_docs
    setup_ide_config
    
    if verify_installation; then
        show_next_steps
    else
        echo -e "\n${RED}${CROSS} 配置过程中出现错误，请检查上述输出${NC}"
        exit 1
    fi
}

# 错误处理
trap 'echo -e "\n${RED}${CROSS} 脚本执行中断${NC}"; exit 1' INT TERM

# 检查网络连接
if ! ping -c 1 google.com &> /dev/null && ! ping -c 1 baidu.com &> /dev/null; then
    echo -e "${YELLOW}⚠️  网络连接检查失败，某些下载可能会失败${NC}"
    read -p "是否继续？(y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# 运行主函数
main "$@"
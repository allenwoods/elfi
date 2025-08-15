#!/bin/bash
# 环境设置脚本 - 解决cargo命令找不到的问题

echo "检查Rust/Cargo环境配置..."

# 检查cargo是否在PATH中
if command -v cargo &> /dev/null; then
    echo "✅ cargo命令已可用"
    cargo --version
else
    echo "❌ cargo命令不在PATH中"
    
    # 检查cargo是否存在于默认位置
    if [ -f "$HOME/.cargo/bin/cargo" ]; then
        echo "🔧 发现cargo在 $HOME/.cargo/bin/cargo"
        echo "需要修复PATH配置..."
        
        # 检查.zshrc是否已有cargo配置
        if grep -q "cargo" "$HOME/.zshrc"; then
            echo "⚠️  .zshrc中已存在cargo配置"
        else
            echo "🔨 正在修复.zshrc配置..."
            echo "" >> "$HOME/.zshrc"
            echo "# Rust cargo path (added by elfi setup)" >> "$HOME/.zshrc"
            echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$HOME/.zshrc"
            echo "✅ 已添加cargo路径到 ~/.zshrc"
            echo ""
            echo "请运行以下命令使配置生效："
            echo "  source ~/.zshrc"
            echo ""
            echo "或者重新启动终端"
        fi
    else
        echo "❌ 未找到cargo，请先安装Rust："
        echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    fi
fi

echo ""
echo "当前PATH内容："
echo "$PATH" | tr ':' '\n' | grep -E "(cargo|rust)" || echo "未找到Rust相关路径"
#!/bin/bash

# ELFI Documentation Markdown Merger
# 使用 awk/sed 合并所有 markdown 文件

set -e

# 配置
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SRC_DIR="$SCRIPT_DIR/src"
OUTPUT_FILE="$SCRIPT_DIR/elfi_documentation.md"

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}ELFI Markdown 文档合并工具${NC}"
echo "========================="

# 检查源目录
if [ ! -d "$SRC_DIR" ]; then
    echo "错误: 源目录 $SRC_DIR 不存在"
    exit 1
fi

# 开始合并
echo -e "${YELLOW}正在合并 markdown 文件...${NC}"

# 创建输出文件并添加标题
cat > "$OUTPUT_FILE" << 'EOF'
# ELFI: Event-sourcing Literate File Interpreter

> 完整文档合集

生成时间: 
EOF

# 添加生成时间
date '+%Y-%m-%d %H:%M:%S' >> "$OUTPUT_FILE"

# 添加目录
echo -e "\n---\n\n## 目录\n" >> "$OUTPUT_FILE"

# 生成目录结构
{
    echo "### Part I: 设计文档"
    echo "- [动机](#动机)"
    echo "- [数据建模](#数据建模)"
    echo "- [存储与同步](#存储与同步)"
    echo "- [Weave API](#weave-api)"
    echo "- [Tangle API](#tangle-api)"
    echo "- [解释器](#解释器)"
    echo ""
    echo "### Part II: 实现"
    echo "- [实现概览](#实现概览)"
    echo "- [解析器与格式](#解析器与格式)"
    echo "- [核心逻辑](#核心逻辑)"
    echo "- [命令行接口](#命令行接口)"
    echo ""
    echo "### Part III: 示例"
    echo "- [ELF 文件示例](#elf-文件示例)"
} >> "$OUTPUT_FILE"

echo -e "\n---\n" >> "$OUTPUT_FILE"

# 合并函数 - 用于普通 markdown 文件
merge_file() {
    local file="$1"
    local title="$2"
    local part="$3"
    
    if [ -f "$file" ]; then
        echo "  处理: $title"
        
        # 添加部分标题（如果是新部分）
        if [ -n "$part" ]; then
            echo -e "\n# $part\n" >> "$OUTPUT_FILE"
        fi
        
        # 添加章节标题和锚点
        echo -e "\n## $title\n" >> "$OUTPUT_FILE"
        
        # 使用 awk 处理文件内容：
        # 1. 跳过第一行（如果是标题）
        # 2. 将所有标题降级一级（# -> ##, ## -> ###, etc.）
        # 3. 保留代码块不变
        awk '
        BEGIN { in_code = 0; skip_first = 1 }
        /^```/ { in_code = !in_code }
        {
            if (NR == 1 && /^#[^#]/) {
                # 跳过第一行的一级标题
                next
            }
            if (!in_code && /^#/) {
                # 降级标题
                sub(/^#/, "##")
            }
            print
        }
        ' "$file" >> "$OUTPUT_FILE"
        
        # 添加分隔符
        echo -e "\n---\n" >> "$OUTPUT_FILE"
    else
        echo "  警告: 文件不存在 - $file"
    fi
}

# 特殊处理函数 - 用于代码文件或需要包裹在代码块中的文件
merge_code_file() {
    local file="$1"
    local title="$2"
    local part="$3"
    local lang="$4"
    
    if [ -f "$file" ]; then
        echo "  处理: $title (代码文件)"
        
        # 添加部分标题（如果是新部分）
        if [ -n "$part" ]; then
            echo -e "\n# $part\n" >> "$OUTPUT_FILE"
        fi
        
        # 添加章节标题
        echo -e "\n## $title\n" >> "$OUTPUT_FILE"
        
        # 将整个文件内容包裹在代码块中
        echo "\`\`\`$lang" >> "$OUTPUT_FILE"
        cat "$file" >> "$OUTPUT_FILE"
        echo "\`\`\`" >> "$OUTPUT_FILE"
        
        # 添加分隔符
        echo -e "\n---\n" >> "$OUTPUT_FILE"
    else
        echo "  警告: 文件不存在 - $file"
    fi
}

# 智能合并函数 - 根据文件扩展名决定处理方式
smart_merge() {
    local file="$1"
    local title="$2"
    local part="$3"
    
    if [ ! -f "$file" ]; then
        echo "  警告: 文件不存在 - $file"
        return
    fi
    
    # 获取文件扩展名
    local ext="${file##*.}"
    
    # 根据扩展名决定处理方式
    case "$ext" in
        md)
            # 检查是否是特殊的 .elf.md 文件
            if [[ "$file" == *".elf.md" ]]; then
                merge_code_file "$file" "$title" "$part" "elf"
            else
                merge_file "$file" "$title" "$part"
            fi
            ;;
        py)
            merge_code_file "$file" "$title" "$part" "python"
            ;;
        rs)
            merge_code_file "$file" "$title" "$part" "rust"
            ;;
        js|ts)
            merge_code_file "$file" "$title" "$part" "javascript"
            ;;
        sh|bash)
            merge_code_file "$file" "$title" "$part" "bash"
            ;;
        yaml|yml)
            merge_code_file "$file" "$title" "$part" "yaml"
            ;;
        json)
            merge_code_file "$file" "$title" "$part" "json"
            ;;
        toml)
            merge_code_file "$file" "$title" "$part" "toml"
            ;;
        *)
            # 默认作为普通文本处理
            echo "  处理: $title (未知类型，作为文本处理)"
            merge_file "$file" "$title" "$part"
            ;;
    esac
}

# Part I: 设计文档
merge_file "$SRC_DIR/designs/01-motivation.md" "动机" "Part I: 设计文档"
merge_file "$SRC_DIR/designs/02-data_modeling.md" "数据建模" ""
merge_file "$SRC_DIR/designs/03-storage_sync.md" "存储与同步" ""
merge_file "$SRC_DIR/designs/04-weave.md" "Weave API" ""
merge_file "$SRC_DIR/designs/05-tangle.md" "Tangle API" ""
merge_file "$SRC_DIR/designs/06-interpreter.md" "解释器" ""

# Part II: 实现
merge_file "$SRC_DIR/implementations/00-overview.md" "实现概览" "Part II: 实现"
merge_file "$SRC_DIR/implementations/01-parser_and_format.md" "解析器与格式" ""
merge_file "$SRC_DIR/implementations/02-core_logic.md" "核心逻辑" ""
merge_file "$SRC_DIR/implementations/03-cli.md" "命令行接口" ""

# Part III: 示例
# 使用智能合并函数处理示例文件
smart_merge "$SRC_DIR/example.elf.md" "ELF 文件示例" "Part III: 示例"

# 如果有其他示例文件，也可以添加
# smart_merge "$SRC_DIR/example.py" "Python 示例" ""
# smart_merge "$SRC_DIR/example.rs" "Rust 示例" ""

# 统计信息
echo -e "\n${GREEN}合并完成！${NC}"

# 使用 awk 统计文件信息
STATS=$(awk '
    BEGIN { lines = 0; words = 0; code_blocks = 0; headers = 0 }
    { lines++ }
    { words += NF }
    /^```/ { code_blocks++ }
    /^#/ { headers++ }
    END {
        printf "  行数: %d\n", lines
        printf "  词数: %d\n", words
        printf "  代码块: %d\n", code_blocks/2
        printf "  标题数: %d\n", headers
    }
' "$OUTPUT_FILE")

echo "$STATS"

# 文件大小
FILE_SIZE=$(ls -lh "$OUTPUT_FILE" | awk '{print $5}')
echo "  文件大小: $FILE_SIZE"
echo "  输出文件: $OUTPUT_FILE"

# 可选：生成简化版（去除代码块）
if [ "$1" = "--no-code" ]; then
    NO_CODE_FILE="${OUTPUT_FILE%.md}_no_code.md"
    echo -e "\n${YELLOW}生成无代码版本...${NC}"
    
    sed '/^```/,/^```/d' "$OUTPUT_FILE" > "$NO_CODE_FILE"
    
    echo "  无代码版本: $NO_CODE_FILE"
fi

echo -e "\n${GREEN}完成！${NC}"
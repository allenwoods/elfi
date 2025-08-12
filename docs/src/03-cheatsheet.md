# Elfi 命令速查表

## 核心命令体系

`elfi` 采用清晰的命令层级设计，一级命令对应核心工作流的五个阶段。

### 命令职责划分

| 一级命令 | 核心职责 | 使用场景 |
|---------|---------|---------|
| `open` | 会话管理 | 开始工作、激活文档 |
| `recipe` | 模式管理 | 设置工作模式、定义行为规则 |
| `weave` | 结构编辑 | 创建、修改、组织内容 |
| `tangle` | 智能纠缠 | 将松散内容纠缠成紧密系统 |
| `sync` | 协作同步 | 推送/拉取变更、解决冲突 |
| `log` | 历史追溯 | 查看历史、版本对比、时间旅行 |
| `validate` | 质量保证 | 语法检查、结构验证、规范检查 |

---

## 命令详解

### 📂 `elfi open` - 会话管理

```bash
# 打开/创建文档会话
elfi open elfi://[user/]repo/doc      # 打开远程文档
elfi open ./project.elf               # 打开本地文件
elfi open --new my-project            # 创建新文档

# 会话操作
elfi session list                     # 列出活动会话
elfi session switch <session-id>      # 切换会话
elfi session close [<session-id>]     # 关闭会话
```

### 🧩 `elfi recipe` - 模式管理

```bash
# Recipe 查看与使用
elfi recipe list                      # 列出所有可用配方
elfi recipe describe <name>           # 查看配方详细说明
elfi recipe set <name>                # 设置当前会话配方
elfi recipe current                   # 查看当前配方

# Recipe 创建与管理
elfi recipe create <name> --from <template>  # 创建新配方
elfi recipe edit <name>               # 编辑配方配置
elfi recipe delete <name>             # 删除配方
elfi recipe validate <name>           # 验证配方有效性

# 内置配方示例：
# - conversation-mode: 对话式协作，保留编辑历史
# - literate-mode: 文学化编程，文档代码并重
# - auto-complete: 智能补全，自动处理依赖
# - production-ready: 生产模式，严格验证
```

### ✏️ `elfi weave` - 结构化编辑

```bash
# 结构操作 - 管理块层级
elfi weave add --type <type> [--parent <id>]     # 添加新块
elfi weave delete <block-id> [--recursive]       # 删除块
elfi weave move <block-id> --to <parent-id>      # 移动块
elfi weave copy <block-id> --to <parent-id>      # 复制块

# 内容操作 - 编辑块内容
elfi weave edit <block-id> [--editor vim]        # 交互式编辑
elfi weave update <block-id> --file <path>       # 从文件更新
elfi weave append <block-id> --text "content"    # 追加内容

# 元数据操作 - 管理属性
elfi weave meta <block-id> --set key=value       # 设置元数据
elfi weave tag <block-id> --add tag1,tag2        # 添加标签
elfi weave annotate <block-id> --note "..."      # 添加注释

# 查询操作 - 浏览结构
elfi weave list [--type <type>]                  # 列出块
elfi weave show <block-id> [--json]              # 显示详情
elfi weave tree [--depth <n>]                    # 树状视图
elfi weave search <pattern> [--regex]            # 搜索内容

# 关系操作 - 管理连接
elfi weave link <from-id> <to-id>                # 创建链接
elfi weave unlink <from-id> <to-id>              # 删除链接
elfi weave depend <block-id> --on <dep-id>       # 声明依赖
```

### 🔧 `elfi tangle` - 智能纠缠

```bash
# 导出操作 - 生成源代码文件
elfi tangle export <block-id> --out <path>       # 导出单块
elfi tangle export --all --out-dir ./src         # 导出所有代码块
elfi tangle bundle --type <npm|pip|cargo>        # 打包项目

# 执行操作 - 运行代码
elfi tangle run <block-id> [--env .env]          # 运行代码块
elfi tangle exec --chain block1,block2,block3    # 链式执行
elfi tangle test [--pattern "test-*"]            # 运行测试块

# 渲染操作 - 生成文档
elfi tangle render --format <html|pdf|md>        # 渲染文档
elfi tangle preview [--port 8080] [--watch]      # 实时预览
elfi tangle publish --to <gh-pages|netlify>      # 发布文档

# 智能纠缠 - 内容自动增强
elfi tangle analyze <block-id>                   # 分析依赖和缺失
elfi tangle suggest <block-id>                   # 获取改进建议
elfi tangle complete <block-id> [--interactive]  # 交互式补全
elfi tangle generate <type> --from <block-id>    # 生成衍生内容

# 配方驱动 - 使用当前 Recipe
elfi tangle --recipe <name> ...                  # 使用指定配方执行
```

### 🔄 `elfi sync` - 协作同步

```bash
# 同步操作 - 推拉变更
elfi sync                                        # 完全同步
elfi sync pull                                   # 仅拉取
elfi sync push                                   # 仅推送

# 远程管理
elfi sync remote add <name> <url>                # 添加远程
elfi sync remote list                            # 列出远程
elfi sync remote remove <name>                   # 删除远程

# 冲突处理
elfi sync status                                 # 查看状态
elfi sync conflicts                              # 显示冲突
elfi sync resolve <block-id> --theirs|--ours     # 解决冲突

# 分支操作（基于 CRDT）
elfi sync branch create <name>                   # 创建分支
elfi sync branch list                            # 列出分支
elfi sync branch merge <name>                    # 合并分支
```

### 📜 `elfi log` - 历史追溯

```bash
# 历史查看
elfi log [--limit 20]                            # 查看历史
elfi log --block <id>                            # 块的历史
elfi log --author <name>                         # 作者历史

# 版本对比
elfi log diff <v1> <v2>                          # 比较版本
elfi log diff --block <id> <v1> <v2>             # 块级对比
elfi log blame <block-id>                        # 追溯来源

# 时间旅行
elfi log show <version>                          # 查看版本
elfi log checkout <version> --to <path>          # 导出版本
elfi log revert <operation-id>                   # 撤销操作

# 统计分析
elfi log stats [--since <date>]                  # 统计信息
elfi log graph                                   # 可视化历史
elfi log contributors                            # 贡献者列表
```

### ✅ `elfi validate` - 质量保证

```bash
# 结构验证
elfi validate                                    # 完整验证
elfi validate structure                          # 结构检查
elfi validate syntax                             # 语法检查

# 内容检查
elfi validate links                              # 检查链接
elfi validate deps                               # 检查依赖
elfi validate refs                               # 检查引用

# 规范检查
elfi validate style --guide <path>               # 风格检查
elfi validate schema --spec <schema.json>        # 模式验证
elfi validate rules --config <rules.yaml>        # 规则检查

# 修复建议
elfi validate fix                                # 自动修复
elfi validate report --format <json|html>        # 生成报告
```

---

## 常用工作流

### 1. 协作开发：从想法到代码
```bash
# 产品经理：定义需求
elfi open --new api-project
elfi recipe set conversation-mode
elfi weave add --type markdown --tag requirement

# 架构师：设计接口  
elfi recipe set literate-mode
elfi weave add --type code --meta language=openapi
elfi weave link <api-design> <requirement>

# 开发者：实现代码
elfi weave add --type code --meta language=python
elfi tangle analyze <impl-block>     # 分析缺失依赖
elfi tangle complete <impl-block>    # 智能补全

# 最终生成
elfi tangle export --all --out-dir ./src
```

### 2. 文学化编程：文档驱动开发
```bash
elfi recipe set literate-mode
elfi weave add --type markdown       # 写说明文档
elfi weave add --type code          # 写实现代码
elfi weave link <code> <doc>        # 建立关联
elfi tangle render --embed-code     # 生成含代码的文档
elfi tangle export --embed-docs     # 生成含文档的代码
```

### 3. 智能补全：渐进式完善
```bash
elfi tangle analyze                 # 分析整个项目
elfi tangle suggest <block-id>      # 获取具体建议
elfi tangle complete --interactive  # 交互式修复
elfi validate                       # 验证完整性
```

### 4. 版本管理：时间旅行
```bash
elfi log --limit 10                 # 查看历史
elfi log diff HEAD~1 HEAD           # 比较版本
elfi log checkout <version> --to backup/  # 导出历史版本
```

### 5. 协作同步：无冲突合并
```bash
elfi sync pull                      # 拉取远程变更
elfi sync conflicts                 # 检查冲突状态
elfi sync resolve <block> --merge   # 解决冲突
elfi sync push                      # 推送本地变更
```

---

## 环境变量

```bash
export ELFI_HOME=~/.elfi              # 配置目录
export ELFI_EDITOR=vim                # 默认编辑器
export ELFI_REMOTE=zenoh://hub.elfi   # 默认远程
export ELFI_AUTHOR="Your Name"        # 作者信息
```

---

## 配置文件

`.elfi/config.toml`:
```toml
[user]
name = "Your Name"
email = "you@example.com"

[sync]
auto_pull = true
conflict_strategy = "prompt"

[tangle]
default_format = "markdown"
preview_port = 8080

[validate]
auto_fix = false
strict_mode = true
```
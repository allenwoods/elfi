# Elfi 命令速查表

## 核心命令体系

`elfi` 采用清晰的命令层级设计，一级命令对应核心工作流的五个阶段。

### 命令职责划分

| 一级命令 | 核心职责 | 使用场景 |
|---------|---------|---------|
| `open` | 会话管理 | 开始工作、激活文档 |
| `weave` | 结构编辑 | 创建、修改、组织内容 |
| `tangle` | 输出生成 | 导出代码、渲染文档、执行程序 |
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

### 🔧 `elfi tangle` - 输出生成

```bash
# 导出操作 - 生成文件
elfi tangle export <block-id> --out <path>       # 导出单块
elfi tangle export --all --out-dir ./src         # 导出所有
elfi tangle bundle --type <npm|pip|cargo>        # 打包项目

# 执行操作 - 运行代码
elfi tangle run <block-id> [--env .env]          # 运行代码块
elfi tangle exec --chain block1,block2,block3    # 链式执行
elfi tangle test [--pattern "test-*"]            # 运行测试

# 渲染操作 - 生成文档
elfi tangle render --format <html|pdf|md>        # 渲染文档
elfi tangle preview [--port 8080]                # 实时预览
elfi tangle publish --to <gh-pages|netlify>      # 发布文档

# 配方操作 - 自动化流程
elfi tangle recipe list                          # 列出配方
elfi tangle recipe new <name> --from template    # 创建配方
elfi tangle recipe run <name> [--watch]          # 执行配方
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

### 1. 创建新项目
```bash
elfi open --new my-project
elfi weave add --type markdown
elfi weave edit <block-id>
elfi tangle render --format html
```

### 2. 文学化编程
```bash
elfi open project.elf
elfi weave add --type code --meta language=python
elfi weave edit <block-id>
elfi tangle run <block-id>
elfi tangle export --all --out-dir ./src
```

### 3. 团队协作
```bash
elfi sync pull
elfi weave list --type todo
elfi weave edit <block-id>
elfi sync push
```

### 4. 版本管理
```bash
elfi log --limit 10
elfi log diff HEAD~1 HEAD
elfi log checkout <version> --to backup/
```

### 5. 质量控制
```bash
elfi validate
elfi validate fix
elfi tangle test
elfi validate report --format html
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
# Elfi 命令速查表

## 核心命令体系

`elfi` 采用清晰的命令层级设计，一级命令对应核心工作流程。

## 标识符系统

elfi 使用三层标识符系统：

- **UUID**：`f47ac10b-58cc-4372-a567-0e02b2c3d479` - 全局唯一的真实身份
- **Hash ID**：`a1b2c3d4` - 8位操作记录指纹，用于log和checkout
- **人类可读名称**：`block-001`, `intro-section` - 便于记忆的别名

## 命令列表

### 📂 open - 会话管理

| 一级命令 | 二级命令 | 必选参数 | 可选参数 | 说明 |
|---------|---------|---------|---------|------|
| open | - | - | --new | 创建新仓库或文档 |
| open | - | URI路径 | - | 打开文档或区块进行编辑 |

**示例：**
- `elfi open --new repo` 
  - 返回: `Repository created: my-project`
- `elfi open --new elf://my-project/doc` 
  - 返回: `Document created and opened`
- `elfi open elf://my-project/doc` 
  - 返回: `Document loaded, sync enabled`
- `elfi open elf://my-project/doc/block-001` 
  - 返回: `Block opened for editing`

### ✏️ add - 内容创建

| 一级命令 | 二级命令 | 必选参数 | 可选参数 | 说明 |
|---------|---------|---------|---------|------|
| add | block | - | --type, --name, --merge_method, --parent | 添加新区块 |

**参数说明：**
- `--type`: 区块类型（markdown, code, python, recipe等）
- `--name`: 人类可读的区块标识符
- `--merge_method`: 合并策略（CRDT或manual）
- `--parent`: 父区块ID

**示例：**
- `elfi add block --type markdown --name block-001` 
  - 返回: `Created block f47ac10b... (aliased as block-001)`
- `elfi add block --merge_method=CRDT --name intro-section` 
  - 返回: `Created block 2a8b9c3d... (aliased as intro-section)`
- `elfi add block --parent block-001 --name block-002` 
  - 返回: `Created block 7e3f2a1b... (aliased as block-002, parent: block-001)`

**错误处理：**
- `elfi add block --name intro-section` (同名区块已存在)
  - 错误: `Block name 'intro-section' already exists in this document`

### 🔗 link - 关系管理

| 一级命令 | 二级命令 | 必选参数 | 可选参数 | 说明 |
|---------|---------|---------|---------|------|
| link | - | from-id, to-id | --type | 建立区块关联 |
| link | list | - | - | 列出所有链接 |
| link | show | block-id | - | 查看特定区块的关联 |
| link | remove | from-id, to-id | - | 删除链接 |

**关系类型：**
- `implements`: 实现关系
- `depends`: 依赖关系
- `extends`: 扩展关系

**示例：**
- `elfi link block-002 block-001 --type "implements"` - 建立实现关系
- `elfi link show block-001` - 查看block-001的关联

### 📤 export - 内容导出

| 一级命令 | 二级命令 | 必选参数 | 可选参数 | 说明 |
|---------|---------|---------|---------|------|
| export | - | 输出路径 | --recipe, --format, --type, --out | 导出文档或区块 |

**参数说明：**
- `--recipe`: 导出配方（使用项目中定义的Recipe名称）
- `--format`: 导出格式（用于单个区块）
- `--type`: 区块类型筛选
- `--out`: 输出目录

**Recipe系统：**
Recipe是存储在特殊区块中的转换脚本（YAML格式），定义如何处理和导出内容。

elfi不提供内置Recipe，每个项目根据需要自定义：
- 使用 `elfi add block --type recipe` 创建Recipe区块
- 编辑YAML配置定义选择器、转换规则、输出格式
- Recipe可以在项目间复制和修改复用

**跨文档引用支持：**
- 支持引用同一仓库内其他文档的区块
- 格式：`elf://repo/doc/block-id`
- 自动检测循环引用并报错
- 支持递归解析（可配置深度限制）

**示例：**
- `elfi export --recipe=markdown ./output.md` - 导出为Markdown
- `elfi export --recipe=code ./src/` - 导出为源代码
- `elfi export block-001 --format markdown` - 导出单个区块

### 🔄 sync - 协作同步

| 一级命令 | 二级命令 | 必选参数 | 可选参数 | 说明 |
|---------|---------|---------|---------|------|
| sync | - | - | - | 同步所有变更 |
| info | - | block-id | - | 查看区块信息和所有权 |
| transfer | - | block-id | --to | 转移区块所有权 |
| claim | - | block-id | - | 获取区块所有权 |
| resolve | - | block-id | --use | 解决冲突 |

**冲突解决选项：**
- `--use <hash-id>`: 使用特定版本
- `--use mine`: 使用本地版本
- `--use theirs`: 使用远程版本

**示例：**
- `elfi sync` 
  - 返回: `✓ CRDT blocks: 2 auto-merged` / `⚠ Manual blocks: 1 conflict detected`
- `elfi info block-002` 
  - 返回: `Type: code (manual), Owner: Alice, Modified: 2 conflicts pending`
- `elfi transfer block-002 --to Bob` 
  - 返回: `Ownership of block-002 transferred to Bob`
- `elfi claim block-002` 
  - 返回: `✓ You are now the owner of block-002`
- `elfi resolve block-002 --use e5f6a7b8` 
  - 返回: `block-002 resolved with specified version`

### 所有权规则

1. **初始分配**：创建区块的用户自动成为owner
2. **CRDT区块**：无ownership概念，所有人平等
3. **Manual区块**：只有owner可以解决冲突，可转移所有权

### 📜 log - 历史追溯

| 一级命令 | 二级命令 | 必选参数 | 可选参数 | 说明 |
|---------|---------|---------|---------|------|
| log | - | - | --limit, --block, --all | 查看操作历史 |

**参数说明：**
- `--limit`: 限制显示条数
- `--block`: 查看特定区块历史
- `--all`: 查看完整协作历史

**日志格式：**
```
<hash> | <时间> | <作者> | <操作描述>
```

**示例：**
- `elfi log --limit 5` - 查看最近5条历史
- `elfi log --block block-002` - 查看特定区块历史

### 🕰️ checkout - 版本切换

| 一级命令 | 二级命令 | 必选参数 | 可选参数 | 说明 |
|---------|---------|---------|---------|------|
| checkout | - | - | --at, --latest | 时间旅行 |

**参数说明：**
- `--at`: 指定变更点的hash
- `--latest`: 返回最新版本

**示例：**
- `elfi checkout --at "a1b2c3d4"` - 回到指定变更点
- `elfi checkout --latest` - 返回最新版本

### 🚪 close - 会话结束

| 一级命令 | 二级命令 | 必选参数 | 可选参数 | 说明 |
|---------|---------|---------|---------|------|
| close | - | URI路径 | - | 关闭文档或仓库 |

**示例：**
- `elfi close elf://my-project/doc` - 关闭文档

### 👁️ watch - 文件监听

| 一级命令 | 二级命令 | 必选参数 | 可选参数 | 说明 |
|---------|---------|---------|---------|------|
| watch | - | - | --sync-from, --format | IDE集成模式 |

**参数说明：**
- `--sync-from`: 监听的文件路径
- `--format`: 文件格式（code等）

**示例：**
- `elfi watch --sync-from ./src/ --format code` - 监听文件变化并同步

### 🏃 run - 执行构建

| 一级命令 | 二级命令 | 必选参数 | 可选参数 | 说明 |
|---------|---------|---------|---------|------|
| run | - | - | --recipe | 运行构建流程 |

**示例：**
- `elfi run --recipe build` 
  - 返回: `Build completed successfully`

### 📋 list - 资源列表

| 一级命令 | 二级命令 | 必选参数 | 可选参数 | 说明 |
|---------|---------|---------|---------|------|
| list | recipes | - | - | 列出所有可用Recipe |
| list | blocks | - | --type | 列出区块 |

**示例：**
- `elfi list recipes` 
  - 返回: `markdown-export | Custom export configuration for this project`

---

## 快速参考

### 命令概览

| 命令 | 用途 | 最常用形式 |
|------|------|------------|
| `open` | 创建或打开文档 | `elfi open --new repo` |
| `add` | 添加区块 | `elfi add block --name block-001` |
| `link` | 建立关联 | `elfi link block-002 block-001 --type "implements"` |
| `export` | 导出内容 | `elfi export --recipe=markdown ./output.md` |
| `sync` | 同步变更 | `elfi sync` |
| `log` | 查看历史 | `elfi log --limit 5` |
| `checkout` | 版本切换 | `elfi checkout --at "hash"` |
| `close` | 关闭文档 | `elfi close elf://project/doc` |
| `watch` | IDE集成 | `elfi watch --sync-from ./src/ --format code` |
| `run` | 执行构建 | `elfi run --recipe build` |

## 常用工作流

### 1. 项目初始化
```bash
elfi open --new repo                     # 创建仓库
elfi open --new elf://my-project/doc     # 创建文档
```

### 2. 内容创建与编辑
```bash
elfi add block --merge_method=CRDT --name=intro-section    # 添加markdown区块
elfi open elf://my-project/doc/intro-section               # 编辑区块  
elfi add block --merge_method=manual --name=main-function  # 添加代码区块
elfi link main-function intro-section --type "implements"  # 建立关联
```

### 3. 历史查看
```bash
elfi log --limit 5                          # 查看操作历史
elfi close elf://my-project/doc             # 关闭文档
```

### 4. 多人协作
```bash
elfi open elf://my-project/doc/block-002    # Bob打开共享文档
elfi add block --name=block-003             # Bob添加新区块
elfi link block-003 block-002 --type "depends"  # 建立依赖关系
```

### 5. 冲突处理
```bash
elfi sync                                      # 同步变更
elfi log --block main-function                 # 查看冲突历史
elfi info main-function                        # 查看区块信息
elfi transfer main-function --to Bob          # 转移所有权
elfi claim main-function                       # Bob获取所有权
elfi resolve main-function --use e5f6a7b8      # 解决冲突
```

### 6. 时间旅行
```bash
elfi log --all                              # 查看完整历史
elfi checkout --at "e5f6a7b8"                # 回到指定时间点
elfi checkout --latest                      # 返回最新版本
elfi close elf://my-project/doc             # 关闭文档
```

### 7. 导出与IDE集成
```bash
elfi export --recipe=markdown ./output.md  # 导出文档
elfi export --recipe=code ./src/           # 导出代码
elfi watch --sync-from ./src/ --format code # 启动双向同步
```

### 8. 构建与部署
```bash
elfi add block --type recipe --name build-config  # 创建构建配置
# 编辑Recipe内容（YAML格式）
elfi export --recipe=build ./output               # 生成构建脚本
elfi run --recipe build                           # 执行构建
elfi list recipes                                # 查看所有Recipe
```

---

## URI 格式

```
elf://[user/]repo/doc[/block-id]

# 示例：
elf://my-project/doc              # 文档根
elf://my-project/doc/block-001    # 特定区块
elf://alice/shared-project/doc    # 用户空间
```

---

## 常见错误类型

| 错误类型 | 示例 | 错误信息 | 解决方案 |
|---------|------|-----------|----------|
| 同名区块 | `elfi add block --name existing-name` | `Block name 'existing-name' already exists` | 使用不同名称或添加后缀 |
| 文档不存在 | `elf://repo/missing-doc/block` | `Document 'missing-doc' not found` | 检查文档路径是否正确 |
| 区块不存在 | `elf://repo/doc/missing-block` | `Block 'missing-block' not found` | 检查区块ID是否正确 |
| 跨仓库引用 | `elf://other-repo/doc/block` | `Cross-repository references not supported` | 仅在同一仓库内引用 |
| 循环引用 | A引用B，B引用A | `Circular reference detected` | 重新设计引用关系 |
| 无效URI格式 | `invalid-uri-format` | `Invalid URI format` | 使用正确格式 `elf://repo/doc/block` |

## 注意事项

### 标识符使用
- `block-001`、`intro-section` 等是人类可读的别名，实际每个区块都有UUID
- **命名唯一性**：同一文档内的区块名称必须唯一
- Hash ID格式为8位十六进制：`a1b2c3d4`
- 日志条目格式：`<hash> | <时间> | <作者> | <操作描述>`

### 协作机制  
- CRDT区块自动合并，Manual区块需要手动解决冲突
- Manual区块有owner概念，只有owner能解决冲突
- 权限转移需要接收方显式accept

### IDE集成条件
- 修改的文件必须是单个区块导出的
- 文件结构与导出时保持完全一致  
- 不允许删除文件或重命名
- 修改必须在合理时间窗口内发生

### Recipe系统
- Recipe是存储在特殊区块中的YAML配置
- 包含选择器、转换规则、输出配置
- Recipe本身也被版本控制，可以在项目间复制和修改
- elfi不提供内置Recipe，每个项目自定义所需配置

### 跨文档引用与错误处理
- 支持引用同一仓库内其他文档的区块内容
- URI格式：`elf://repo/doc/block-id`
- 错误处理策略：`placeholder`（占位符）、`error`（停止）、`skip`（跳过）
- 自动检测循环引用并防止无限递归
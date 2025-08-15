# 3. ELFI 依赖管理规范

**关联文件**: [01-overview.md](./01-overview.md)

ℹ️ **版本更新**: 基于2025年最新版本信息更新

## 核心依赖列表

基于[实现总览](../docs/src/implementations/00-overview.md)的技术栈选择:

### CRDT实现
```toml
automerge = "0.5"  # 注意: API仍在开发中，需要读源码
```
**用途**: 无冲突复制数据类型，事件溯源基础
**核心API**: 
- `AutomergeDoc::new()` - 创建文档
- `doc.transact()` - 执行操作
- `doc.merge()` - 合并变更

**选择理由**: 保留完整操作历史，支持时间旅行，比Yjs更适合版本控制场景

### 网络通信
```toml
zenoh = "1.5"  # 最新稳定版
```
**用途**: 分布式发布/订阅网络
**核心API**:
- `zenoh::open()` - 建立会话
- `session.declare_publisher()` - 发布者
- `session.declare_subscriber()` - 订阅者

**选择理由**: 支持多种网络拓扑，协议无关，比直接使用WebRTC更灵活

### 异步运行时
```toml
tokio = { version = "1.0", features = ["full"] }
```
**用途**: 异步IO和并发管理
**核心API**:
- `tokio::spawn()` - 异步任务
- `tokio::sync::Mutex` - 异步锁
- `tokio::time::sleep()` - 异步延时

**选择理由**: Rust生态标准，与zenoh 1.5兼容性好

### 序列化
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
```
**用途**: 数据序列化/反序列化
**核心API**:
- `#[derive(Serialize, Deserialize)]` - 自动实现
- `serde_json::to_string()` - JSON序列化
- `serde_yaml::from_str()` - YAML解析

**选择理由**: Rust生态标准，性能优秀

### 解析器
```toml
tree-sitter = "0.25"  # 最新版本，包含ABI版本15的重要改进
```
**用途**: .elf文件语法解析
**核心API**:
- `Parser::new()` - 创建解析器
- `parser.parse()` - 解析语法树
- `tree.root_node()` - 获取根节点

**选择理由**: 增量解析，语法高亮支持，比手写parser更robust

### 错误处理
```toml
thiserror = "1.0"
anyhow = "1.0"
```
**用途**: 错误类型定义和传播
**核心API**:
- `#[derive(Error)]` - 错误类型
- `anyhow::Result<T>` - 通用错误
- `context()` - 错误上下文

**选择理由**: 社区标准，互操作性好

**⚠️ 相似功能选择说明**:
- `thiserror` vs `failure`: thiserror更轻量，维护更积极
- `anyhow` vs `eyre`: anyhow简单直接，eyre功能过于复杂

### 模板引擎
```toml
handlebars = "4.0"
```
**用途**: Recipe系统内容转换
**核心API**:
- `Handlebars::new()` - 创建引擎
- `registry.register_template()` - 注册模板
- `registry.render()` - 渲染输出

**选择理由**: 语法简单，Rust实现稳定

**⚠️ 相似功能选择说明**:
- `handlebars` vs `tera`: handlebars语法更通用，跨语言支持好
- `handlebars` vs `minijinja`: handlebars生态更成熟

### 并发数据结构
```toml
dashmap = "5.0"
```
**用途**: 高性能并发HashMap
**核心API**:
- `DashMap::new()` - 创建并发映射
- `map.insert()` - 插入键值
- `map.get()` - 读取值

**选择理由**: 比`Arc<Mutex<HashMap>>`性能更好，API简洁

### 日志追踪
```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```
**用途**: 结构化日志和性能追踪
**核心API**:
- `info!()`, `debug!()` - 日志宏
- `tracing::span!()` - 性能跨度
- `tracing_subscriber::fmt::init()` - 初始化

**选择理由**: 比`log`更强大，支持结构化和分布式追踪

### HTTP客户端
```toml
reqwest = { version = "0.11", features = ["json"] }
```
**用途**: 跨文档引用的HTTP获取
**核心API**:
- `reqwest::get()` - GET请求
- `response.json()` - JSON解析
- `Client::new()` - 可配置客户端

**选择理由**: 异步友好，API简洁，社区首选

### 文件监听
```toml
notify = "6.0"
```
**用途**: IDE集成的文件变更监听
**核心API**:
- `RecommendedWatcher::new()` - 创建监听器
- `watcher.watch()` - 监听路径
- `Event::Create` - 事件类型

**选择理由**: 跨平台，事件丰富，性能良好

### UUID生成
```toml
uuid = { version = "1.0", features = ["v4", "serde"] }
```
**用途**: 块ID和会话ID生成
**核心API**:
- `Uuid::new_v4()` - 随机UUID
- `uuid.to_string()` - 字符串转换
- `Uuid::parse_str()` - 解析UUID

**选择理由**: 标准实现，加密安全随机

### 时间处理
```toml
chrono = { version = "0.4", features = ["serde"] }
```
**用途**: 时间戳和事件时间
**核心API**:
- `Utc::now()` - 当前UTC时间
- `DateTime::parse_from_rfc3339()` - 解析时间
- `dt.format()` - 格式化输出

**选择理由**: 功能完整，时区支持好

## 版本管理策略

### 主要版本锁定
锁定主要版本避免breaking changes:
```toml
automerge = "0.5"      # 保持0.5.x系列
zenoh = "0.10"         # 保持0.10.x系列
tokio = "1.0"          # 保持1.x系列
```

### 依赖更新流程
1. **安全更新**: 自动应用patch版本
2. **功能更新**: 手动评估minor版本  
3. **重大更新**: 创建专门的升级计划

### 冲突解决策略
```toml
# 当多个crate依赖不同版本时
[patch.crates-io]
# 强制统一版本
serde = { version = "1.0.130" }
```

## 开发依赖

### 测试框架
```toml
[dev-dependencies]
tokio-test = "0.4"     # 异步测试工具
proptest = "1.0"       # 属性测试
criterion = "0.4"      # 性能基准测试
```

### 代码质量
```toml
[dev-dependencies]
clippy = "0.1"         # 代码检查
rustfmt = "1.0"        # 代码格式化
tarpaulin = "0.22"     # 覆盖率统计
```

## 构建优化

### 发布配置
```toml
[profile.release]
opt-level = 3          # 最高优化
lto = true            # 链接时优化
codegen-units = 1     # 单线程编译优化
panic = "abort"       # 减小二进制大小
```

### 开发配置
```toml
[profile.dev]
opt-level = 0         # 快速编译
debug = true          # 调试信息
overflow-checks = true # 溢出检查
```

## 平台兼容性

### 目标平台
- `x86_64-unknown-linux-gnu` - Linux主要目标
- `x86_64-pc-windows-msvc` - Windows支持
- `x86_64-apple-darwin` - macOS Intel
- `aarch64-apple-darwin` - macOS ARM

### 平台特定依赖
```toml
[target.'cfg(windows)'.dependencies]
winapi = "0.3"

[target.'cfg(unix)'.dependencies]
libc = "0.2"
```

## 依赖审计

### 安全扫描
```bash
# 安装cargo-audit
cargo install cargo-audit

# 定期扫描漏洞
cargo audit
```

### 许可证检查
```bash
# 安装cargo-license
cargo install cargo-license

# 检查许可证兼容性
cargo license
```

### 依赖分析
```bash
# 依赖树分析
cargo tree

# 重复依赖检查
cargo duplicate
```

支持的许可证类型:
- MIT
- Apache-2.0  
- BSD-3-Clause
- ISC

**禁止使用**:
- GPL系列许可证
- 未知许可证的crate
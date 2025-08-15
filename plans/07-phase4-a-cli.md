# 4.a cli 模块开发计划

**阶段**: 第四阶段 - 用户接口 (串行)
**关联文件**: [01-overview.md](./01-overview.md), [04-phase1-c-core.md](./04-phase1-c-core.md), [07-phase4-b-extension.md](./07-phase4-b-extension.md)

## 模块职责
命令行工具实现，用户接口封装，配置管理。

## 数据结构设计

### CLI结构
```rust
pub struct ElfiCli {
    main: Arc<Main>,
    config: CliConfig,
    progress_reporter: ProgressReporter,
}
```

### Command枚举
```rust
pub enum Command {
    Open { uri: String },
    Add { doc_uri: String, block_type: String, name: Option<String> },
    Link { from: String, to: String, relation_type: String },
    Export { doc_uri: String, recipe: String, output: String },
    Sync { doc_uri: Option<String> },
    Watch { config: WatchConfig },
}
```

## API接口定义

```rust
pub trait CliInterface {
    async fn execute_command(command: Command) -> Result<CommandResult>;
    fn parse_args(args: Vec<String>) -> Result<Command>;
    fn format_output(result: &CommandResult, format: OutputFormat) -> String;
    fn load_config() -> Result<CliConfig>;
    fn save_config(config: &CliConfig) -> Result<()>;
}
```

## 功能点覆盖
- [ ] 命令参数解析
- [ ] 所有核心命令实现
- [ ] 配置文件管理
- [ ] 进度显示和用户体验
- [ ] 错误处理和友好提示
- [ ] 命令自动完成

## 依赖其他模块
- elfi-core: Main统一接口
- elfi-types: 所有数据类型

## 测试策略
- 命令参数解析测试
- 所有命令的功能测试
- 配置管理测试
- 错误处理测试

## 🤖 推荐使用的 Subagent

### 主要开发 Subagent
**@cli-ux-specialist**: 负责命令行用户体验的专业设计
- 设计直观、一致的命令行界面
- 实现用户友好的交互流程和错误处理
- 设计灵活的配置管理系统
- 实现进度显示和用户反馈
- 创建智能的自动完成和帮助系统

### 支持 Subagent
**@rust-tdd-developer**: 负责CLI功能的测试和质量保证
- 编写命令参数解析的测试
- 验证所有CLI命令的功能正确性
- 测试配置管理和用户交互
- 错误处理和边界条件测试

### 使用示例
```bash
# 第一步：CLI 界面设计和实现
@cli-ux-specialist 请实现用户友好的 ELFI 命令行工具。
要求：
1. 参考 docs/src/03-cheatsheet.md 中的完整命令规范
2. 实现所有核心命令（open, add, link, export, sync, watch等）
3. 设计统一的参数模式和错误处理
4. 实现多层级配置管理（项目、用户、系统）
5. 添加进度显示、交互提示和智能建议
6. 支持Shell自动完成和批处理脚本
7. 确保命令响应时间 < 100ms（本地操作）

# 第二步：CLI 功能测试
@rust-tdd-developer 请为CLI工具编写完整的测试套件。
要求：
1. 所有命令的参数解析测试
2. 命令功能的端到端测试
3. 配置文件管理的测试
4. 用户交互和错误处理测试
5. 批处理脚本的执行测试
6. 性能和响应时间验证

# 第三步：文档更新
@docs-maintainer 请更新以下文档：
1. docs/src/03-cheatsheet.md - 确保所有命令都有文档
2. docs/src/implementations/03-cli.md - CLI实现文档
3. docs/src/api/cli.md - CLI API参考文档
4. 用户使用示例和最佳实践
```
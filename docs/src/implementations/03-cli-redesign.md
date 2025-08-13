# CLI 重新设计 (`elfi-cli`)

基于用例场景和快速入门文档，重新设计完整的命令行界面，确保 CLI 作为 `Main` 类接口的简单封装。

## 1. CLI 架构设计

### 1.1. 设计原则

```
CLI 命令 → Main 接口函数 → Core 逻辑
   ↓           ↓              ↓
elfi open → Main.open() → DocumentManager.open_document()
```

每个 CLI 命令都直接映射到 `Main` 类的对应方法，确保其他语言绑定能够复用相同的逻辑。

### 1.2. 主要结构

```rust
// src/cli.rs
use clap::{Parser, Subcommand};
use elfi_core::{Main, ElfiConfig};

#[derive(Parser)]
#[command(
    name = "elfi",
    version = "0.1.0",
    about = "Event-sourcing Literate File Interpreter",
    long_about = "ELFI：事件溯源的文学化文件解释器，支持实时协作、跨文档引用和内容转换"
)]
struct Cli {
    /// 全局配置文件路径
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    
    /// 日志级别
    #[arg(long, global = true, default_value = "info")]
    log_level: String,
    
    /// Zenoh 路由器地址
    #[arg(long, global = true)]
    router: Option<String>,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 会话管理
    Open(OpenCommand),
    
    /// 内容创建
    Add(AddCommand),
    
    /// 关系管理  
    Link(LinkCommand),
    
    /// 内容导出
    Export(ExportCommand),
    
    /// 协作同步
    Sync(SyncCommand),
    Info(InfoCommand),
    Transfer(TransferCommand),
    Claim(ClaimCommand),
    Resolve(ResolveCommand),
    
    /// 历史追溯
    Log(LogCommand),
    Checkout(CheckoutCommand),
    
    /// 会话结束
    Close(CloseCommand),
    
    /// 文件监听
    Watch(WatchCommand),
    
    /// 执行构建
    Run(RunCommand),
    
    /// 资源列表
    List(ListCommand),
    
    /// 文件验证
    Validate(ValidateCommand),
    
    /// 新建文档
    New(NewCommand),
}
```

## 2. 会话管理命令

### 2.1. open - 创建或打开文档

```rust
#[derive(Args)]
struct OpenCommand {
    /// 文档或仓库 URI，如 elf://my-project/doc
    uri: Option<String>,
    
    /// 创建新仓库或文档
    #[arg(long)]
    new: bool,
}

impl OpenCommand {
    async fn execute(&self, main: &Main) -> Result<(), CliError> {
        match (&self.uri, self.new) {
            (None, true) => {
                // elfi open --new repo
                println!("创建新仓库...");
                let repo_name = prompt_input("仓库名称: ")?;
                let repo_uri = format!("elf://{}", repo_name);
                main.create_repository(&repo_uri).await?;
                println!("✅ Repository created: {}", repo_name);
            }
            (Some(uri), true) => {
                // elfi open --new elf://my-project/doc
                println!("创建新文档: {}", uri);
                main.create_document(uri).await?;
                println!("✅ Document created and opened");
            }
            (Some(uri), false) => {
                // elfi open elf://my-project/doc
                println!("打开文档: {}", uri);
                let handle = main.open(uri).await?;
                let info = handle.get_info().await?;
                println!("✅ Document loaded, {} blocks, sync enabled", info.block_count);
            }
            (None, false) => {
                return Err(CliError::MissingArgument("URI or --new flag required".to_string()));
            }
        }
        Ok(())
    }
}
```

### 2.2. close - 关闭文档

```rust
#[derive(Args)]
struct CloseCommand {
    /// 要关闭的文档 URI
    uri: String,
}

impl CloseCommand {
    async fn execute(&self, main: &Main) -> Result<(), CliError> {
        println!("关闭文档: {}", self.uri);
        main.close_document(&self.uri).await?;
        println!("✅ Document closed");
        Ok(())
    }
}
```

## 3. 内容创建命令

### 3.1. add block - 添加区块

```rust
#[derive(Args)]
struct AddCommand {
    /// 子命令：目前只支持 block
    #[command(subcommand)]
    subcommand: AddSubcommand,
}

#[derive(Subcommand)]
enum AddSubcommand {
    Block(AddBlockCommand),
}

#[derive(Args)]
struct AddBlockCommand {
    /// 区块类型
    #[arg(long, default_value = "markdown")]
    r#type: String,
    
    /// 人类可读的区块名称
    #[arg(long)]
    name: Option<String>,
    
    /// 合并策略：CRDT 或 manual
    #[arg(long)]
    merge_method: Option<String>,
    
    /// 父区块 ID
    #[arg(long)]
    parent: Option<String>,
    
    /// 语言（代码块）
    #[arg(long)]
    language: Option<String>,
    
    /// 当前工作的文档 URI（从环境或配置获取）
    #[arg(long)]
    doc: Option<String>,
}

impl AddBlockCommand {
    async fn execute(&self, main: &Main) -> Result<(), CliError> {
        let doc_uri = self.doc.as_ref()
            .or_else(|| std::env::var("ELFI_CURRENT_DOC").ok().as_ref())
            .ok_or_else(|| CliError::MissingArgument("Document URI required. Use --doc or set ELFI_CURRENT_DOC".to_string()))?;
        
        let block_type = BlockType::from_str(&self.r#type)?;
        let block_id = main.add_block(doc_uri, block_type, self.name.clone()).await?;
        
        // 设置附加属性
        if let Some(merge_method) = &self.merge_method {
            main.set_block_metadata(doc_uri, &block_id, "merge_method", merge_method).await?;
        }
        
        if let Some(parent) = &self.parent {
            main.set_block_metadata(doc_uri, &block_id, "parent", parent).await?;
        }
        
        if let Some(language) = &self.language {
            main.set_block_metadata(doc_uri, &block_id, "language", language).await?;
        }
        
        let name = self.name.as_deref().unwrap_or(&block_id[..8]);
        println!("✅ Created block {} (aliased as {})", &block_id[..8], name);
        
        Ok(())
    }
}
```

## 4. 导出命令

### 4.1. export - Recipe 驱动导出

```rust
#[derive(Args)]
struct ExportCommand {
    /// 输出路径
    output: Option<PathBuf>,
    
    /// Recipe 名称
    #[arg(long)]
    recipe: Option<String>,
    
    /// 输出格式（用于单个区块导出）
    #[arg(long)]
    format: Option<String>,
    
    /// 区块类型筛选
    #[arg(long)]
    r#type: Option<String>,
    
    /// 输出目录
    #[arg(long)]
    out: Option<PathBuf>,
    
    /// 目标区块（单个区块导出）
    block: Option<String>,
    
    /// 文档 URI
    #[arg(long)]
    doc: Option<String>,
}

impl ExportCommand {
    async fn execute(&self, main: &Main) -> Result<(), CliError> {
        let doc_uri = self.doc.as_ref()
            .or_else(|| std::env::var("ELFI_CURRENT_DOC").ok().as_ref())
            .ok_or_else(|| CliError::MissingArgument("Document URI required".to_string()))?;
        
        match (&self.recipe, &self.block, &self.format) {
            (Some(recipe_name), _, _) => {
                // Recipe 导出
                let output_path = self.output.as_ref()
                    .or(self.out.as_ref())
                    .ok_or_else(|| CliError::MissingArgument("Output path required".to_string()))?;
                
                println!("执行 Recipe: {} → {}", recipe_name, output_path.display());
                let result = main.export(doc_uri, recipe_name, &output_path.to_string_lossy()).await?;
                
                println!("✅ Recipe 执行成功:");
                println!("   - 处理了 {} 个区块", result.blocks_processed);
                println!("   - 解析了 {} 个外部引用", result.references_resolved);
                println!("   - 生成文件: {}", result.output_files.join(", "));
                
                if !result.warnings.is_empty() {
                    println!("\n⚠️  警告:");
                    for warning in &result.warnings {
                        println!("   - {}", warning);
                    }
                }
            }
            (None, Some(block_id), Some(format)) => {
                // 单个区块导出
                println!("导出区块 {} 为 {} 格式", block_id, format);
                let result = main.export_block(doc_uri, block_id, format).await?;
                if let Some(output) = &self.output {
                    tokio::fs::write(output, result.content).await?;
                    println!("✅ 导出到: {}", output.display());
                } else {
                    println!("{}", result.content);
                }
            }
            _ => {
                return Err(CliError::MissingArgument(
                    "需要指定 --recipe 或 --block+--format".to_string()
                ));
            }
        }
        
        Ok(())
    }
}
```

## 5. 协作同步命令

### 5.1. sync - 同步变更

```rust
#[derive(Args)]
struct SyncCommand {
    /// 文档 URI
    #[arg(long)]
    doc: Option<String>,
}

impl SyncCommand {
    async fn execute(&self, main: &Main) -> Result<(), CliError> {
        let doc_uri = self.doc.as_ref()
            .or_else(|| std::env::var("ELFI_CURRENT_DOC").ok().as_ref())
            .ok_or_else(|| CliError::MissingArgument("Document URI required".to_string()))?;
        
        println!("同步文档: {}", doc_uri);
        let result = main.sync(doc_uri).await?;
        
        match (result.crdt_merges, result.manual_conflicts) {
            (crdt, 0) => {
                println!("✅ CRDT blocks: {} auto-merged", crdt);
            }
            (crdt, conflicts) => {
                println!("✅ CRDT blocks: {} auto-merged", crdt);
                println!("⚠️  Manual blocks: {} conflict(s) detected", conflicts);
                
                // 显示冲突详情
                for conflict in &result.conflict_details {
                    println!("   - Block '{}': {} 个版本冲突", conflict.block_id, conflict.version_count);
                }
                
                println!("\n使用 'elfi resolve' 命令解决冲突");
            }
        }
        
        Ok(())
    }
}
```

### 5.2. transfer/claim - 所有权管理

```rust
#[derive(Args)]
struct TransferCommand {
    /// 区块 ID
    block_id: String,
    
    /// 目标用户
    #[arg(long)]
    to: String,
    
    /// 文档 URI
    #[arg(long)]
    doc: Option<String>,
}

impl TransferCommand {
    async fn execute(&self, main: &Main) -> Result<(), CliError> {
        let doc_uri = self.get_current_doc()?;
        
        println!("转移区块 '{}' 的所有权给 '{}'", self.block_id, self.to);
        main.transfer_ownership(&doc_uri, &self.block_id, &self.to).await?;
        
        println!("✅ Ownership of {} transferred to {}", self.block_id, self.to);
        Ok(())
    }
}

#[derive(Args)]
struct ClaimCommand {
    /// 区块 ID
    block_id: String,
    
    /// 文档 URI
    #[arg(long)]
    doc: Option<String>,
}

impl ClaimCommand {
    async fn execute(&self, main: &Main) -> Result<(), CliError> {
        let doc_uri = self.get_current_doc()?;
        
        println!("获取区块 '{}' 的所有权", self.block_id);
        main.claim_ownership(&doc_uri, &self.block_id).await?;
        
        println!("✅ You are now the owner of {}", self.block_id);
        Ok(())
    }
}
```

## 6. IDE 集成命令

### 6.1. watch - 文件监听

```rust
#[derive(Args)]
struct WatchCommand {
    /// 项目根目录
    #[arg(long)]
    project: Option<PathBuf>,
    
    /// 导出目录
    #[arg(long)]
    export_dir: Option<PathBuf>,
    
    /// 监听的文件路径
    #[arg(long)]
    sync_from: Option<PathBuf>,
    
    /// 文件格式过滤
    #[arg(long)]
    format: Option<String>,
    
    /// 区块类型过滤
    #[arg(long)]
    types: Option<String>,
    
    /// 同步模式
    #[arg(long, default_value = "bidirectional")]
    sync_mode: String,
}

impl WatchCommand {
    async fn execute(&self, main: &Main) -> Result<(), CliError> {
        let config = WatchConfig {
            project_root: self.project.clone().unwrap_or_else(|| PathBuf::from(".")),
            export_dir: self.export_dir.clone().unwrap_or_else(|| PathBuf::from("./exported")),
            sync_from: self.sync_from.clone(),
            format_filter: self.format.clone(),
            type_filter: self.parse_types()?,
            sync_mode: WatchMode::from_str(&self.sync_mode)?,
        };
        
        println!("启动文件监听服务...");
        println!("  项目目录: {}", config.project_root.display());
        println!("  导出目录: {}", config.export_dir.display());
        println!("  同步模式: {}", self.sync_mode);
        
        let watch_handle = main.watch(config).await?;
        
        println!("✅ 监听服务已启动，按 Ctrl+C 停止");
        
        // 等待中断信号
        tokio::signal::ctrl_c().await?;
        println!("\n正在停止监听服务...");
        
        watch_handle.stop().await?;
        println!("✅ 监听服务已停止");
        
        Ok(())
    }
    
    fn parse_types(&self) -> Result<Option<Vec<BlockType>>, CliError> {
        if let Some(types_str) = &self.types {
            let types = types_str
                .split(',')
                .map(|s| BlockType::from_str(s.trim()))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Some(types))
        } else {
            Ok(None)
        }
    }
}
```

## 7. 资源列表命令

### 7.1. list - 列出资源

```rust
#[derive(Args)]
struct ListCommand {
    #[command(subcommand)]
    subcommand: ListSubcommand,
}

#[derive(Subcommand)]
enum ListSubcommand {
    Recipes(ListRecipesCommand),
    Blocks(ListBlocksCommand),
    Documents(ListDocumentsCommand),
}

#[derive(Args)]
struct ListRecipesCommand {
    /// 文档 URI
    #[arg(long)]
    doc: Option<String>,
}

impl ListRecipesCommand {
    async fn execute(&self, main: &Main) -> Result<(), CliError> {
        let doc_uri = self.get_current_doc()?;
        let recipes = main.list_recipes(&doc_uri).await?;
        
        if recipes.is_empty() {
            println!("该文档中没有找到 Recipe");
            println!("\n使用 'elfi add block --type recipe' 创建 Recipe");
        } else {
            println!("可用的 Recipe:");
            for recipe in recipes {
                println!("  {} | {}", recipe.name, recipe.description);
                println!("    类型选择器: {}", recipe.selector_types.join(", "));
                println!("    外部引用: {}", recipe.references.len());
            }
        }
        
        Ok(())
    }
}
```

## 8. 错误处理和用户体验

### 8.1. 统一错误处理

```rust
#[derive(Error, Debug)]
pub enum CliError {
    #[error("缺少必需参数: {0}")]
    MissingArgument(String),
    
    #[error("无效的参数值: {0}")]
    InvalidArgument(String),
    
    #[error("Core 错误: {0}")]
    Core(#[from] elfi_core::ElfiError),
    
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("序列化错误: {0}")]
    Serde(#[from] serde_json::Error),
}

impl CliError {
    pub fn print_user_friendly(&self) {
        match self {
            CliError::Core(core_error) => {
                eprintln!("错误: {}", core_error.user_message());
                
                let suggestions = core_error.suggestions();
                if !suggestions.is_empty() {
                    eprintln!("\n建议:");
                    for suggestion in suggestions {
                        eprintln!("  • {}", suggestion);
                    }
                }
            }
            _ => {
                eprintln!("错误: {}", self);
            }
        }
    }
}
```

### 8.2. 进度显示和交互式提示

```rust
use indicatif::{ProgressBar, ProgressStyle};

pub struct ProgressReporter {
    bar: ProgressBar,
}

impl ProgressReporter {
    pub fn new(message: &str, total: u64) -> Self {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}"
            )
            .unwrap()
            .progress_chars("#>-")
        );
        bar.set_message(message.to_string());
        
        Self { bar }
    }
    
    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }
    
    pub fn finish_with_message(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }
}

pub fn prompt_input(message: &str) -> Result<String, CliError> {
    use std::io::{self, Write};
    
    print!("{}", message);
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(input.trim().to_string())
}

pub fn prompt_confirm(message: &str) -> Result<bool, CliError> {
    let response = prompt_input(&format!("{} (y/N): ", message))?;
    Ok(response.to_lowercase().starts_with('y'))
}
```

## 9. 配置和环境

### 9.1. 配置文件支持

```rust
#[derive(Serialize, Deserialize)]
pub struct CliConfig {
    pub default_router: Option<String>,
    pub current_document: Option<String>,
    pub user_name: String,
    pub log_level: String,
    pub watch_config: WatchConfig,
}

impl CliConfig {
    pub fn load() -> Result<Self, CliError> {
        let config_path = Self::config_file_path()?;
        
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }
    
    pub fn save(&self) -> Result<(), CliError> {
        let config_path = Self::config_file_path()?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        
        Ok(())
    }
    
    fn config_file_path() -> Result<PathBuf, CliError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| CliError::InvalidArgument("无法确定配置目录".to_string()))?;
        Ok(config_dir.join("elfi").join("config.toml"))
    }
}
```

这个重新设计的 CLI 确保了：

1. **完整的命令覆盖**：支持快速入门文档中的所有命令
2. **Main 接口映射**：每个命令都对应 Main 类的方法
3. **用户友好的错误处理**：详细的错误信息和建议
4. **进度反馈**：长时间操作显示进度
5. **配置管理**：支持配置文件和环境变量
6. **交互式体验**：确认提示和输入验证

这样的设计使得 CLI 成为核心功能的直接封装，其他语言的绑定也可以直接使用相同的 Main 接口。
# Extension系统实现机制

本文档详细阐述 ELFI Extension系统的具体实现机制，包括核心接口定义、加载流程、安全沙箱和开发工具链的技术实现细节。

## 1. 核心接口实现

### 1.1. Extension主接口定义

**Extension核心trait**：

```rust
// Extension主接口定义
use elfi_extension::{Extension, ExtensionContext, Result};

#[async_trait]
pub trait Extension: Send + Sync {
    // 基础信息
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn author(&self) -> &'static str;
    
    // 兼容性信息
    fn elfi_api_version(&self) -> &'static str;
    fn required_permissions(&self) -> Vec<Permission>;
    
    // 生命周期钩子
    async fn initialize(&mut self, context: &mut ExtensionContext) -> Result<()>;
    async fn activate(&mut self) -> Result<()>;
    async fn deactivate(&mut self) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
    
    // 健康检查
    async fn health_check(&self) -> HealthStatus;
}

// Extension上下文
pub struct ExtensionContext {
    plugin_registry: Arc<dyn PluginRegistry>,
    config: ExtensionConfig,
    logger: Arc<dyn Logger>,
    permissions: PermissionSet,
}

impl ExtensionContext {
    // 注册各种类型的插件
    pub fn register_block_type<T: BlockType + 'static>(&mut self, type_name: &str, block_type: T) -> Result<()> {
        self.plugin_registry.register_block_type(type_name, Box::new(block_type))
    }
    
    pub fn register_transformer<T: Transformer + 'static>(&mut self, name: &str, transformer: T) -> Result<()> {
        self.plugin_registry.register_transformer(name, Box::new(transformer))
    }
    
    pub fn register_renderer<T: Renderer + 'static>(&mut self, name: &str, renderer: T) -> Result<()> {
        self.plugin_registry.register_renderer(name, Box::new(renderer))
    }
}
```

### 1.2. BlockType扩展接口

**自定义块类型实现**：

```rust
// 自定义块类型trait
pub trait BlockType: Send + Sync {
    // 类型标识
    fn type_name(&self) -> &str;
    fn display_name(&self) -> &str;
    fn description(&self) -> &str;
    fn version(&self) -> &str;
    
    // 内容处理
    fn parse_content(&self, raw: &str, metadata: &BlockMetadata) -> Result<BlockContent>;
    fn serialize_content(&self, content: &BlockContent) -> Result<String>;
    fn validate_content(&self, content: &BlockContent) -> ValidationResult;
    
    // 渲染支持
    fn render_html(&self, content: &BlockContent, context: &RenderContext) -> Result<String>;
    fn render_preview(&self, content: &BlockContent) -> Result<String>;
    fn get_css_styles(&self) -> Vec<String>;
    
    // 编辑器集成
    fn get_syntax_definition(&self) -> Option<SyntaxDefinition>;
    fn get_completion_provider(&self) -> Option<Box<dyn CompletionProvider>>;
    fn get_hover_provider(&self) -> Option<Box<dyn HoverProvider>>;
    fn get_diagnostic_provider(&self) -> Option<Box<dyn DiagnosticProvider>>;
    
    // 序列化格式支持
    fn supported_formats(&self) -> Vec<SerializationFormat>;
    fn export_to_format(&self, content: &BlockContent, format: SerializationFormat) -> Result<String>;
}

// Protocol Buffers块类型实现示例
pub struct ProtobufMessageType;

impl BlockType for ProtobufMessageType {
    fn type_name(&self) -> &str { "proto_message" }
    fn display_name(&self) -> &str { "Protocol Buffer Message" }
    fn description(&self) -> &str { "Protocol Buffer message definition" }
    fn version(&self) -> &str { "1.0.0" }
    
    fn parse_content(&self, raw: &str, metadata: &BlockMetadata) -> Result<BlockContent> {
        // 解析protobuf语法
        let parsed = protobuf_parser::parse_message(raw)?;
        Ok(BlockContent::ProtobufMessage(parsed))
    }
    
    fn serialize_content(&self, content: &BlockContent) -> Result<String> {
        match content {
            BlockContent::ProtobufMessage(msg) => Ok(msg.to_proto_string()),
            _ => Err(Error::InvalidContentType),
        }
    }
    
    fn validate_content(&self, content: &BlockContent) -> ValidationResult {
        // 验证protobuf语法
        match content {
            BlockContent::ProtobufMessage(msg) => {
                let mut errors = Vec::new();
                if msg.fields.is_empty() {
                    errors.push(ValidationError::new("Message must have at least one field"));
                }
                ValidationResult { errors, warnings: vec![] }
            },
            _ => ValidationResult::invalid_type(),
        }
    }
    
    fn render_html(&self, content: &BlockContent, context: &RenderContext) -> Result<String> {
        // 渲染为HTML
        match content {
            BlockContent::ProtobufMessage(msg) => {
                let template = context.get_template("protobuf_message.hbs")?;
                template.render(&ProtobufMessageContext::from(msg))
            },
            _ => Err(Error::InvalidContentType),
        }
    }
    
    fn get_syntax_definition(&self) -> Option<SyntaxDefinition> {
        Some(SyntaxDefinition::from_file("protobuf.sublime-syntax"))
    }
    
    fn supported_formats(&self) -> Vec<SerializationFormat> {
        vec![
            SerializationFormat::Protobuf,
            SerializationFormat::Json,
            SerializationFormat::TypeScript,
            SerializationFormat::Python,
            SerializationFormat::Go,
        ]
    }
    
    fn export_to_format(&self, content: &BlockContent, format: SerializationFormat) -> Result<String> {
        match (content, format) {
            (BlockContent::ProtobufMessage(msg), SerializationFormat::TypeScript) => {
                // 生成TypeScript接口
                let generator = TypeScriptGenerator::new();
                generator.generate_interface(msg)
            },
            (BlockContent::ProtobufMessage(msg), SerializationFormat::Python) => {
                // 生成Python类
                let generator = PythonGenerator::new();
                generator.generate_class(msg)
            },
            _ => Err(Error::UnsupportedFormat),
        }
    }
}
```

### 1.3. Transformer扩展接口

**转换器管道实现**：

```rust
// 转换器trait
pub trait Transformer: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn version(&self) -> &str;
    
    // 输入输出类型
    fn input_types(&self) -> Vec<&str>;
    fn output_type(&self) -> &str;
    
    // 转换配置
    fn get_config_schema(&self) -> ConfigSchema;
    fn validate_config(&self, config: &Value) -> Result<()>;
    
    // 核心转换逻辑
    async fn transform(&self, input: &TransformInput, config: &Value) -> Result<TransformOutput>;
    
    // 批量处理支持
    async fn transform_batch(&self, inputs: Vec<&TransformInput>, config: &Value) -> Result<Vec<TransformOutput>>;
}

// Protobuf编译器转换器实现
pub struct ProtobufCompilerTransformer;

impl Transformer for ProtobufCompilerTransformer {
    fn name(&self) -> &str { "protobuf_compiler" }
    fn description(&self) -> &str { "Compile Protocol Buffer definitions to multiple languages" }
    fn version(&self) -> &str { "1.0.0" }
    
    fn input_types(&self) -> Vec<&str> {
        vec!["proto_message", "proto_service", "proto_enum"]
    }
    
    fn output_type(&self) -> &str { "generated_code" }
    
    fn get_config_schema(&self) -> ConfigSchema {
        ConfigSchema::builder()
            .property("languages", ConfigType::Array(Box::new(ConfigType::String)))
            .property("output_format", ConfigType::Enum(vec!["modules", "single_file"]))
            .property("grpc_support", ConfigType::Boolean)
            .property("include_paths", ConfigType::Array(Box::new(ConfigType::String)))
            .build()
    }
    
    async fn transform(&self, input: &TransformInput, config: &Value) -> Result<TransformOutput> {
        let languages: Vec<String> = config.get("languages")
            .and_then(|v| v.as_array())
            .ok_or(Error::InvalidConfig("languages field required"))?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        
        let mut outputs = HashMap::new();
        
        for lang in languages {
            match lang.as_str() {
                "typescript" => {
                    let code = self.compile_typescript(input).await?;
                    outputs.insert(format!("{}.ts", input.name), code);
                },
                "python" => {
                    let code = self.compile_python(input).await?;
                    outputs.insert(format!("{}_pb2.py", input.name), code);
                },
                "go" => {
                    let code = self.compile_go(input).await?;
                    outputs.insert(format!("{}.pb.go", input.name), code);
                },
                _ => return Err(Error::UnsupportedLanguage(lang)),
            }
        }
        
        Ok(TransformOutput::MultiFile(outputs))
    }
    
    async fn compile_typescript(&self, input: &TransformInput) -> Result<String> {
        // 调用protoc-gen-ts
        let temp_dir = TempDir::new()?;
        let proto_file = temp_dir.path().join("input.proto");
        fs::write(&proto_file, &input.content)?;
        
        let output = Command::new("protoc")
            .arg("--ts_out=./")
            .arg("--proto_path=./")
            .arg("input.proto")
            .current_dir(&temp_dir)
            .output()?;
        
        if !output.status.success() {
            return Err(Error::CompilationFailed(String::from_utf8_lossy(&output.stderr).to_string()));
        }
        
        let ts_file = temp_dir.path().join("input.ts");
        Ok(fs::read_to_string(ts_file)?)
    }
}
```

## 2. Extension加载机制

### 2.1. 动态加载实现

**Extension包加载器**：

```rust
// Extension包加载器
pub struct ExtensionLoader {
    loaded_extensions: HashMap<String, LoadedExtension>,
    extension_paths: Vec<PathBuf>,
    security_manager: Arc<SecurityManager>,
}

impl ExtensionLoader {
    pub async fn load_extension(&mut self, package_path: &Path) -> Result<String> {
        // 1. 验证包完整性
        self.verify_package_integrity(package_path).await?;
        
        // 2. 解析元数据
        let metadata = self.parse_metadata(package_path)?;
        
        // 3. 检查兼容性
        self.check_compatibility(&metadata)?;
        
        // 4. 验证权限
        self.verify_permissions(&metadata).await?;
        
        // 5. 加载动态库
        let library = self.load_dynamic_library(package_path, &metadata)?;
        
        // 6. 初始化Extension
        let extension = self.initialize_extension(&library, &metadata).await?;
        
        // 7. 注册到系统
        let extension_id = format!("{}@{}", metadata.name, metadata.version);
        self.loaded_extensions.insert(extension_id.clone(), LoadedExtension {
            extension,
            library,
            metadata,
            status: ExtensionStatus::Loaded,
        });
        
        Ok(extension_id)
    }
    
    async fn verify_package_integrity(&self, package_path: &Path) -> Result<()> {
        // 验证数字签名
        let signature_file = package_path.join("SIGNATURE");
        if signature_file.exists() {
            let signature = fs::read(&signature_file)?;
            let package_hash = self.calculate_package_hash(package_path)?;
            self.security_manager.verify_signature(&package_hash, &signature)?;
        }
        
        Ok(())
    }
    
    fn parse_metadata(&self, package_path: &Path) -> Result<ExtensionMetadata> {
        let metadata_file = package_path.join("extension.toml");
        let content = fs::read_to_string(metadata_file)?;
        let metadata: ExtensionMetadata = toml::from_str(&content)?;
        
        // 验证元数据
        metadata.validate()?;
        Ok(metadata)
    }
    
    fn check_compatibility(&self, metadata: &ExtensionMetadata) -> Result<()> {
        let elfi_version = env!("CARGO_PKG_VERSION");
        let requirement = VersionReq::parse(&metadata.elfi_version)?;
        let current = Version::parse(elfi_version)?;
        
        if !requirement.matches(&current) {
            return Err(Error::IncompatibleVersion {
                required: metadata.elfi_version.clone(),
                current: elfi_version.to_string(),
            });
        }
        
        Ok(())
    }
    
    fn load_dynamic_library(&self, package_path: &Path, metadata: &ExtensionMetadata) -> Result<Library> {
        let lib_path = package_path.join("lib").join(format!("lib{}.so", metadata.name));
        
        // 在沙箱环境中加载
        let library = unsafe {
            Library::new(lib_path)?
        };
        
        Ok(library)
    }
    
    async fn initialize_extension(&self, library: &Library, metadata: &ExtensionMetadata) -> Result<Box<dyn Extension>> {
        // 获取初始化函数
        let init_fn: Symbol<fn() -> Box<dyn Extension>> = unsafe {
            library.get(b"create_extension")?
        };
        
        // 创建Extension实例
        let mut extension = init_fn();
        
        // 创建受限的上下文
        let context = self.create_extension_context(metadata).await?;
        
        // 初始化Extension
        extension.initialize(context).await?;
        
        Ok(extension)
    }
}

// 动态库导出的标准接口
#[no_mangle]
pub extern "C" fn create_extension() -> Box<dyn Extension> {
    Box::new(MyExtension::new())
}

#[no_mangle]
pub extern "C" fn extension_api_version() -> &'static str {
    "1.0.0"
}
```

### 2.2. 安全沙箱实现

**权限控制系统**：

```rust
// 权限管理器
pub struct PermissionManager {
    granted_permissions: HashMap<String, PermissionSet>,
    permission_policies: Vec<PermissionPolicy>,
    audit_logger: Arc<dyn AuditLogger>,
}

impl PermissionManager {
    pub fn check_permission(&self, extension_id: &str, permission: &Permission) -> Result<()> {
        let granted = self.granted_permissions.get(extension_id)
            .ok_or(Error::ExtensionNotFound)?;
        
        if !granted.contains(permission) {
            self.audit_logger.log_permission_denied(extension_id, permission);
            return Err(Error::PermissionDenied {
                extension: extension_id.to_string(),
                permission: permission.clone(),
            });
        }
        
        self.audit_logger.log_permission_used(extension_id, permission);
        Ok(())
    }
    
    pub fn grant_permissions(&mut self, extension_id: &str, permissions: PermissionSet) -> Result<()> {
        // 检查是否需要用户确认
        let requires_confirmation = permissions.iter()
            .any(|p| self.requires_user_confirmation(p));
        
        if requires_confirmation {
            // 请求用户确认
            self.request_user_confirmation(extension_id, &permissions)?;
        }
        
        self.granted_permissions.insert(extension_id.to_string(), permissions);
        Ok(())
    }
}

// 沙箱执行环境
pub struct SandboxedExtension {
    extension: Box<dyn Extension>,
    permissions: PermissionSet,
    resource_limits: ResourceLimits,
    monitor: Arc<ResourceMonitor>,
}

impl SandboxedExtension {
    pub async fn execute_with_sandbox<F, R>(&self, operation: F) -> Result<R>
    where
        F: FnOnce() -> Result<R> + Send + 'static,
        R: Send + 'static,
    {
        // 设置资源限制
        let _guard = self.resource_limits.apply()?;
        
        // 在监控下执行
        let result = self.monitor.execute_monitored(operation).await?;
        
        // 检查资源使用情况
        self.monitor.check_resource_usage()?;
        
        Ok(result)
    }
}

// 资源监控器
pub struct ResourceMonitor {
    cpu_limit: Duration,
    memory_limit: usize,
    start_time: Instant,
    start_memory: usize,
}

impl ResourceMonitor {
    pub async fn execute_monitored<F, R>(&self, operation: F) -> Result<R>
    where
        F: FnOnce() -> Result<R> + Send + 'static,
        R: Send + 'static,
    {
        let start = Instant::now();
        let start_memory = self.get_memory_usage()?;
        
        // 使用timeout执行操作
        let result = timeout(self.cpu_limit, spawn_blocking(operation)).await??;
        
        // 检查执行时间
        let elapsed = start.elapsed();
        if elapsed > self.cpu_limit {
            return Err(Error::CpuLimitExceeded { limit: self.cpu_limit, used: elapsed });
        }
        
        // 检查内存使用
        let memory_used = self.get_memory_usage()? - start_memory;
        if memory_used > self.memory_limit {
            return Err(Error::MemoryLimitExceeded { limit: self.memory_limit, used: memory_used });
        }
        
        Ok(result)
    }
}
```

## 3. 开发工具链实现

### 3.1. Extension脚手架

**项目生成器**：

```rust
// Extension项目生成器
pub struct ExtensionGenerator {
    template_registry: TemplateRegistry,
    dependency_resolver: DependencyResolver,
}

impl ExtensionGenerator {
    pub async fn generate_project(&self, config: &GenerationConfig) -> Result<()> {
        let template = self.template_registry.get_template(&config.template_type)?;
        
        // 创建项目目录
        fs::create_dir_all(&config.output_path)?;
        
        // 生成基础文件
        self.generate_cargo_toml(&config).await?;
        self.generate_extension_toml(&config).await?;
        self.generate_source_files(&config, &template).await?;
        self.generate_tests(&config).await?;
        self.generate_documentation(&config).await?;
        
        // 初始化git仓库
        if config.init_git {
            self.init_git_repository(&config.output_path).await?;
        }
        
        Ok(())
    }
    
    async fn generate_cargo_toml(&self, config: &GenerationConfig) -> Result<()> {
        let cargo_toml = CargoToml {
            package: Package {
                name: config.name.clone(),
                version: "0.1.0".to_string(),
                edition: "2021".to_string(),
                authors: vec![config.author.clone()],
                description: config.description.clone(),
                license: config.license.clone(),
            },
            lib: Some(Lib {
                crate_type: vec!["cdylib".to_string()],
            }),
            dependencies: self.get_base_dependencies(),
        };
        
        let content = toml::to_string_pretty(&cargo_toml)?;
        let path = config.output_path.join("Cargo.toml");
        fs::write(path, content)?;
        
        Ok(())
    }
    
    async fn generate_source_files(&self, config: &GenerationConfig, template: &Template) -> Result<()> {
        let src_dir = config.output_path.join("src");
        fs::create_dir_all(&src_dir)?;
        
        // 生成lib.rs
        let lib_content = template.render("lib.rs", config)?;
        fs::write(src_dir.join("lib.rs"), lib_content)?;
        
        // 根据Extension类型生成特定文件
        match config.extension_type {
            ExtensionType::BlockType => {
                let content = template.render("block_type.rs", config)?;
                fs::write(src_dir.join("block_type.rs"), content)?;
            },
            ExtensionType::Transformer => {
                let content = template.render("transformer.rs", config)?;
                fs::write(src_dir.join("transformer.rs"), content)?;
            },
            ExtensionType::Renderer => {
                let content = template.render("renderer.rs", config)?;
                fs::write(src_dir.join("renderer.rs"), content)?;
            },
            ExtensionType::Full => {
                // 生成所有类型的文件
                for file in &["block_type.rs", "transformer.rs", "renderer.rs"] {
                    let content = template.render(file, config)?;
                    fs::write(src_dir.join(file), content)?;
                }
            },
        }
        
        Ok(())
    }
}

// 命令行接口
#[derive(Parser)]
#[command(name = "elfi")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Extension management
    Extension {
        #[command(subcommand)]
        extension_command: ExtensionCommand,
    },
    /// Permission management
    Permission {
        #[command(subcommand)]
        permission_command: PermissionCommand,
    },
    // ... 其他主命令
}

#[derive(Subcommand)]
pub enum ExtensionCommand {
    /// Install an Extension
    Install {
        /// Extension name or path
        extension: String,
        /// Specific version
        #[arg(long)]
        version: Option<String>,
        /// Install globally
        #[arg(long)]
        global: bool,
        /// Development mode
        #[arg(long)]
        dev: bool,
        /// Force reinstall
        #[arg(long)]
        force: bool,
    },
    /// Update an Extension
    Update {
        /// Extension name
        extension: String,
    },
    /// Remove an Extension
    Remove {
        /// Extension name
        extension: String,
    },
    /// Search for Extensions
    Search {
        /// Search keyword
        keyword: String,
    },
    /// Initialize a new Extension project
    Init {
        /// Extension name
        name: String,
        /// Template type
        #[arg(long, default_value = "basic")]
        template: String,
        /// Author name
        #[arg(long)]
        author: Option<String>,
        /// Extension description
        #[arg(long)]
        description: Option<String>,
    },
    /// Pack Extension for distribution
    Pack {
        /// Output path
        #[arg(long)]
        output: Option<String>,
    },
    /// Publish Extension to registry
    Publish {
        /// Target registry
        #[arg(long)]
        registry: Option<String>,
    },
    /// Test Extension
    Test {
        /// Test target
        #[arg(long)]
        target: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum PermissionCommand {
    /// View permission information
    Info {
        /// URI of the resource
        uri: String,
    },
    /// Transfer ownership
    Transfer {
        /// URI of the resource
        uri: String,
        /// Target user
        #[arg(long)]
        to: String,
    },
    /// Claim ownership
    Claim {
        /// URI of the resource
        uri: String,
    },
    /// Grant permission
    Grant {
        /// URI of the resource
        uri: String,
        /// Target user
        #[arg(long)]
        user: String,
        /// Permission type
        #[arg(long)]
        permission: String,
    },
    /// Revoke permission
    Revoke {
        /// URI of the resource
        uri: String,
        /// Target user
        #[arg(long)]
        user: String,
        /// Permission type
        #[arg(long)]
        permission: String,
    },
    /// Review permission history
    Review {
        /// URI of the resource
        uri: String,
    },
}
```

### 3.2. 调试和测试支持

**Extension测试框架**：

```rust
// Extension测试框架
pub struct ExtensionTestFramework {
    test_environment: TestEnvironment,
    mock_registry: MockPluginRegistry,
}

impl ExtensionTestFramework {
    pub async fn test_extension(&self, extension_path: &Path) -> Result<TestReport> {
        let mut report = TestReport::new();
        
        // 加载Extension
        let extension = self.load_test_extension(extension_path).await?;
        
        // 运行单元测试
        let unit_results = self.run_unit_tests(&extension).await?;
        report.add_results("unit", unit_results);
        
        // 运行集成测试
        let integration_results = self.run_integration_tests(&extension).await?;
        report.add_results("integration", integration_results);
        
        // 运行性能测试
        let performance_results = self.run_performance_tests(&extension).await?;
        report.add_results("performance", performance_results);
        
        // 运行安全测试
        let security_results = self.run_security_tests(&extension).await?;
        report.add_results("security", security_results);
        
        Ok(report)
    }
    
    async fn run_unit_tests(&self, extension: &dyn Extension) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();
        
        // 测试基本功能
        results.push(self.test_extension_metadata(extension)?);
        results.push(self.test_initialization(extension).await?);
        results.push(self.test_block_type_registration(extension).await?);
        
        Ok(results)
    }
    
    async fn run_integration_tests(&self, extension: &dyn Extension) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();
        
        // 测试与ELFI核心的集成
        results.push(self.test_core_integration(extension).await?);
        results.push(self.test_api_compatibility(extension).await?);
        
        Ok(results)
    }
    
    async fn run_security_tests(&self, extension: &dyn Extension) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();
        
        // 测试权限遵守
        results.push(self.test_permission_compliance(extension).await?);
        
        // 测试资源使用
        results.push(self.test_resource_limits(extension).await?);
        
        // 测试输入验证
        results.push(self.test_input_validation(extension).await?);
        
        Ok(results)
    }
}

// 开发模式支持
pub struct DevelopmentMode {
    file_watcher: RecommendedWatcher,
    extension_reloader: ExtensionReloader,
    debug_server: DebugServer,
}

impl DevelopmentMode {
    pub async fn start_dev_mode(&mut self, extension_path: &Path) -> Result<()> {
        // 启动文件监控
        self.watch_extension_files(extension_path).await?;
        
        // 启动调试服务器
        self.debug_server.start().await?;
        
        // 首次加载Extension
        self.extension_reloader.load_extension(extension_path).await?;
        
        println!("Development mode started. Extension will be reloaded on file changes.");
        
        // 监听文件变化
        loop {
            if let Ok(event) = self.file_watcher.recv().await {
                match event {
                    DebouncedEvent::Write(path) | DebouncedEvent::Create(path) => {
                        if self.is_source_file(&path) {
                            println!("File changed: {:?}, reloading extension...", path);
                            match self.extension_reloader.reload_extension().await {
                                Ok(_) => println!("Extension reloaded successfully"),
                                Err(e) => eprintln!("Failed to reload extension: {}", e),
                            }
                        }
                    },
                    _ => {},
                }
            }
        }
    }
}
```

## 4. 包格式和元数据

### 4.1. extension.toml元数据结构

**完整的元数据定义**：

```rust
// Extension元数据结构
#[derive(Debug, Deserialize, Serialize)]
pub struct ExtensionMetadata {
    pub extension: ExtensionInfo,
    pub compatibility: CompatibilityInfo,
    pub capabilities: CapabilityInfo,
    pub permissions: PermissionInfo,
    pub installation: InstallationInfo,
    pub configuration: Option<ConfigurationInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExtensionInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub license: String,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CompatibilityInfo {
    pub elfi_version: String,
    pub api_version: String,
    pub rust_version: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CapabilityInfo {
    pub block_types: Vec<String>,
    pub transformers: Vec<String>,
    pub renderers: Vec<String>,
    pub network_protocols: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PermissionInfo {
    pub file_system: Vec<String>,
    pub network: Vec<String>,
    pub external_commands: Vec<String>,
    pub system_info: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstallationInfo {
    pub platforms: Vec<String>,
    pub architectures: Vec<String>,
    pub system_dependencies: Vec<String>,
    pub pre_install: Option<String>,
    pub post_install: Option<String>,
}
```

### 4.2. 包验证和完整性检查

**数字签名验证机制**：

```rust
// 包验证器
pub struct PackageValidator {
    public_keys: HashMap<String, PublicKey>,
    trust_store: TrustStore,
}

impl PackageValidator {
    pub async fn validate_package(&self, package_path: &Path) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();
        
        // 1. 验证包结构
        self.validate_structure(package_path, &mut report)?;
        
        // 2. 验证元数据
        self.validate_metadata(package_path, &mut report)?;
        
        // 3. 验证数字签名
        self.validate_signature(package_path, &mut report).await?;
        
        // 4. 验证依赖关系
        self.validate_dependencies(package_path, &mut report)?;
        
        // 5. 安全扫描
        self.security_scan(package_path, &mut report).await?;
        
        Ok(report)
    }
    
    fn validate_structure(&self, package_path: &Path, report: &mut ValidationReport) -> Result<()> {
        let required_files = vec![
            "extension.toml",
            "Cargo.toml",
            "src/lib.rs",
        ];
        
        for file in required_files {
            let file_path = package_path.join(file);
            if !file_path.exists() {
                report.add_error(format!("Required file missing: {}", file));
            }
        }
        
        Ok(())
    }
    
    async fn validate_signature(&self, package_path: &Path, report: &mut ValidationReport) -> Result<()> {
        let signature_file = package_path.join("SIGNATURE");
        if !signature_file.exists() {
            report.add_warning("Package is not signed".to_string());
            return Ok(());
        }
        
        let signature = fs::read(&signature_file)?;
        let package_hash = self.calculate_package_hash(package_path)?;
        
        // 尝试验证签名
        let verified = self.verify_signature(&package_hash, &signature).await?;
        if !verified {
            report.add_error("Invalid package signature".to_string());
        } else {
            report.add_info("Package signature verified".to_string());
        }
        
        Ok(())
    }
}
```

这个实现机制文档专注于Extension系统的具体技术实现，包括接口定义、加载流程、安全机制和开发工具的代码级别细节，为开发者提供详细的实现指导。
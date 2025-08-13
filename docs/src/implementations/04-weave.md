# Weave 层 - 内容创作与协作

本文档详述 Weave 层的实现，专注于内容创作、IDE 集成、文件监听和协作机制。Weave 层为文档的主要贡献者提供高级 API，封装底层 CRDT 和网络复杂性。

## 1. Weave 层设计理念

### 1.1. 核心职责

Weave 层服务于文档内容的**主要创作者**：

- **作家和研究员**：撰写和编辑文档内容
- **程序员**：编写和管理代码块
- **技术文档工程师**：维护项目文档结构

### 1.2. 仓库模型抽象

采用仓库模型将复杂性抽象为简单接口：

```rust
// core/src/weave/mod.rs
pub struct WeaveApi {
    main: Arc<Main>,
    ide_integration: Arc<IdeIntegration>,
    conflict_ui: Arc<ConflictUi>,
    file_watcher: Arc<FileWatcher>,
}

impl WeaveApi {
    pub async fn new(main: Arc<Main>, config: WeaveConfig) -> Result<Self, WeaveError> {
        let ide_integration = Arc::new(
            IdeIntegration::new(main.clone(), &config.ide).await?
        );
        
        let conflict_ui = Arc::new(
            ConflictUi::new(&config.conflict_handling)
        );
        
        let file_watcher = Arc::new(
            FileWatcher::new(main.clone(), &config.file_watching).await?
        );
        
        Ok(Self {
            main,
            ide_integration,
            conflict_ui,
            file_watcher,
        })
    }
}
```

### 1.3. 文档句柄接口

提供面向对象的文档操作接口：

```rust
// core/src/weave/document_handle.rs
pub struct DocumentHandle {
    document_uri: String,
    main: Arc<Main>,
    subscribers: Arc<RwLock<Vec<Box<dyn DocumentObserver>>>>,
    history: Arc<RwLock<HistoryManager>>,
}

impl DocumentHandle {
    /// 修改文档内容的唯一入口
    pub async fn change<F, R>(&self, callback: F) -> Result<R, WeaveError>
    where
        F: FnOnce(&mut MutableDocument) -> Result<R, WeaveError>,
    {
        // 创建可变文档代理
        let mut mutable_doc = MutableDocument::new(&self.document_uri, &self.main).await?;
        
        // 执行用户回调
        let result = callback(&mut mutable_doc)?;
        
        // 提交所有变更
        mutable_doc.commit().await?;
        
        // 通知订阅者
        self.notify_subscribers().await;
        
        Ok(result)
    }
    
    /// 订阅文档变更
    pub async fn subscribe<F>(&self, callback: F) -> Result<SubscriptionId, WeaveError>
    where
        F: Fn(&DocumentState) + Send + Sync + 'static,
    {
        let observer = Box::new(CallbackObserver::new(callback));
        let subscription_id = SubscriptionId::new();
        
        let mut subscribers = self.subscribers.write().await;
        subscribers.push(observer);
        
        Ok(subscription_id)
    }
    
    /// 获取完整操作历史
    pub async fn get_history(&self) -> Result<HistoryGraph, WeaveError> {
        let history = self.history.read().await;
        Ok(history.build_graph().await?)
    }
    
    /// 时间旅行到指定版本
    pub async fn view_at(&self, heads: &[ChangeHash]) -> Result<ImmutableDocument, WeaveError> {
        self.main.view_document_at(&self.document_uri, heads).await
    }
}
```

### 1.4. 可变文档代理

```rust
// core/src/weave/mutable_document.rs
pub struct MutableDocument {
    document_uri: String,
    main: Arc<Main>,
    pending_operations: Vec<DocumentOperation>,
    snapshot: DocumentSnapshot,
}

impl MutableDocument {
    /// 添加新区块
    pub fn add_block(&mut self, block_type: BlockType, name: Option<String>) -> Result<BlockProxy, WeaveError> {
        let block_id = uuid::Uuid::new_v4();
        let operation = DocumentOperation::AddBlock {
            id: block_id,
            block_type: block_type.clone(),
            name: name.clone(),
            position: self.calculate_insert_position(),
        };
        
        self.pending_operations.push(operation);
        
        // 创建区块代理，支持链式操作
        Ok(BlockProxy::new(block_id, block_type, &mut self.pending_operations))
    }
    
    /// 删除区块
    pub fn delete_block(&mut self, block_id: &uuid::Uuid) -> Result<(), WeaveError> {
        let operation = DocumentOperation::DeleteBlock {
            id: *block_id,
        };
        
        self.pending_operations.push(operation);
        Ok(())
    }
    
    /// 获取区块代理进行编辑
    pub fn get_block(&mut self, block_id: &uuid::Uuid) -> Result<BlockProxy, WeaveError> {
        // 检查区块是否存在
        if !self.snapshot.has_block(block_id) {
            return Err(WeaveError::BlockNotFound(*block_id));
        }
        
        Ok(BlockProxy::existing(*block_id, &mut self.pending_operations))
    }
    
    /// 移动区块到新的父级下
    pub fn move_block(&mut self, block_id: &uuid::Uuid, new_parent: Option<uuid::Uuid>) -> Result<(), WeaveError> {
        let operation = DocumentOperation::MoveBlock {
            id: *block_id,
            new_parent,
        };
        
        self.pending_operations.push(operation);
        Ok(())
    }
    
    /// 提交所有变更
    pub(crate) async fn commit(self) -> Result<(), WeaveError> {
        for operation in self.pending_operations {
            match operation {
                DocumentOperation::AddBlock { id, block_type, name, position } => {
                    self.main.add_block(&self.document_uri, block_type, name).await?;
                }
                DocumentOperation::DeleteBlock { id } => {
                    self.main.delete_block(&self.document_uri, &id.to_string()).await?;
                }
                DocumentOperation::MoveBlock { id, new_parent } => {
                    self.main.move_block(
                        &self.document_uri, 
                        &id.to_string(), 
                        new_parent.map(|p| p.to_string())
                    ).await?;
                }
                DocumentOperation::UpdateBlockContent { id, content } => {
                    self.main.update_block_content(&self.document_uri, &id.to_string(), content).await?;
                }
            }
        }
        
        Ok(())
    }
}
```

### 1.5. 区块代理接口

```rust
// core/src/weave/block_proxy.rs
pub struct BlockProxy<'a> {
    block_id: uuid::Uuid,
    pending_operations: &'a mut Vec<DocumentOperation>,
}

impl<'a> BlockProxy<'a> {
    /// 设置区块内容
    pub fn set_content(self, content: BlockContent) -> Self {
        let operation = DocumentOperation::UpdateBlockContent {
            id: self.block_id,
            content,
        };
        self.pending_operations.push(operation);
        self
    }
    
    /// 设置区块名称
    pub fn set_name(self, name: String) -> Self {
        let operation = DocumentOperation::UpdateBlockMetadata {
            id: self.block_id,
            field: "name".to_string(),
            value: serde_json::Value::String(name),
        };
        self.pending_operations.push(operation);
        self
    }
    
    /// 添加标签
    pub fn add_tag(self, tag: String) -> Self {
        let operation = DocumentOperation::AddTag {
            id: self.block_id,
            tag,
        };
        self.pending_operations.push(operation);
        self
    }
    
    /// 设置父级
    pub fn set_parent(self, parent_id: Option<uuid::Uuid>) -> Self {
        let operation = DocumentOperation::UpdateBlockMetadata {
            id: self.block_id,
            field: "parent".to_string(),
            value: match parent_id {
                Some(id) => serde_json::Value::String(id.to_string()),
                None => serde_json::Value::Null,
            },
        };
        self.pending_operations.push(operation);
        self
    }
}
```

## 2. IDE 集成与文件监听

### 2.1. IDE 集成架构

```rust
// core/src/weave/ide_integration.rs
pub struct IdeIntegration {
    main: Arc<Main>,
    file_watcher: Arc<FileWatcher>,
    export_manager: Arc<ExportManager>,
    sync_validator: Arc<SyncValidator>,
}

impl IdeIntegration {
    /// 启动 IDE 集成服务
    pub async fn start_watch_mode(&self, config: WatchConfig) -> Result<WatchHandle, IdeError> {
        // 创建导出目录
        if !config.export_dir.exists() {
            tokio::fs::create_dir_all(&config.export_dir).await?;
        }
        
        // 初始导出所有匹配的区块
        self.initial_export(&config).await?;
        
        // 启动双向文件监听
        let watch_handle = self.file_watcher.start_watching(
            config.clone(),
            {
                let export_manager = self.export_manager.clone();
                let sync_validator = self.sync_validator.clone();
                
                move |event| {
                    let export_manager = export_manager.clone();
                    let sync_validator = sync_validator.clone();
                    
                    tokio::spawn(async move {
                        match event {
                            WatchEvent::DocumentChanged { uri, changes } => {
                                // 文档变更，重新导出影响的文件
                                if let Err(e) = export_manager.handle_document_change(&uri, &changes).await {
                                    tracing::error!("处理文档变更失败: {:?}", e);
                                }
                            }
                            WatchEvent::FileChanged { path, content } => {
                                // 外部文件变更，同步回文档
                                if let Err(e) = sync_validator.handle_file_change(&path, &content).await {
                                    tracing::error!("处理文件变更失败: {:?}", e);
                                }
                            }
                        }
                    });
                }
            }
        ).await?;
        
        tracing::info!("IDE 集成服务已启动");
        Ok(watch_handle)
    }
    
    /// 初始导出
    async fn initial_export(&self, config: &WatchConfig) -> Result<(), IdeError> {
        // 获取当前项目的所有文档
        let documents = self.main.list_project_documents(&config.project_root).await?;
        
        for doc_uri in documents {
            let handle = self.main.open(&doc_uri).await?;
            
            // 导出匹配的区块
            let blocks = handle.get_blocks_matching(&config.export_filter).await?;
            
            for block in blocks {
                self.export_manager.export_block(&doc_uri, &block, &config.export_dir).await?;
            }
        }
        
        Ok(())
    }
}
```

### 2.2. 文件监听服务

```rust
// core/src/weave/file_watcher.rs
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

pub struct FileWatcher {
    main: Arc<Main>,
    watcher: Option<RecommendedWatcher>,
    active_mappings: Arc<DashMap<PathBuf, BlockMapping>>,
}

#[derive(Debug, Clone)]
pub struct BlockMapping {
    pub document_uri: String,
    pub block_id: uuid::Uuid,
    pub export_path: PathBuf,
    pub last_sync: chrono::DateTime<chrono::Utc>,
    pub content_hash: String,
}

impl FileWatcher {
    pub async fn start_watching<F>(
        &mut self,
        config: WatchConfig,
        callback: F
    ) -> Result<WatchHandle, WatchError>
    where
        F: Fn(WatchEvent) + Send + Sync + 'static,
    {
        let callback = Arc::new(callback);
        let active_mappings = self.active_mappings.clone();
        let main = self.main.clone();
        
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        
        // 创建文件系统监听器
        let watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.try_send(event);
                }
            },
            notify::Config::default(),
        )?;
        
        // 监听导出目录
        watcher.watch(&config.export_dir, RecursiveMode::Recursive)?;
        
        // 启动事件处理循环
        let handle = tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event.kind {
                    notify::EventKind::Modify(notify::event::ModifyKind::Data(_)) => {
                        for path in event.paths {
                            if let Some(mapping) = active_mappings.get(&path) {
                                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                                    // 验证同步条件
                                    if Self::validate_sync_conditions(&mapping, &content) {
                                        callback(WatchEvent::FileChanged { path: path.clone(), content });
                                    }
                                }
                            }
                        }
                    }
                    _ => {} // 忽略其他事件类型
                }
            }
        });
        
        self.watcher = Some(watcher);
        
        Ok(WatchHandle::new(handle))
    }
    
    /// 验证同步条件
    fn validate_sync_conditions(mapping: &BlockMapping, content: &str) -> bool {
        // 检查时间窗口
        let now = chrono::Utc::now();
        if now.signed_duration_since(mapping.last_sync).num_seconds() < 5 {
            return false; // 避免频繁同步
        }
        
        // 检查内容变更
        let content_hash = sha256::digest(content);
        if content_hash == mapping.content_hash {
            return false; // 内容未变更
        }
        
        // 基本语法验证（针对代码文件）
        if mapping.export_path.extension().and_then(|e| e.to_str()) == Some("rs") {
            // Rust 语法基本验证
            if !content.chars().all(|c| c.is_ascii() || c.is_whitespace()) {
                return false;
            }
        }
        
        true
    }
}
```

### 2.3. 导出管理器

```rust
// core/src/weave/export_manager.rs
pub struct ExportManager {
    template_engine: Arc<TemplateEngine>,
    file_mappings: Arc<DashMap<String, Vec<ExportMapping>>>,
}

#[derive(Debug, Clone)]
pub struct ExportMapping {
    pub block_id: uuid::Uuid,
    pub export_path: PathBuf,
    pub template: Option<String>,
    pub format: ExportFormat,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Raw,           // 直接导出内容
    WithHeader,    // 添加生成头部
    Template(String), // 使用自定义模板
}

impl ExportManager {
    /// 导出区块到文件
    pub async fn export_block(
        &self,
        doc_uri: &str,
        block: &Block,
        export_dir: &Path
    ) -> Result<PathBuf, ExportError> {
        // 确定导出路径
        let export_path = self.determine_export_path(block, export_dir)?;
        
        // 准备导出内容
        let content = match &block.content {
            BlockContent::Text(text) => text.clone(),
            BlockContent::Structured(json) => json.to_string(),
            _ => return Err(ExportError::UnsupportedContentType),
        };
        
        // 应用格式化
        let formatted_content = self.format_content(&content, block, &ExportFormat::WithHeader)?;
        
        // 确保目录存在
        if let Some(parent) = export_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        // 写入文件
        tokio::fs::write(&export_path, formatted_content).await?;
        
        // 记录映射关系
        self.record_export_mapping(doc_uri, block.id, &export_path);
        
        tracing::info!("区块已导出: {} -> {}", 
            &block.id.to_string()[..8], export_path.display());
        
        Ok(export_path)
    }
    
    /// 格式化导出内容
    fn format_content(
        &self,
        content: &str,
        block: &Block,
        format: &ExportFormat
    ) -> Result<String, ExportError> {
        match format {
            ExportFormat::Raw => Ok(content.to_string()),
            ExportFormat::WithHeader => {
                let header = format!(
                    "// This file was automatically exported from {}.elf\n\
                     // Block: {} ({})\n\
                     // Last modified: {}\n\
                     // Do not edit directly - make changes in the .elf file\n\n",
                    "document", // TODO: 获取实际文档名
                    block.name.as_deref().unwrap_or("unnamed"),
                    block.id,
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
                );
                Ok(format!("{}{}", header, content))
            }
            ExportFormat::Template(template) => {
                let mut context = std::collections::HashMap::new();
                context.insert("content".to_string(), content.to_string());
                context.insert("block_id".to_string(), block.id.to_string());
                context.insert("block_name".to_string(), 
                    block.name.as_deref().unwrap_or("unnamed").to_string());
                
                self.template_engine.render(template, &context)
                    .map_err(ExportError::TemplateError)
            }
        }
    }
    
    /// 确定导出路径
    fn determine_export_path(&self, block: &Block, export_dir: &Path) -> Result<PathBuf, ExportError> {
        // 基于区块类型和元数据确定文件名和路径
        let filename = if let Some(name) = &block.name {
            let extension = match block.block_type {
                BlockType::Code => {
                    // 从元数据获取语言扩展名
                    block.metadata.language.as_ref()
                        .and_then(|lang| self.get_extension_for_language(lang))
                        .unwrap_or("txt")
                }
                BlockType::Markdown => "md",
                _ => "txt",
            };
            format!("{}.{}", name.replace(' ', "_"), extension)
        } else {
            format!("{}.txt", &block.id.to_string()[..8])
        };
        
        Ok(export_dir.join(filename))
    }
    
    fn get_extension_for_language(&self, language: &str) -> Option<&'static str> {
        match language.to_lowercase().as_str() {
            "rust" => Some("rs"),
            "python" => Some("py"),
            "javascript" => Some("js"),
            "typescript" => Some("ts"),
            "go" => Some("go"),
            "java" => Some("java"),
            "cpp" | "c++" => Some("cpp"),
            "c" => Some("c"),
            "shell" | "bash" => Some("sh"),
            _ => None,
        }
    }
}
```

### 2.4. 同步验证器

```rust
// core/src/weave/sync_validator.rs
pub struct SyncValidator {
    main: Arc<Main>,
    file_mappings: Arc<DashMap<PathBuf, BlockMapping>>,
}

impl SyncValidator {
    /// 处理外部文件变更
    pub async fn handle_file_change(
        &self,
        path: &PathBuf,
        content: &str
    ) -> Result<(), SyncError> {
        // 查找对应的区块映射
        let mapping = self.file_mappings.get(path)
            .ok_or_else(|| SyncError::NoMappingFound(path.clone()))?;
        
        // 验证同步条件
        if !self.validate_sync_conditions(&mapping, content).await? {
            return Err(SyncError::SyncConditionsNotMet);
        }
        
        // 检查是否为单区块完整导出
        if !self.is_complete_block_export(&mapping, content).await? {
            return Err(SyncError::PartialContentDetected);
        }
        
        // 提取实际内容（移除生成的头部）
        let cleaned_content = self.extract_original_content(content, &mapping)?;
        
        // 同步回文档
        let block_content = match mapping.block_type {
            BlockType::Code => BlockContent::Text(cleaned_content),
            BlockType::Markdown => BlockContent::Text(cleaned_content),
            _ => return Err(SyncError::UnsupportedBlockType),
        };
        
        self.main.update_block_content(
            &mapping.document_uri,
            &mapping.block_id.to_string(),
            block_content
        ).await?;
        
        // 更新映射信息
        self.update_mapping_after_sync(&mapping, content);
        
        tracing::info!("文件变更已同步: {} -> {}", 
            path.display(), &mapping.block_id.to_string()[..8]);
        
        Ok(())
    }
    
    /// 验证同步条件
    async fn validate_sync_conditions(
        &self,
        mapping: &BlockMapping,
        content: &str
    ) -> Result<bool, SyncError> {
        // 时间窗口检查
        let now = chrono::Utc::now();
        let time_since_last_sync = now.signed_duration_since(mapping.last_sync);
        if time_since_last_sync.num_seconds() < 5 {
            return Ok(false); // 防止无限循环
        }
        
        // 内容变更检查
        let new_hash = sha256::digest(content);
        if new_hash == mapping.content_hash {
            return Ok(false); // 内容未变更
        }
        
        // 结构一致性检查
        if !self.verify_file_structure(mapping, content).await? {
            return Ok(false);
        }
        
        // 语法验证
        if let Some(language) = &mapping.language {
            if !self.validate_syntax(language, content).await? {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// 验证文件结构
    async fn verify_file_structure(
        &self,
        mapping: &BlockMapping,
        content: &str
    ) -> Result<bool, SyncError> {
        // 检查是否包含生成标记
        if content.contains("automatically exported from") {
            return Ok(true);
        }
        
        // 检查文件路径和名称是否与预期一致
        let expected_filename = format!("{}.{}", 
            mapping.block_name.as_deref().unwrap_or("unnamed"),
            mapping.file_extension.as_deref().unwrap_or("txt")
        );
        
        if mapping.export_path.file_name() != Some(expected_filename.as_ref()) {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// 语法验证
    async fn validate_syntax(
        &self,
        language: &str,
        content: &str
    ) -> Result<bool, SyncError> {
        match language.to_lowercase().as_str() {
            "rust" => {
                // Rust 基本语法检查
                self.validate_rust_syntax(content).await
            }
            "python" => {
                // Python 基本语法检查
                self.validate_python_syntax(content).await
            }
            _ => Ok(true), // 其他语言暂时跳过语法验证
        }
    }
    
    /// 提取原始内容
    fn extract_original_content(&self, content: &str, mapping: &BlockMapping) -> Result<String, SyncError> {
        // 移除自动生成的头部
        let lines: Vec<&str> = content.lines().collect();
        let mut start_index = 0;
        
        // 查找内容开始位置
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("//") && line.contains("Do not edit directly") {
                start_index = i + 1;
                // 跳过空行
                while start_index < lines.len() && lines[start_index].trim().is_empty() {
                    start_index += 1;
                }
                break;
            }
        }
        
        if start_index < lines.len() {
            Ok(lines[start_index..].join("\n"))
        } else {
            Ok(content.to_string()) // 如果没有找到头部，返回原始内容
        }
    }
}
```

## 3. 冲突解决 UI

### 3.1. 冲突检测与呈现

```rust
// core/src/weave/conflict_ui.rs
pub struct ConflictUi {
    conflict_resolver: Arc<ConflictResolver>,
    active_conflicts: Arc<DashMap<uuid::Uuid, ConflictSession>>,
}

#[derive(Debug, Clone)]
pub struct ConflictSession {
    pub block_id: uuid::Uuid,
    pub conflict_type: ConflictType,
    pub participants: Vec<String>,
    pub versions: HashMap<String, ConflictVersion>,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub status: ConflictStatus,
}

#[derive(Debug, Clone)]
pub struct ConflictVersion {
    pub author: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation_id: String,
}

#[derive(Debug, Clone)]
pub enum ConflictStatus {
    Detected,
    UserNotified,
    InResolution,
    Resolved,
    Escalated,
}

impl ConflictUi {
    /// 处理检测到的冲突
    pub async fn handle_conflict(&self, conflict: ConflictInfo) -> Result<(), ConflictError> {
        let session = ConflictSession {
            block_id: conflict.block_id,
            conflict_type: conflict.conflict_type.clone(),
            participants: conflict.participants,
            versions: self.extract_conflict_versions(&conflict).await?,
            detected_at: conflict.detected_at,
            status: ConflictStatus::Detected,
        };
        
        // 根据冲突类型选择处理策略
        match conflict.conflict_type {
            ConflictType::ConcurrentTextEdit => {
                self.handle_text_conflict(&session).await?;
            }
            ConflictType::MetadataConflict => {
                self.handle_metadata_conflict(&session).await?;
            }
            ConflictType::StructuralConflict => {
                self.handle_structural_conflict(&session).await?;
            }
        }
        
        self.active_conflicts.insert(conflict.block_id, session);
        
        Ok(())
    }
    
    /// 处理文本编辑冲突
    async fn handle_text_conflict(&self, session: &ConflictSession) -> Result<(), ConflictError> {
        // 尝试自动合并
        if let Ok(merged) = self.attempt_auto_merge(session).await {
            tracing::info!("文本冲突自动解决: {}", &session.block_id.to_string()[..8]);
            return Ok(());
        }
        
        // 生成冲突标记
        let conflict_marker = self.generate_conflict_marker(session)?;
        
        // 创建用户通知
        self.create_user_notification(session, &conflict_marker).await?;
        
        Ok(())
    }
    
    /// 尝试自动合并
    async fn attempt_auto_merge(&self, session: &ConflictSession) -> Result<String, ConflictError> {
        let versions: Vec<_> = session.versions.values().collect();
        
        if versions.len() == 2 {
            // 尝试三路合并
            let base = ""; // TODO: 获取公共祖先版本
            let merged = self.three_way_merge(base, &versions[0].content, &versions[1].content)?;
            
            if !merged.contains("<<<<<<<") {
                // 合并成功，没有冲突标记
                return Ok(merged);
            }
        }
        
        Err(ConflictError::AutoMergesFailed)
    }
    
    /// 生成冲突标记
    fn generate_conflict_marker(&self, session: &ConflictSession) -> Result<String, ConflictError> {
        let mut marker = String::new();
        let sorted_versions: Vec<_> = session.versions.iter()
            .map(|(author, version)| (author, version))
            .collect();
        
        if sorted_versions.len() == 2 {
            let (author1, version1) = sorted_versions[0];
            let (author2, version2) = sorted_versions[1];
            
            marker.push_str(&format!("<<<<<<< {} ({})\n", author1, version1.timestamp.format("%H:%M")));
            marker.push_str(&version1.content);
            marker.push_str("\n=======\n");
            marker.push_str(&version2.content);
            marker.push_str(&format!("\n>>>>>>> {} ({})\n", author2, version2.timestamp.format("%H:%M")));
        } else {
            // 多方冲突
            for (author, version) in sorted_versions {
                marker.push_str(&format!("<<<<<<< {}\n", author));
                marker.push_str(&version.content);
                marker.push_str("\n=======\n");
            }
            marker.push_str(">>>>>>>\n");
        }
        
        Ok(marker)
    }
    
    /// 创建用户通知
    async fn create_user_notification(
        &self,
        session: &ConflictSession,
        conflict_marker: &str
    ) -> Result<(), ConflictError> {
        let notification = ConflictNotification {
            block_id: session.block_id,
            title: format!("区块 '{}' 存在编辑冲突", 
                session.block_id.to_string().get(..8).unwrap_or("unknown")),
            description: format!(
                "{} 个用户同时修改了这个区块。请选择要保留的版本或手动合并。",
                session.participants.len()
            ),
            participants: session.participants.clone(),
            conflict_content: conflict_marker.to_string(),
            actions: vec![
                ConflictAction::AcceptVersion { author: session.participants[0].clone() },
                ConflictAction::AcceptVersion { author: session.participants[1].clone() },
                ConflictAction::ManualMerge,
                ConflictAction::Defer,
            ],
        };
        
        // 发送到用户界面
        self.send_notification(notification).await?;
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ConflictNotification {
    pub block_id: uuid::Uuid,
    pub title: String,
    pub description: String,
    pub participants: Vec<String>,
    pub conflict_content: String,
    pub actions: Vec<ConflictAction>,
}

#[derive(Debug, Clone)]
pub enum ConflictAction {
    AcceptVersion { author: String },
    ManualMerge,
    Defer,
    RequestHelp,
}
```

### 3.2. 三路合并实现

```rust
// core/src/weave/three_way_merge.rs
impl ConflictUi {
    /// 执行三路合并
    fn three_way_merge(&self, base: &str, left: &str, right: &str) -> Result<String, ConflictError> {
        let base_lines: Vec<&str> = base.lines().collect();
        let left_lines: Vec<&str> = left.lines().collect();
        let right_lines: Vec<&str> = right.lines().collect();
        
        // 计算差异
        let left_diff = self.compute_diff(&base_lines, &left_lines);
        let right_diff = self.compute_diff(&base_lines, &right_lines);
        
        // 合并差异
        let mut result = Vec::new();
        let mut base_idx = 0;
        let mut left_idx = 0;
        let mut right_idx = 0;
        
        while base_idx < base_lines.len() || left_idx < left_diff.len() || right_idx < right_diff.len() {
            // 简化的合并逻辑 - 实际实现会更复杂
            if let Some(left_change) = left_diff.get(left_idx) {
                if let Some(right_change) = right_diff.get(right_idx) {
                    if left_change.line_number == right_change.line_number {
                        // 同一行的并发修改 - 冲突
                        result.push(format!("<<<<<<< left\n{}\n=======\n{}\n>>>>>>> right", 
                            left_change.content, right_change.content));
                        left_idx += 1;
                        right_idx += 1;
                    } else if left_change.line_number < right_change.line_number {
                        // 左侧变更
                        result.push(left_change.content.to_string());
                        left_idx += 1;
                    } else {
                        // 右侧变更
                        result.push(right_change.content.to_string());
                        right_idx += 1;
                    }
                } else {
                    // 只有左侧变更
                    result.push(left_change.content.to_string());
                    left_idx += 1;
                }
            } else if let Some(right_change) = right_diff.get(right_idx) {
                // 只有右侧变更
                result.push(right_change.content.to_string());
                right_idx += 1;
            } else {
                // 没有变更，使用基础版本
                if base_idx < base_lines.len() {
                    result.push(base_lines[base_idx].to_string());
                    base_idx += 1;
                } else {
                    break;
                }
            }
        }
        
        Ok(result.join("\n"))
    }
    
    /// 计算文本差异
    fn compute_diff(&self, base: &[&str], modified: &[&str]) -> Vec<DiffChange> {
        // 简化的差异算法 - 实际使用更高级的算法如 Myers
        let mut changes = Vec::new();
        
        for (i, (base_line, modified_line)) in base.iter().zip(modified.iter()).enumerate() {
            if base_line != modified_line {
                changes.push(DiffChange {
                    line_number: i,
                    change_type: DiffChangeType::Modified,
                    content: modified_line.to_string(),
                });
            }
        }
        
        // 处理新增行
        if modified.len() > base.len() {
            for (i, line) in modified[base.len()..].iter().enumerate() {
                changes.push(DiffChange {
                    line_number: base.len() + i,
                    change_type: DiffChangeType::Added,
                    content: line.to_string(),
                });
            }
        }
        
        changes
    }
}

#[derive(Debug, Clone)]
struct DiffChange {
    line_number: usize,
    change_type: DiffChangeType,
    content: String,
}

#[derive(Debug, Clone)]
enum DiffChangeType {
    Added,
    Removed,
    Modified,
}
```

## 4. 层级结构操作 API

### 4.3. 层级导航 API

```rust
// core/src/weave/hierarchy.rs
impl DocumentHandle {
    /// 获取块的父级
    pub async fn get_parent(&self, block_id: &uuid::Uuid) -> Result<Option<Block>, WeaveError> {
        // 通过 Main 接口获取文档状态
        let document = self.main.get_document_state(&self.document_uri).await?;
        
        let block = document.get_block(block_id)
            .ok_or_else(|| WeaveError::BlockNotFound(*block_id))?;
        
        if let Some(parent_id) = block.metadata.parent {
            let parent = document.get_block(&parent_id)
                .ok_or_else(|| WeaveError::BlockNotFound(parent_id))?;
            Ok(Some(parent.clone()))
        } else {
            Ok(None)
        }
    }
    
    /// 获取块的所有直接子级
    pub async fn get_children(&self, block_id: &uuid::Uuid) -> Result<Vec<Block>, WeaveError> {
        let document = self.main.get_document_state(&self.document_uri).await?;
        
        let children: Vec<Block> = document.blocks()
            .filter(|block| block.metadata.parent == Some(*block_id))
            .cloned()
            .collect();
        
        Ok(children)
    }
    
    /// 获取完整的文档树结构
    pub async fn get_tree(&self) -> Result<HierarchyTree, WeaveError> {
        let document = self.main.get_document_state(&self.document_uri).await?;
        
        // 找到所有根级块（没有父级的块）
        let root_blocks: Vec<_> = document.blocks()
            .filter(|block| block.metadata.parent.is_none())
            .cloned()
            .collect();
        
        // 递归构建树结构
        let tree = HierarchyTree::new();
        for root_block in root_blocks {
            let subtree = self.build_subtree(&document, &root_block).await?;
            tree.add_subtree(subtree);
        }
        
        Ok(tree)
    }
    
    /// 递归构建子树
    async fn build_subtree(&self, document: &DocumentState, block: &Block) -> Result<HierarchyNode, WeaveError> {
        let children = self.get_children(&block.id).await?;
        let mut child_nodes = Vec::new();
        
        for child in children {
            let child_node = self.build_subtree(document, &child).await?;
            child_nodes.push(child_node);
        }
        
        Ok(HierarchyNode {
            block: block.clone(),
            children: child_nodes,
        })
    }
}
```

## 验证清单

### Weave 接口完整性
- [ ] 文档句柄提供完整的 CRUD 操作
- [ ] 可变文档代理支持链式操作
- [ ] 订阅机制正确处理状态变更通知
- [ ] 历史管理和时间旅行功能正常

### IDE 集成稳定性
- [ ] 文件监听服务正确处理变更事件
- [ ] 双向同步机制防止无限循环
- [ ] 导出格式和模板系统工作正常
- [ ] 同步条件验证准确可靠

### 冲突解决有效性
- [ ] 冲突检测覆盖所有并发场景
- [ ] 自动合并算法正确实现
- [ ] 用户界面清晰展示冲突信息
- [ ] 手动解决工作流完整

### 层级结构正确性
- [ ] 父子关系操作正确维护
- [ ] 树结构构建算法正确
- [ ] 移动操作不破坏引用完整性
- [ ] 性能在大型文档中可接受

### 性能和用户体验
- [ ] 实时同步延迟在合理范围
- [ ] 文件监听不影响系统性能
- [ ] 错误处理和用户提示友好
- [ ] 内存使用随文档规模合理增长

这个 Weave 层实现为内容创作者提供了强大而直观的编辑体验，同时保持了与底层 CRDT 系统的完整集成。
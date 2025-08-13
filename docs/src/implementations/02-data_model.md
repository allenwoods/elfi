# 核心数据结构、验证、存储与同步

本文档详细描述 ELFI 的核心数据模型、验证机制、存储策略和同步机制的实现方案。

## 1. CRDT 数据模型设计

### 1.1. 设计理念：事件溯源与协作优先

ELFI 的数据模型基于两个核心原则：

**事件溯源 (Event Sourcing)**：
- 文档状态通过不可变操作日志重建，而非静态快照
- 每个编辑操作都是一个原子事件，具有完整的因果链
- 支持时间旅行、精确差异比较和明确变更归因

**无冲突复制数据类型 (CRDT)**：
- 支持并发、无协调的分布式编辑
- 自动合并大部分冲突，保证最终一致性
- 保留完整操作历史，支持语义冲突解决

### 1.2. 技术选型：基于 Automerge 的全历史模型

选择 Automerge 作为 CRDT 实现的核心原因：

```rust
// core/src/types/document.rs
use automerge::{AutoCommit, ObjType, ScalarValue};
use serde::{Deserialize, Serialize};

/// ELFI 文档的根数据结构
#[derive(Debug, Clone)]
pub struct ElfiDocument {
    /// Automerge CRDT 文档实例
    inner: AutoCommit,
    /// 文档元数据
    metadata: DocumentMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub id: String,
    pub title: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub authors: Vec<String>,
    pub version: String,
}
```

**全历史保留的优势**：
- 完整的操作日志支持审计和版本控制
- 可实现类似 Git 的分支和合并机制
- 支持冲突的透明化处理，不丢失任何编辑信息

## 2. 块级数据结构

### 2.1. 扁平化块列表设计

采用扁平的块列表结构，通过元数据构建层级关系：

```rust
// core/src/types/block.rs
use uuid::Uuid;
use serde_json::Value as JsonValue;

/// 文档块的核心数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// 全局唯一标识符 (UUID)
    pub id: Uuid,
    /// 人类可读的块名称 (可选，文档内唯一)
    pub name: Option<String>,
    /// 块类型，决定内容的解析和渲染方式
    pub block_type: BlockType,
    /// 主要内容（对文本类型是 CRDT Text）
    pub content: BlockContent,
    /// 扩展元数据
    pub metadata: BlockMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    /// 标准 Markdown 文本
    Markdown,
    /// 程序代码
    Code,
    /// 跨文档引用
    Link,
    /// 内容转换配方
    Recipe,
    /// 文档级元数据
    Metadata,
    /// 自定义类型 (由插件处理)
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockContent {
    /// 文本内容 (使用 CRDT Text)
    Text(String), // 在内部实现时使用 automerge::Text
    /// 结构化内容 (JSON)
    Structured(JsonValue),
    /// Link Block 特殊内容
    Link(LinkContent),
    /// Recipe 配置
    Recipe(RecipeConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    /// 父块 ID (构建层级结构)
    pub parent: Option<Uuid>,
    /// 代码语言 (适用于 Code 类型)
    pub language: Option<String>,
    /// 是否需要交互式渲染
    pub interactive: bool,
    /// 标签数组
    pub tags: Vec<String>,
    /// 描述信息
    pub description: Option<String>,
    /// 块所有者 (协作权限控制)
    pub owner: Option<String>,
    /// 合并策略
    pub merge_method: MergeMethod,
    /// 其他扩展属性
    pub extra: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeMethod {
    /// CRDT 自动合并
    Crdt,
    /// 手动解决冲突
    Manual,
    /// 最后写入者获胜
    LastWriterWins,
}
```

### 2.2. 层级结构实现：邻接列表模型

通过 `parent` 元数据字段实现逻辑层级：

```rust
// core/src/types/hierarchy.rs
impl ElfiDocument {
    /// 获取块的所有直接子块
    pub fn get_children(&self, parent_id: &Uuid) -> Vec<&Block> {
        self.blocks()
            .filter(|block| block.metadata.parent == Some(*parent_id))
            .collect()
    }
    
    /// 获取块的父块
    pub fn get_parent(&self, block_id: &Uuid) -> Option<&Block> {
        let block = self.get_block(block_id)?;
        let parent_id = block.metadata.parent?;
        self.get_block(&parent_id)
    }
    
    /// 构建完整的层级树
    pub fn build_hierarchy(&self) -> HierarchyTree {
        let mut tree = HierarchyTree::new();
        let root_blocks: Vec<_> = self.blocks()
            .filter(|block| block.metadata.parent.is_none())
            .collect();
        
        for block in root_blocks {
            tree.add_subtree(block, self);
        }
        
        tree
    }
}

#[derive(Debug, Clone)]
pub struct HierarchyTree {
    root: HierarchyNode,
}

#[derive(Debug, Clone)]
pub struct HierarchyNode {
    block: Block,
    children: Vec<HierarchyNode>,
}
```

**协作优势**：
- 移动块只需修改 `parent` 字段，操作原子且冲突概率低
- 扁平存储简化 CRDT 合并逻辑
- 层级关系在应用层重建，不影响底层数据同步

## 3. Link Block 与跨文档引用

### 3.1. Link Block 数据结构

```rust
// core/src/types/link.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkContent {
    /// 目标 URI (elf://[user/]repo/doc[#block-name])
    pub target: String,
    /// 引用类型
    pub ref_type: ReferenceType,
    /// 显示文本
    pub display_text: Option<String>,
    /// 引用描述
    pub description: Option<String>,
    /// 缓存策略
    pub cache_policy: CachePolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceType {
    /// 包含目标内容 (默认)
    Include,
    /// 仅引用链接
    Reference,
    /// 嵌入并可编辑
    Embed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CachePolicy {
    /// 内容变更时更新
    OnChange,
    /// 总是获取最新
    AlwaysFresh,
    /// 手动控制
    Manual,
}
```

### 3.2. URI 引用格式

标准化的跨文档引用格式：

```rust
// core/src/link/uri.rs
#[derive(Debug, Clone, PartialEq)]
pub struct ElfUri {
    /// 用户或组织名 (可选)
    pub user: Option<String>,
    /// 仓库名
    pub repo: String,
    /// 文档名
    pub document: String,
    /// 块名称 (可选)
    pub block_name: Option<String>,
}

impl ElfUri {
    /// 解析 ELF URI 格式
    pub fn parse(uri: &str) -> Result<Self, UriParseError> {
        // elf://[user/]repo/doc[#block-name]
        if !uri.starts_with("elf://") {
            return Err(UriParseError::InvalidScheme);
        }
        
        let path_part = &uri[6..]; // 移除 "elf://"
        
        // 分离区块名称部分
        let (path, block_name) = if let Some(hash_pos) = path_part.find('#') {
            let (path, block) = path_part.split_at(hash_pos);
            (path, Some(block[1..].to_string()))
        } else {
            (path_part, None)
        };
        
        // 解析路径部分
        let path_segments: Vec<&str> = path.split('/').collect();
        
        match path_segments.len() {
            2 => Ok(ElfUri {
                user: None,
                repo: path_segments[0].to_string(),
                document: path_segments[1].to_string(),
                block_name,
            }),
            3 => Ok(ElfUri {
                user: Some(path_segments[0].to_string()),
                repo: path_segments[1].to_string(),
                document: path_segments[2].to_string(),
                block_name,
            }),
            _ => Err(UriParseError::InvalidFormat),
        }
    }
    
    /// 标准化 URI (处理相对引用)
    pub fn normalize(&self, base_uri: &ElfUri) -> String {
        match (&self.user, &base_uri.user) {
            (None, Some(base_user)) => {
                // 相对引用，继承基础 URI 的用户
                format!("elf://{}/{}/{}{}",
                    base_user, self.repo, self.document,
                    self.block_name.as_ref().map(|b| format!("#{}", b)).unwrap_or_default())
            },
            _ => self.to_string()
        }
    }
}
```

## 4. Recipe 系统数据模型

### 4.1. Recipe 配置结构

```rust
// core/src/types/recipe.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeConfig {
    /// Recipe 名称
    pub name: String,
    /// 版本号
    pub version: String,
    /// 描述
    pub description: String,
    /// 跨文档引用
    #[serde(default)]
    pub references: Vec<CrossDocumentReference>,
    /// 内容选择器
    pub selector: BlockSelector,
    /// 转换规则
    pub transform: Vec<TransformRule>,
    /// 输出配置
    pub output: OutputConfig,
    /// 错误处理策略
    #[serde(default)]
    pub error_handling: ErrorHandlingConfig,
    /// 执行配置
    #[serde(default)]
    pub execution: ExecutionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDocumentReference {
    /// 源 URI
    pub source: String,
    /// 本地别名
    pub target: String,
    /// 缓存策略
    pub cache_policy: CachePolicy,
    /// 解析模式
    pub resolve_mode: ResolveMode,
    /// 内容模板
    pub template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolveMode {
    /// 延迟解析
    Lazy,
    /// 立即解析
    Eager,
    /// 预取
    Prefetch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSelector {
    /// 块类型过滤
    #[serde(default)]
    pub types: Vec<String>,
    /// 标签过滤
    #[serde(default)]
    pub tags: Vec<String>,
    /// 块名称过滤 (支持通配符)
    #[serde(default)]
    pub names: Vec<String>,
    /// 引用过滤
    #[serde(default)]
    pub references: Vec<String>,
    /// 元数据条件
    #[serde(default)]
    pub metadata: HashMap<String, JsonValue>,
}
```

## 5. 冲突解决机制

### 5.1. 块级语义冲突解决

基于块类型的分层冲突解决策略：

```rust
// core/src/conflict/resolver.rs
#[derive(Debug)]
pub struct ConflictResolver {
    strategies: HashMap<BlockType, Box<dyn ConflictStrategy>>,
}

pub trait ConflictStrategy: Send + Sync {
    fn resolve_conflict(
        &self,
        block: &Block,
        conflicts: &HashMap<automerge::OpId, ScalarValue>
    ) -> ConflictResolution;
}

#[derive(Debug, Clone)]
pub enum ConflictResolution {
    /// 自动解决，使用指定值
    AutoResolve(ScalarValue),
    /// 需要用户手动解决
    RequireManualResolution {
        conflict_marker: String,
        metadata: HashMap<String, JsonValue>,
    },
    /// 合并所有值
    MergeAll(Vec<ScalarValue>),
    /// 使用最后写入者获胜
    LastWriterWins,
}

impl ConflictResolver {
    pub fn new() -> Self {
        let mut strategies = HashMap::new();
        
        // Markdown 块：Text CRDT 通常能自动处理
        strategies.insert(BlockType::Markdown, Box::new(TextMergeStrategy));
        
        // 代码块：需要更谨慎的合并策略
        strategies.insert(BlockType::Code, Box::new(CodeMergeStrategy));
        
        // Link 块：引用完整性很重要
        strategies.insert(BlockType::Link, Box::new(LinkMergeStrategy));
        
        // Recipe 块：复杂配置需要手动解决
        strategies.insert(BlockType::Recipe, Box::new(RecipeMergeStrategy));
        
        Self { strategies }
    }
}

/// 代码块的冲突解决策略
struct CodeMergeStrategy;

impl ConflictStrategy for CodeMergeStrategy {
    fn resolve_conflict(
        &self,
        block: &Block,
        conflicts: &HashMap<automerge::OpId, ScalarValue>
    ) -> ConflictResolution {
        // 对于代码块，尝试结构化合并
        if let Some(language) = &block.metadata.language {
            // 基于语言的智能合并（如果可能）
            if self.can_smart_merge(language, conflicts) {
                return self.perform_smart_merge(language, conflicts);
            }
        }
        
        // 无法自动合并，生成冲突标记
        let conflict_marker = self.generate_conflict_marker(conflicts);
        let mut metadata = HashMap::new();
        metadata.insert("conflict".to_string(), JsonValue::Bool(true));
        metadata.insert("conflict_count".to_string(), JsonValue::from(conflicts.len()));
        
        ConflictResolution::RequireManualResolution {
            conflict_marker,
            metadata,
        }
    }
}
```

### 5.2. 冲突检测与通知

```rust
// core/src/conflict/detector.rs
#[derive(Debug)]
pub struct ConflictDetector {
    active_conflicts: DashMap<Uuid, ConflictInfo>,
    subscribers: Vec<Box<dyn ConflictObserver>>,
}

#[derive(Debug, Clone)]
pub struct ConflictInfo {
    pub block_id: Uuid,
    pub field_path: Vec<String>,
    pub conflict_type: ConflictType,
    pub participants: Vec<String>, // 冲突参与者
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub resolution_deadline: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub enum ConflictType {
    /// 并发文本编辑
    ConcurrentTextEdit,
    /// 元数据冲突
    MetadataConflict,
    /// 引用完整性冲突
    ReferenceIntegrityConflict,
    /// 结构化数据冲突
    StructuredDataConflict,
}

pub trait ConflictObserver: Send + Sync {
    fn on_conflict_detected(&self, conflict: &ConflictInfo);
    fn on_conflict_resolved(&self, block_id: &Uuid);
}
```

## 6. 存储与同步机制

### 6.1. Zenoh 网络集成

基于 Eclipse Zenoh 的分布式同步：

```rust
// core/src/storage/zenoh_adapter.rs
use zenoh::{Session, config::Config};

pub struct ZenohAdapter {
    session: Arc<Session>,
    document_subscribers: DashMap<String, zenoh::Subscriber<'static, ()>>,
    operation_publishers: DashMap<String, zenoh::Publisher<'static>>,
}

impl ZenohAdapter {
    pub async fn new(config: Config) -> Result<Self, ZenohError> {
        let session = Arc::new(zenoh::open(config).await?);
        
        Ok(Self {
            session,
            document_subscribers: DashMap::new(),
            operation_publishers: DashMap::new(),
        })
    }
    
    /// 发布文档操作到 Zenoh 网络
    pub async fn publish_operation(
        &self,
        doc_id: &str,
        operation: &CrdtOperation
    ) -> Result<(), ZenohError> {
        let key = format!("/elf/docs/{}/ops", doc_id);
        
        // 获取或创建发布者
        let publisher = self.operation_publishers
            .entry(key.clone())
            .or_insert_with(|| self.session.declare_publisher(&key));
        
        // 序列化操作并发布
        let serialized = bincode::serialize(operation)?;
        publisher.put(serialized).await?;
        
        Ok(())
    }
    
    /// 订阅文档变更
    pub async fn subscribe_document(
        &self,
        doc_id: &str,
        callback: impl Fn(CrdtOperation) + Send + Sync + 'static
    ) -> Result<(), ZenohError> {
        let key = format!("/elf/docs/{}/ops", doc_id);
        let callback = Arc::new(callback);
        
        let subscriber = self.session
            .declare_subscriber(&key)
            .callback({
                let callback = callback.clone();
                move |sample| {
                    if let Ok(operation) = bincode::deserialize::<CrdtOperation>(sample.payload()) {
                        callback(operation);
                    }
                }
            })
            .await?;
        
        self.document_subscribers.insert(key, subscriber);
        Ok(())
    }
    
    /// 查询历史操作
    pub async fn query_history(
        &self,
        doc_id: &str,
        since: Option<chrono::DateTime<chrono::Utc>>
    ) -> Result<Vec<CrdtOperation>, ZenohError> {
        let key = format!("/elf/docs/{}/ops", doc_id);
        let selector = if let Some(timestamp) = since {
            format!("{}?_time=[{},*]", key, timestamp.timestamp())
        } else {
            key
        };
        
        let mut operations = Vec::new();
        let replies = self.session.get(&selector).await?;
        
        while let Ok(reply) = replies.recv_async().await {
            if let Ok(operation) = bincode::deserialize::<CrdtOperation>(reply.payload()) {
                operations.push(operation);
            }
        }
        
        // 按时间戳排序
        operations.sort_by_key(|op| op.timestamp);
        Ok(operations)
    }
}
```

### 6.2. 本地存储与缓存

多级缓存架构：

```rust
// core/src/storage/cache.rs
pub struct MultiLevelCache {
    /// L1: 内存缓存 (最近访问的文档)
    memory_cache: Arc<DashMap<String, Arc<ElfiDocument>>>,
    /// L2: 本地持久化缓存
    disk_cache: Arc<DiskCache>,
    /// L3: 网络获取
    network: Arc<ZenohAdapter>,
    /// 缓存策略配置
    config: CacheConfig,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub memory_limit_mb: usize,
    pub disk_limit_gb: usize,
    pub ttl_seconds: u64,
    pub prefetch_related: bool,
}

impl MultiLevelCache {
    /// 获取文档，自动回退到不同缓存级别
    pub async fn get_document(&self, doc_id: &str) -> Result<Arc<ElfiDocument>, CacheError> {
        // L1: 内存缓存
        if let Some(doc) = self.memory_cache.get(doc_id) {
            return Ok(doc.clone());
        }
        
        // L2: 磁盘缓存
        if let Ok(doc) = self.disk_cache.get(doc_id).await {
            // 升级到内存缓存
            let arc_doc = Arc::new(doc);
            self.memory_cache.insert(doc_id.to_string(), arc_doc.clone());
            return Ok(arc_doc);
        }
        
        // L3: 网络获取
        let operations = self.network.query_history(doc_id, None).await?;
        let doc = self.rebuild_document_from_operations(operations)?;
        
        // 存储到各级缓存
        let arc_doc = Arc::new(doc);
        self.memory_cache.insert(doc_id.to_string(), arc_doc.clone());
        self.disk_cache.store(doc_id, &arc_doc).await?;
        
        Ok(arc_doc)
    }
    
    /// 智能预取相关文档
    pub async fn prefetch_related(&self, doc: &ElfiDocument) -> Result<(), CacheError> {
        if !self.config.prefetch_related {
            return Ok(());
        }
        
        // 提取所有 Link Block 的引用
        let references: Vec<String> = doc.blocks()
            .filter_map(|block| {
                if let BlockContent::Link(link_content) = &block.content {
                    Some(link_content.target.clone())
                } else {
                    None
                }
            })
            .collect();
        
        // 并行预取引用的文档
        let futures: Vec<_> = references.into_iter()
            .map(|ref_uri| {
                let cache = self.clone();
                async move {
                    if let Ok(parsed) = ElfUri::parse(&ref_uri) {
                        let doc_id = format!("{}/{}", parsed.repo, parsed.document);
                        let _ = cache.get_document(&doc_id).await;
                    }
                }
            })
            .collect();
        
        futures::future::join_all(futures).await;
        Ok(())
    }
}
```

### 6.3. 跨文档引用解析

```rust
// core/src/link/resolver.rs
pub struct LinkResolver {
    cache: Arc<MultiLevelCache>,
    uri_parser: UriParser,
    validation_cache: Arc<DashMap<String, ReferenceValidation>>,
}

#[derive(Debug, Clone)]
pub struct ReferenceValidation {
    pub is_valid: bool,
    pub last_checked: chrono::DateTime<chrono::Utc>,
    pub error_message: Option<String>,
    pub content_hash: Option<String>,
}

impl LinkResolver {
    /// 解析跨文档引用
    pub async fn resolve_reference(&self, uri: &str) -> Result<ResolvedReference, LinkError> {
        let parsed = self.uri_parser.parse(uri)?;
        
        // 检查验证缓存
        if let Some(validation) = self.validation_cache.get(uri) {
            if !validation.is_valid {
                return Err(LinkError::InvalidReference(
                    validation.error_message.clone().unwrap_or_default()
                ));
            }
        }
        
        // 获取目标文档
        let doc_id = format!("{}/{}", parsed.repo, parsed.document);
        let document = self.cache.get_document(&doc_id).await?;
        
        // 提取目标内容
        let content = if let Some(block_name) = parsed.block_name {
            // 查找指定区块
            document.blocks()
                .find(|block| block.name.as_ref() == Some(&block_name))
                .map(|block| match &block.content {
                    BlockContent::Text(text) => text.clone(),
                    BlockContent::Structured(json) => json.to_string(),
                    other => format!("{:?}", other),
                })
                .ok_or_else(|| LinkError::BlockNotFound {
                    block_name,
                    document_id: doc_id.clone(),
                })?
        } else {
            // 返回整个文档的摘要
            self.generate_document_summary(&document)
        };
        
        // 更新验证缓存
        self.validation_cache.insert(uri.to_string(), ReferenceValidation {
            is_valid: true,
            last_checked: chrono::Utc::now(),
            error_message: None,
            content_hash: Some(self.calculate_content_hash(&content)),
        });
        
        Ok(ResolvedReference {
            uri: uri.to_string(),
            content,
            resolved_at: chrono::Utc::now(),
            document_metadata: document.metadata.clone(),
        })
    }
    
    /// 批量验证引用完整性
    pub async fn validate_references(&self, references: &[String]) -> Vec<ReferenceValidation> {
        let futures: Vec<_> = references.iter()
            .map(|uri| self.validate_single_reference(uri))
            .collect();
        
        futures::future::join_all(futures).await
    }
}
```

## 7. 数据验证机制

### 7.1. 语法验证

```rust
// core/src/validation/syntax.rs
pub struct SyntaxValidator {
    uuid_validator: regex::Regex,
    block_type_whitelist: HashSet<String>,
}

impl SyntaxValidator {
    pub fn validate_document(&self, doc: &ElfiDocument) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // 验证文档级元数据
        if let Err(e) = self.validate_document_metadata(&doc.metadata) {
            errors.push(e);
        }
        
        // 验证每个区块
        let mut block_names = HashSet::new();
        for block in doc.blocks() {
            // 检查必需字段
            if let Err(e) = self.validate_required_fields(block) {
                errors.push(e);
            }
            
            // 检查名称唯一性
            if let Some(name) = &block.name {
                if !block_names.insert(name.clone()) {
                    errors.push(ValidationError::DuplicateBlockName(name.clone()));
                }
            }
            
            // 类型特定验证
            if let Err(e) = self.validate_block_content(block) {
                errors.push(e);
            }
        }
        
        ValidationResult { errors, warnings }
    }
    
    fn validate_block_content(&self, block: &Block) -> Result<(), ValidationError> {
        match block.block_type {
            BlockType::Code => {
                if block.metadata.language.is_none() {
                    return Err(ValidationError::MissingLanguage(block.id));
                }
            },
            BlockType::Link => {
                if let BlockContent::Link(link_content) = &block.content {
                    ElfUri::parse(&link_content.target)
                        .map_err(|_| ValidationError::InvalidUri(link_content.target.clone()))?;
                } else {
                    return Err(ValidationError::InvalidContentType {
                        block_id: block.id,
                        expected: "Link".to_string(),
                    });
                }
            },
            BlockType::Recipe => {
                // Recipe 配置的 YAML 验证
                if let BlockContent::Recipe(recipe) = &block.content {
                    self.validate_recipe_config(recipe)?;
                }
            },
            _ => {} // 其他类型的基础验证
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
    
    pub fn summary(&self) -> String {
        if self.is_valid() {
            if self.warnings.is_empty() {
                "✅ 文档验证通过".to_string()
            } else {
                format!("✅ 文档验证通过 ({} 个警告)", self.warnings.len())
            }
        } else {
            format!("❌ 发现 {} 个错误，{} 个警告", 
                self.errors.len(), self.warnings.len())
        }
    }
}
```

### 7.2. 引用完整性验证

```rust
// core/src/validation/integrity.rs
pub struct IntegrityValidator {
    link_resolver: Arc<LinkResolver>,
}

impl IntegrityValidator {
    /// 验证文档的引用完整性
    pub async fn validate_references(&self, doc: &ElfiDocument) -> IntegrityResult {
        let mut results = Vec::new();
        
        // 收集所有引用
        let references: Vec<_> = doc.blocks()
            .filter_map(|block| match &block.content {
                BlockContent::Link(link) => Some((block.id, &link.target)),
                BlockContent::Recipe(recipe) => {
                    Some((block.id, &recipe.references.iter()
                        .map(|r| r.source.as_str()).collect::<Vec<_>>()))
                },
                _ => None,
            })
            .collect();
        
        // 并行验证所有引用
        for (block_id, target) in references {
            match self.link_resolver.resolve_reference(target).await {
                Ok(_) => {
                    results.push(ReferenceCheck {
                        block_id,
                        target: target.to_string(),
                        status: ReferenceStatus::Valid,
                        message: None,
                    });
                },
                Err(e) => {
                    results.push(ReferenceCheck {
                        block_id,
                        target: target.to_string(),
                        status: ReferenceStatus::Invalid,
                        message: Some(e.to_string()),
                    });
                }
            }
        }
        
        IntegrityResult { checks: results }
    }
    
    /// 检测循环引用
    pub fn detect_circular_references(&self, doc: &ElfiDocument) -> Result<(), CircularReferenceError> {
        let mut graph = HashMap::new();
        
        // 构建引用图
        for block in doc.blocks() {
            if let Some(parent) = block.metadata.parent {
                graph.entry(parent)
                    .or_insert_with(Vec::new)
                    .push(block.id);
            }
        }
        
        // DFS 检测循环
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        for &start in graph.keys() {
            if !visited.contains(&start) {
                if let Some(cycle) = self.dfs_find_cycle(start, &graph, &mut visited, &mut rec_stack) {
                    return Err(CircularReferenceError::new(cycle));
                }
            }
        }
        
        Ok(())
    }
}
```

## 验证清单

### CRDT 数据模型
- [ ] Automerge 集成正确实现事件溯源
- [ ] 块级数据结构支持所有标准类型
- [ ] 层级结构通过邻接列表正确实现
- [ ] 并发操作的因果关系正确维护

### 冲突解决机制
- [ ] 块类型特定的冲突策略正确实现
- [ ] 冲突检测能覆盖所有并发场景
- [ ] 手动冲突解决的工作流完整
- [ ] 冲突解决不丢失任何编辑信息

### 存储同步机制
- [ ] Zenoh 网络集成支持实时同步
- [ ] 多级缓存提供良好的性能
- [ ] 跨文档引用解析稳定可靠
- [ ] 离线编辑和断网重连正常工作

### 数据验证完整性
- [ ] 语法验证覆盖所有必需字段和格式
- [ ] 引用完整性验证能检测损坏链接
- [ ] 循环引用检测算法正确
- [ ] 验证错误信息对用户友好

### 性能和扩展性
- [ ] 大型文档 (1000+ 区块) 的操作性能合理
- [ ] 网络同步延迟在可接受范围 (< 100ms)
- [ ] 内存使用随文档大小线性增长
- [ ] 支持自定义块类型和验证策略

这个数据模型设计确保了 ELFI 能够支持大规模、分布式的协作编辑，同时保持数据的完整性和一致性。
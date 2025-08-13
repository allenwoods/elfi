# 核心功能模块 - Main 接口与共享组件

本文档详述 ELFI 核心模块的实现，包括统一的 Main 类接口、会话管理、文档管理以及被 Weave 和 Tangle 层共同使用的核心功能。

## 1. Main 类统一接口设计

### 1.1. 设计理念

Main 类是 ELFI 系统的统一入口点，为所有上层封装（CLI、FFI、WASM）提供一致的接口：

- **接口统一性**：所有功能通过 Main 类暴露，确保多语言绑定的一致性
- **异步优先**：所有网络和 I/O 操作采用异步设计
- **错误透明**：提供详细的错误信息和用户友好的建议
- **状态管理**：管理文档生命周期和网络会话状态

### 1.2. Main 类核心结构

```rust
// core/src/main.rs
use std::sync::Arc;
use tokio::sync::RwLock;
use dashmap::DashMap;

pub struct Main {
    /// Zenoh 网络会话管理器
    session_manager: Arc<SessionManager>,
    /// 文档生命周期管理器
    document_manager: Arc<DocumentManager>,
    /// Recipe 执行引擎
    recipe_engine: Arc<RecipeEngine>,
    /// 多级缓存管理器
    cache_manager: Arc<CacheManager>,
    /// 活跃的文档句柄
    active_documents: Arc<DashMap<String, Arc<DocumentHandle>>>,
    /// 配置信息
    config: Arc<ElfiConfig>,
    /// 系统指标收集器
    metrics: Arc<SystemMetrics>,
}

impl Main {
    /// 创建新的 Main 实例
    pub async fn new(config: ElfiConfig) -> Result<Self, ElfiError> {
        // 初始化 Zenoh 网络会话
        let session_manager = Arc::new(
            SessionManager::new(&config.network).await?
        );
        
        // 初始化缓存管理器
        let cache_manager = Arc::new(
            CacheManager::new(&config.cache).await?
        );
        
        // 初始化文档管理器
        let document_manager = Arc::new(
            DocumentManager::new(session_manager.clone(), cache_manager.clone()).await?
        );
        
        // 初始化 Recipe 引擎
        let recipe_engine = Arc::new(
            RecipeEngine::new(session_manager.clone(), document_manager.clone()).await?
        );
        
        let metrics = Arc::new(SystemMetrics::new());
        
        Ok(Self {
            session_manager,
            document_manager,
            recipe_engine,
            cache_manager,
            active_documents: Arc::new(DashMap::new()),
            config: Arc::new(config),
            metrics,
        })
    }
}
```

### 1.3. 文档管理接口

```rust
impl Main {
    /// 打开或创建文档
    pub async fn open(&self, uri: &str) -> Result<DocumentHandle, ElfiError> {
        let start_time = std::time::Instant::now();
        
        // 检查是否已经打开
        if let Some(handle) = self.active_documents.get(uri) {
            return Ok(handle.clone());
        }
        
        // 解析 URI
        let parsed_uri = ElfUri::parse(uri)?;
        
        // 通过文档管理器打开文档
        let document = self.document_manager.open_document(&parsed_uri).await?;
        
        // 创建文档句柄
        let handle = Arc::new(DocumentHandle::new(
            document,
            self.session_manager.clone(),
            self.recipe_engine.clone(),
        ));
        
        // 注册到活跃文档列表
        self.active_documents.insert(uri.to_string(), handle.clone());
        
        // 记录指标
        self.metrics.record_operation(
            MetricOperation::DocumentOpen,
            start_time.elapsed(),
        );
        
        tracing::info!("文档已打开: {} (耗时: {:?})", uri, start_time.elapsed());
        
        Ok(handle)
    }
    
    /// 创建新文档
    pub async fn create(&self, config: CreateConfig) -> Result<DocumentHandle, ElfiError> {
        let document_id = uuid::Uuid::new_v4().to_string();
        let uri = format!("elf://{}/{}", config.repo, config.document_name);
        
        // 创建初始文档结构
        let mut initial_doc = ElfiDocument::new(DocumentMetadata {
            id: document_id,
            title: config.title,
            created_at: chrono::Utc::now(),
            last_modified: chrono::Utc::now(),
            authors: vec![config.author.unwrap_or_else(|| "anonymous".to_string())],
            version: "0.1.0".to_string(),
        });
        
        // 添加初始元数据块
        if config.add_metadata_block {
            let metadata_block = Block {
                id: uuid::Uuid::new_v4(),
                name: Some("document-metadata".to_string()),
                block_type: BlockType::Metadata,
                content: BlockContent::Structured(serde_json::json!({
                    "title": config.title,
                    "created": chrono::Utc::now(),
                    "purpose": config.purpose,
                })),
                metadata: BlockMetadata::default(),
            };
            initial_doc.add_block(metadata_block)?;
        }
        
        // 通过文档管理器持久化
        let document = self.document_manager.create_document(&uri, initial_doc).await?;
        
        // 创建句柄
        let handle = Arc::new(DocumentHandle::new(
            document,
            self.session_manager.clone(),
            self.recipe_engine.clone(),
        ));
        
        self.active_documents.insert(uri.clone(), handle.clone());
        
        tracing::info!("新文档已创建: {}", uri);
        
        Ok(handle)
    }
    
    /// 关闭文档
    pub async fn close(&self, uri: &str) -> Result<(), ElfiError> {
        if let Some((_, handle)) = self.active_documents.remove(uri) {
            // 等待所有未完成的操作
            handle.flush().await?;
            
            // 通知文档管理器
            self.document_manager.close_document(uri).await?;
            
            tracing::info!("文档已关闭: {}", uri);
        }
        
        Ok(())
    }
}
```

### 1.4. 内容操作接口

```rust
impl Main {
    /// 添加新区块
    pub async fn add_block(
        &self,
        doc_uri: &str,
        block_type: BlockType,
        name: Option<String>
    ) -> Result<String, ElfiError> {
        let handle = self.active_documents
            .get(doc_uri)
            .ok_or_else(|| ElfiError::DocumentNotOpen(doc_uri.to_string()))?;
        
        let block_id = uuid::Uuid::new_v4();
        let block = Block {
            id: block_id,
            name: name.clone(),
            block_type: block_type.clone(),
            content: Self::default_content_for_type(&block_type),
            metadata: BlockMetadata::default(),
        };
        
        handle.add_block(block).await?;
        
        tracing::info!("新区块已添加: {} (类型: {:?}, 名称: {:?})", 
            &block_id.to_string()[..8], block_type, name);
        
        Ok(block_id.to_string())
    }
    
    /// 删除区块
    pub async fn delete_block(
        &self,
        doc_uri: &str,
        block_id: &str
    ) -> Result<(), ElfiError> {
        let handle = self.active_documents
            .get(doc_uri)
            .ok_or_else(|| ElfiError::DocumentNotOpen(doc_uri.to_string()))?;
        
        let uuid = uuid::Uuid::parse_str(block_id)
            .map_err(|_| ElfiError::InvalidBlockId(block_id.to_string()))?;
        
        handle.delete_block(&uuid).await?;
        
        tracing::info!("区块已删除: {}", &block_id[..8]);
        
        Ok(())
    }
    
    /// 修改区块内容
    pub async fn update_block_content(
        &self,
        doc_uri: &str,
        block_id: &str,
        content: BlockContent
    ) -> Result<(), ElfiError> {
        let handle = self.active_documents
            .get(doc_uri)
            .ok_or_else(|| ElfiError::DocumentNotOpen(doc_uri.to_string()))?;
        
        let uuid = uuid::Uuid::parse_str(block_id)
            .map_err(|_| ElfiError::InvalidBlockId(block_id.to_string()))?;
        
        handle.update_content(&uuid, content).await?;
        
        Ok(())
    }
    
    /// 移动区块（修改 parent 关系）
    pub async fn move_block(
        &self,
        doc_uri: &str,
        block_id: &str,
        new_parent: Option<String>
    ) -> Result<(), ElfiError> {
        let handle = self.active_documents
            .get(doc_uri)
            .ok_or_else(|| ElfiError::DocumentNotOpen(doc_uri.to_string()))?;
        
        let uuid = uuid::Uuid::parse_str(block_id)
            .map_err(|_| ElfiError::InvalidBlockId(block_id.to_string()))?;
        
        let parent_uuid = if let Some(parent_id) = new_parent {
            Some(uuid::Uuid::parse_str(&parent_id)
                .map_err(|_| ElfiError::InvalidBlockId(parent_id))?)
        } else {
            None
        };
        
        handle.move_block(&uuid, parent_uuid).await?;
        
        tracing::info!("区块已移动: {} -> {:?}", 
            &block_id[..8], parent_uuid.map(|id| &id.to_string()[..8]));
        
        Ok(())
    }
}
```

### 1.5. 协作功能接口

```rust
impl Main {
    /// 同步文档变更
    pub async fn sync(&self, doc_uri: &str) -> Result<SyncResult, ElfiError> {
        let handle = self.active_documents
            .get(doc_uri)
            .ok_or_else(|| ElfiError::DocumentNotOpen(doc_uri.to_string()))?;
        
        let start_time = std::time::Instant::now();
        
        // 执行同步操作
        let result = handle.sync().await?;
        
        // 记录指标
        self.metrics.record_sync_operation(&result, start_time.elapsed());
        
        tracing::info!("文档同步完成: {} (CRDT: {}, 冲突: {}, 耗时: {:?})",
            doc_uri, result.crdt_merges, result.manual_conflicts, start_time.elapsed());
        
        Ok(result)
    }
    
    /// 转移区块所有权
    pub async fn transfer_ownership(
        &self,
        doc_uri: &str,
        block_id: &str,
        to_user: &str
    ) -> Result<(), ElfiError> {
        let handle = self.active_documents
            .get(doc_uri)
            .ok_or_else(|| ElfiError::DocumentNotOpen(doc_uri.to_string()))?;
        
        let uuid = uuid::Uuid::parse_str(block_id)
            .map_err(|_| ElfiError::InvalidBlockId(block_id.to_string()))?;
        
        handle.transfer_ownership(&uuid, to_user).await?;
        
        tracing::info!("区块所有权已转移: {} -> {}", &block_id[..8], to_user);
        
        Ok(())
    }
    
    /// 声明区块所有权
    pub async fn claim_ownership(
        &self,
        doc_uri: &str,
        block_id: &str
    ) -> Result<(), ElfiError> {
        let handle = self.active_documents
            .get(doc_uri)
            .ok_or_else(|| ElfiError::DocumentNotOpen(doc_uri.to_string()))?;
        
        let uuid = uuid::Uuid::parse_str(block_id)
            .map_err(|_| ElfiError::InvalidBlockId(block_id.to_string()))?;
        
        let current_user = self.session_manager.get_current_user().await?;
        handle.claim_ownership(&uuid, &current_user).await?;
        
        tracing::info!("区块所有权已声明: {} -> {}", &block_id[..8], current_user);
        
        Ok(())
    }
}
```

### 1.6. Recipe 和导出接口

```rust
impl Main {
    /// 列出可用的 Recipe
    pub async fn list_recipes(&self, doc_uri: &str) -> Result<Vec<RecipeInfo>, ElfiError> {
        let handle = self.active_documents
            .get(doc_uri)
            .ok_or_else(|| ElfiError::DocumentNotOpen(doc_uri.to_string()))?;
        
        let recipes = handle.list_recipes().await?;
        
        Ok(recipes)
    }
    
    /// 执行 Recipe 导出
    pub async fn export(
        &self,
        doc_uri: &str,
        recipe_name: &str,
        output_path: &str
    ) -> Result<ExportResult, ElfiError> {
        let handle = self.active_documents
            .get(doc_uri)
            .ok_or_else(|| ElfiError::DocumentNotOpen(doc_uri.to_string()))?;
        
        let start_time = std::time::Instant::now();
        
        // 执行 Recipe
        let result = self.recipe_engine
            .execute_recipe(doc_uri, recipe_name, output_path)
            .await?;
        
        // 记录指标
        self.metrics.record_recipe_execution(
            recipe_name,
            start_time.elapsed(),
            result.blocks_processed,
            result.references_resolved,
        );
        
        tracing::info!("Recipe 执行完成: {} (处理 {} 个区块, {} 个引用, 耗时 {:?})",
            recipe_name, result.blocks_processed, result.references_resolved, start_time.elapsed());
        
        Ok(result)
    }
    
    /// 验证 Recipe 配置
    pub async fn validate_recipe(
        &self,
        doc_uri: &str,
        recipe_name: &str
    ) -> Result<RecipeValidation, ElfiError> {
        let handle = self.active_documents
            .get(doc_uri)
            .ok_or_else(|| ElfiError::DocumentNotOpen(doc_uri.to_string()))?;
        
        let validation = handle.validate_recipe(recipe_name).await?;
        
        Ok(validation)
    }
}
```

## 2. SessionManager - 网络会话管理

### 2.1. Zenoh 会话抽象

```rust
// core/src/session/manager.rs
pub struct SessionManager {
    zenoh_session: Arc<zenoh::Session>,
    network_config: NetworkConfig,
    peer_discovery: Arc<PeerDiscovery>,
    connection_pool: Arc<ConnectionPool>,
    encryption: Arc<EncryptionManager>,
}

impl SessionManager {
    pub async fn new(config: &NetworkConfig) -> Result<Self, SessionError> {
        // 配置 Zenoh 会话
        let mut zenoh_config = zenoh::config::Config::default();
        
        // 设置监听地址
        if let Some(listen) = &config.listen_addresses {
            zenoh_config.insert_json5("listen/endpoints", &serde_json::to_string(listen)?)?;
        }
        
        // 设置连接目标
        if let Some(connect) = &config.connect_endpoints {
            zenoh_config.insert_json5("connect/endpoints", &serde_json::to_string(connect)?)?;
        }
        
        // 开启 Zenoh 会话
        let session = Arc::new(zenoh::open(zenoh_config).await?);
        
        // 初始化对等发现
        let peer_discovery = Arc::new(PeerDiscovery::new(session.clone(), config));
        
        Ok(Self {
            zenoh_session: session,
            network_config: config.clone(),
            peer_discovery,
            connection_pool: Arc::new(ConnectionPool::new()),
            encryption: Arc::new(EncryptionManager::new(&config.encryption)?),
        })
    }
    
    /// 获取当前用户信息
    pub async fn get_current_user(&self) -> Result<String, SessionError> {
        // 从配置或认证令牌中获取用户信息
        Ok(self.network_config.user_id.clone()
            .unwrap_or_else(|| "anonymous".to_string()))
    }
    
    /// 发布操作到网络
    pub async fn publish_operation(
        &self,
        doc_uri: &str,
        operation: &CrdtOperation
    ) -> Result<(), SessionError> {
        let key = format!("/elf/docs/{}/ops", doc_uri);
        
        // 加密操作数据
        let encrypted_data = self.encryption.encrypt(operation).await?;
        
        // 发布到 Zenoh 网络
        self.zenoh_session
            .put(&key, encrypted_data)
            .await?;
        
        Ok(())
    }
    
    /// 订阅文档操作
    pub async fn subscribe_operations<F>(
        &self,
        doc_uri: &str,
        callback: F
    ) -> Result<zenoh::Subscriber<'static, ()>, SessionError>
    where
        F: Fn(CrdtOperation) + Send + Sync + 'static,
    {
        let key = format!("/elf/docs/{}/ops", doc_uri);
        let encryption = self.encryption.clone();
        let callback = Arc::new(callback);
        
        let subscriber = self.zenoh_session
            .declare_subscriber(&key)
            .callback(move |sample| {
                let encryption = encryption.clone();
                let callback = callback.clone();
                
                tokio::spawn(async move {
                    if let Ok(operation) = encryption.decrypt::<CrdtOperation>(sample.payload()).await {
                        callback(operation);
                    }
                });
            })
            .await?;
        
        Ok(subscriber)
    }
    
    /// 查询历史操作
    pub async fn query_history(
        &self,
        doc_uri: &str,
        since: Option<chrono::DateTime<chrono::Utc>>
    ) -> Result<Vec<CrdtOperation>, SessionError> {
        let key = format!("/elf/docs/{}/ops", doc_uri);
        let selector = if let Some(timestamp) = since {
            format!("{}?_time=[{},*]", key, timestamp.timestamp())
        } else {
            key
        };
        
        let mut operations = Vec::new();
        let replies = self.zenoh_session.get(&selector).await?;
        
        while let Ok(reply) = replies.recv_async().await {
            if let Ok(operation) = self.encryption.decrypt::<CrdtOperation>(reply.payload()).await {
                operations.push(operation);
            }
        }
        
        // 按时间戳排序
        operations.sort_by_key(|op| op.timestamp);
        Ok(operations)
    }
}
```

### 2.2. 对等节点发现

```rust
// core/src/session/peer_discovery.rs
pub struct PeerDiscovery {
    zenoh_session: Arc<zenoh::Session>,
    discovered_peers: Arc<DashMap<String, PeerInfo>>,
    mdns_service: Option<mdns::Service>,
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub user_id: String,
    pub device_name: String,
    pub capabilities: Vec<String>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub network_address: String,
}

impl PeerDiscovery {
    pub fn new(session: Arc<zenoh::Session>, config: &NetworkConfig) -> Self {
        let mdns_service = if config.enable_mdns_discovery {
            Some(Self::setup_mdns_discovery(config))
        } else {
            None
        };
        
        Self {
            zenoh_session: session,
            discovered_peers: Arc::new(DashMap::new()),
            mdns_service,
        }
    }
    
    /// 启动对等发现服务
    pub async fn start_discovery(&self) -> Result<(), DiscoveryError> {
        // 发布自身信息
        let self_info = PeerInfo {
            user_id: "current_user".to_string(), // 从配置获取
            device_name: hostname::get()?.to_string_lossy().to_string(),
            capabilities: vec!["elfi-core".to_string()],
            last_seen: chrono::Utc::now(),
            network_address: "".to_string(), // Zenoh 会自动处理
        };
        
        let key = format!("/elf/peers/{}", self_info.user_id);
        self.zenoh_session
            .put(&key, serde_json::to_vec(&self_info)?)
            .await?;
        
        // 订阅其他节点信息
        let discovered_peers = self.discovered_peers.clone();
        self.zenoh_session
            .declare_subscriber("/elf/peers/*")
            .callback(move |sample| {
                if let Ok(peer_info) = serde_json::from_slice::<PeerInfo>(sample.payload()) {
                    discovered_peers.insert(peer_info.user_id.clone(), peer_info);
                }
            })
            .await?;
        
        Ok(())
    }
    
    /// 获取发现的对等节点列表
    pub fn get_peers(&self) -> Vec<PeerInfo> {
        self.discovered_peers
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}
```

## 3. DocumentManager - 文档生命周期管理

### 3.1. 文档管理核心

```rust
// core/src/document/manager.rs
pub struct DocumentManager {
    session_manager: Arc<SessionManager>,
    cache_manager: Arc<CacheManager>,
    active_documents: Arc<DashMap<String, Arc<RwLock<ElfiDocument>>>>,
    parser: Arc<ElfParser>,
    conflict_resolver: Arc<ConflictResolver>,
}

impl DocumentManager {
    pub async fn new(
        session_manager: Arc<SessionManager>,
        cache_manager: Arc<CacheManager>
    ) -> Result<Self, DocumentError> {
        Ok(Self {
            session_manager,
            cache_manager,
            active_documents: Arc::new(DashMap::new()),
            parser: Arc::new(ElfParser::new()?),
            conflict_resolver: Arc::new(ConflictResolver::new()),
        })
    }
    
    /// 打开文档
    pub async fn open_document(&self, uri: &ElfUri) -> Result<Arc<RwLock<ElfiDocument>>, DocumentError> {
        let uri_str = uri.to_string();
        
        // 检查是否已加载
        if let Some(doc) = self.active_documents.get(&uri_str) {
            return Ok(doc.clone());
        }
        
        // 从缓存或网络加载
        let document = match self.cache_manager.get_document(&uri_str).await {
            Ok(cached_doc) => cached_doc,
            Err(_) => {
                // 从网络重建文档
                let operations = self.session_manager
                    .query_history(&uri_str, None)
                    .await?;
                self.rebuild_document_from_operations(operations)?
            }
        };
        
        let doc_ref = Arc::new(RwLock::new(document));
        
        // 启动实时同步
        self.start_realtime_sync(&uri_str, doc_ref.clone()).await?;
        
        // 加入活跃文档列表
        self.active_documents.insert(uri_str, doc_ref.clone());
        
        Ok(doc_ref)
    }
    
    /// 创建新文档
    pub async fn create_document(
        &self,
        uri: &str,
        initial_doc: ElfiDocument
    ) -> Result<Arc<RwLock<ElfiDocument>>, DocumentError> {
        // 发布初始状态到网络
        let create_operation = CrdtOperation::CreateDocument {
            document_id: initial_doc.metadata.id.clone(),
            metadata: initial_doc.metadata.clone(),
            timestamp: chrono::Utc::now(),
        };
        
        self.session_manager
            .publish_operation(uri, &create_operation)
            .await?;
        
        // 发布初始区块
        for block in initial_doc.blocks() {
            let add_block_op = CrdtOperation::AddBlock {
                block: block.clone(),
                timestamp: chrono::Utc::now(),
            };
            
            self.session_manager
                .publish_operation(uri, &add_block_op)
                .await?;
        }
        
        let doc_ref = Arc::new(RwLock::new(initial_doc));
        
        // 启动实时同步
        self.start_realtime_sync(uri, doc_ref.clone()).await?;
        
        // 存储到缓存
        self.cache_manager.store_document(uri, &doc_ref).await?;
        
        // 加入活跃文档列表
        self.active_documents.insert(uri.to_string(), doc_ref.clone());
        
        Ok(doc_ref)
    }
    
    /// 启动实时同步
    async fn start_realtime_sync(
        &self,
        uri: &str,
        document: Arc<RwLock<ElfiDocument>>
    ) -> Result<(), DocumentError> {
        let conflict_resolver = self.conflict_resolver.clone();
        let cache_manager = self.cache_manager.clone();
        let uri_str = uri.to_string();
        
        self.session_manager.subscribe_operations(uri, move |operation| {
            let document = document.clone();
            let conflict_resolver = conflict_resolver.clone();
            let cache_manager = cache_manager.clone();
            let uri = uri_str.clone();
            
            tokio::spawn(async move {
                // 应用操作到文档
                let result = {
                    let mut doc = document.write().await;
                    doc.apply_operation(&operation)
                };
                
                match result {
                    Ok(conflicts) => {
                        // 处理冲突
                        if !conflicts.is_empty() {
                            for conflict in conflicts {
                                if let Err(e) = conflict_resolver.resolve_conflict(&conflict).await {
                                    tracing::warn!("冲突解决失败: {:?}", e);
                                }
                            }
                        }
                        
                        // 更新缓存
                        if let Err(e) = cache_manager.update_document(&uri, &document).await {
                            tracing::warn!("缓存更新失败: {:?}", e);
                        }
                    },
                    Err(e) => {
                        tracing::error!("应用操作失败: {:?}", e);
                    }
                }
            });
        }).await?;
        
        Ok(())
    }
}
```

## 4. 解析器集成

### 4.1. .elf 文件解析

```rust
// core/src/parser/mod.rs
pub struct ElfParser {
    tree_sitter_parser: tree_sitter::Parser,
}

impl ElfParser {
    pub fn new() -> Result<Self, ParseError> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_elf::language())?;
        
        Ok(Self {
            tree_sitter_parser: parser,
        })
    }
    
    /// 解析 .elf 文本为文档结构
    pub fn parse_text(&mut self, content: &str) -> Result<ElfiDocument, ParseError> {
        // Tree-sitter 解析
        let tree = self.tree_sitter_parser
            .parse(content, None)
            .ok_or(ParseError::TreeSitterFailed)?;
        
        let root_node = tree.root_node();
        let mut document = ElfiDocument::new(DocumentMetadata::default());
        
        // 遍历所有块节点
        for block_node in root_node.children(&mut tree.walk()) {
            if block_node.kind() == "block" {
                let block = self.parse_block_node(content, &block_node)?;
                document.add_block(block)?;
            }
        }
        
        Ok(document)
    }
    
    /// 解析单个块节点
    fn parse_block_node(
        &self,
        source: &str,
        node: &tree_sitter::Node
    ) -> Result<Block, ParseError> {
        let mut metadata_text = None;
        let mut content_text = None;
        
        // 提取元数据和内容部分
        for child in node.children(&mut node.walk()) {
            match child.kind() {
                "metadata_section" => {
                    metadata_text = Some(child.utf8_text(source.as_bytes())?);
                }
                "content_section" => {
                    content_text = Some(child.utf8_text(source.as_bytes())?);
                }
                _ => {}
            }
        }
        
        // 解析 YAML 元数据
        let metadata_str = metadata_text
            .ok_or(ParseError::MissingMetadata)?
            .trim_start_matches("---")
            .trim_end_matches("---")
            .trim();
        
        let parsed_metadata: BlockMetadataRaw = serde_yaml::from_str(metadata_str)?;
        
        // 验证必需字段
        parsed_metadata.validate()?;
        
        // 解析内容
        let content = match parsed_metadata.block_type.as_str() {
            "link" => {
                let link_content: LinkContent = serde_yaml::from_str(
                    content_text.unwrap_or("")
                )?;
                BlockContent::Link(link_content)
            }
            "recipe" => {
                let recipe_config: RecipeConfig = serde_yaml::from_str(
                    content_text.unwrap_or("")
                )?;
                BlockContent::Recipe(recipe_config)
            }
            _ => {
                BlockContent::Text(content_text.unwrap_or("").to_string())
            }
        };
        
        Ok(Block {
            id: uuid::Uuid::parse_str(&parsed_metadata.id)?,
            name: parsed_metadata.name,
            block_type: BlockType::from_str(&parsed_metadata.block_type)?,
            content,
            metadata: parsed_metadata.metadata.unwrap_or_default(),
        })
    }
}

#[derive(Deserialize)]
struct BlockMetadataRaw {
    id: String,
    #[serde(rename = "type")]
    block_type: String,
    name: Option<String>,
    metadata: Option<BlockMetadata>,
}

impl BlockMetadataRaw {
    fn validate(&self) -> Result<(), ParseError> {
        // 验证 UUID 格式
        uuid::Uuid::parse_str(&self.id)
            .map_err(|_| ParseError::InvalidUuid(self.id.clone()))?;
        
        // 验证块类型
        match self.block_type.as_str() {
            "markdown" | "code" | "link" | "recipe" | "metadata" => Ok(()),
            _ => Err(ParseError::UnknownBlockType(self.block_type.clone())),
        }
    }
}
```

## 5. 系统指标收集

### 5.1. 性能指标

```rust
// core/src/metrics.rs
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    // 操作计数器
    document_operations: Arc<AtomicU64>,
    recipe_executions: Arc<AtomicU64>,
    network_requests: Arc<AtomicU64>,
    
    // 性能指标（移动平均）
    avg_sync_latency: Arc<RwLock<f64>>,
    avg_recipe_execution_time: Arc<RwLock<f64>>,
    
    // 资源使用
    memory_usage: Arc<AtomicU64>,
    active_documents_count: Arc<AtomicUsize>,
    
    // 错误统计
    total_errors: Arc<AtomicU64>,
    network_errors: Arc<AtomicU64>,
}

impl SystemMetrics {
    pub fn new() -> Self {
        Self {
            document_operations: Arc::new(AtomicU64::new(0)),
            recipe_executions: Arc::new(AtomicU64::new(0)),
            network_requests: Arc::new(AtomicU64::new(0)),
            avg_sync_latency: Arc::new(RwLock::new(0.0)),
            avg_recipe_execution_time: Arc::new(RwLock::new(0.0)),
            memory_usage: Arc::new(AtomicU64::new(0)),
            active_documents_count: Arc::new(AtomicUsize::new(0)),
            total_errors: Arc::new(AtomicU64::new(0)),
            network_errors: Arc::new(AtomicU64::new(0)),
        }
    }
    
    pub fn record_operation(&self, operation: MetricOperation, duration: Duration) {
        match operation {
            MetricOperation::DocumentOpen => {
                self.document_operations.fetch_add(1, Ordering::Relaxed);
            }
            MetricOperation::DocumentSync => {
                self.document_operations.fetch_add(1, Ordering::Relaxed);
                self.update_avg_latency(&self.avg_sync_latency, duration);
            }
            MetricOperation::RecipeExecution => {
                self.recipe_executions.fetch_add(1, Ordering::Relaxed);
                self.update_avg_latency(&self.avg_recipe_execution_time, duration);
            }
            MetricOperation::NetworkRequest => {
                self.network_requests.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    
    fn update_avg_latency(&self, avg_ref: &RwLock<f64>, new_duration: Duration) {
        let new_ms = new_duration.as_millis() as f64;
        if let Ok(mut avg) = avg_ref.write() {
            // 指数移动平均 (α = 0.1)
            *avg = *avg * 0.9 + new_ms * 0.1;
        }
    }
    
    pub async fn get_summary(&self) -> MetricsSummary {
        MetricsSummary {
            document_operations_total: self.document_operations.load(Ordering::Relaxed),
            recipe_executions_total: self.recipe_executions.load(Ordering::Relaxed),
            network_requests_total: self.network_requests.load(Ordering::Relaxed),
            avg_sync_latency_ms: *self.avg_sync_latency.read().await,
            avg_recipe_execution_ms: *self.avg_recipe_execution_time.read().await,
            memory_usage_mb: self.memory_usage.load(Ordering::Relaxed) as f64 / 1024.0 / 1024.0,
            active_documents: self.active_documents_count.load(Ordering::Relaxed),
            total_errors: self.total_errors.load(Ordering::Relaxed),
            network_errors: self.network_errors.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MetricOperation {
    DocumentOpen,
    DocumentSync,
    RecipeExecution,
    NetworkRequest,
}
```

## 验证清单

### Main 接口完整性
- [ ] 所有 CLI 命令都有对应的 Main 方法
- [ ] 异步接口设计支持高并发场景
- [ ] 错误处理提供详细信息和建议
- [ ] 多语言绑定的兼容性验证

### 会话管理稳定性
- [ ] Zenoh 网络会话的连接管理正确
- [ ] 对等节点发现和连接建立正常
- [ ] 网络分区和重连恢复机制有效
- [ ] 加密和安全机制正确实现

### 文档管理正确性
- [ ] 文档生命周期管理完整
- [ ] 实时同步和冲突解决正确
- [ ] 缓存策略和性能优化有效
- [ ] 内存使用和资源释放合理

### 解析器准确性
- [ ] .elf 语法解析完全正确
- [ ] 错误处理和用户提示友好
- [ ] Tree-sitter 集成性能良好
- [ ] 各种块类型的解析支持完整

### 系统指标可观察性
- [ ] 关键性能指标收集完整
- [ ] 错误统计和分析有帮助
- [ ] 指标数据的准确性和实时性
- [ ] 监控和告警机制有效

这个核心模块设计确保了 ELFI 系统的稳定性、性能和可扩展性，为上层的 Weave 和 Tangle 模块提供了坚实的基础。
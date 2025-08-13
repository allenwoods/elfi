# 核心逻辑重新设计 (`elfi-core`)

基于用例场景和架构重新设计，本文档详细说明了 `elfi-core` 库的数据结构、API 设计和核心工作流程。

## 1. 核心数据结构

### 1.1. 主要接口定义

```rust
// src/lib.rs - 公共 API
pub mod session;     // SessionManager - 会话和网络管理
pub mod document;    // DocumentManager - 文档生命周期管理  
pub mod recipe;      // RecipeEngine - 内容转换引擎
pub mod link;        // LinkResolver - 跨文档引用解析
pub mod error;       // 统一错误处理
pub mod types;       // 核心数据类型

// 重新导出主要接口，简化使用
pub use session::SessionManager;
pub use document::{DocumentManager, DocumentHandle};
pub use recipe::RecipeEngine;
pub use error::ElfiError;
```

### 1.2. Main 类接口 (对应 CLI 命令)

```rust
// src/main.rs - 对应用户提到的 Main.open 等函数
pub struct Main {
    session: SessionManager,
    documents: DocumentManager,
    recipes: RecipeEngine,
}

impl Main {
    /// 创建新的 elfi 实例
    pub async fn new(config: ElfiConfig) -> Result<Self, ElfiError> {
        let session = SessionManager::new(config.zenoh_config).await?;
        let documents = DocumentManager::new(&session);
        let recipes = RecipeEngine::new(&session, &documents);
        
        Ok(Self {
            session,
            documents,
            recipes,
        })
    }

    /// 对应 `elfi open` 命令
    pub async fn open(&self, uri: &str) -> Result<DocumentHandle, ElfiError> {
        self.documents.open_document(uri).await
    }
    
    /// 对应 `elfi add` 命令
    pub async fn add_block(&self, doc_uri: &str, block_type: BlockType, name: Option<String>) 
        -> Result<String, ElfiError> {
        let handle = self.documents.get_handle(doc_uri).await?;
        handle.add_block(block_type, name).await
    }
    
    /// 对应 `elfi export` 命令
    pub async fn export(&self, doc_uri: &str, recipe_name: &str, output_path: &str) 
        -> Result<ExportResult, ElfiError> {
        self.recipes.execute_recipe(doc_uri, recipe_name, output_path).await
    }
    
    /// 对应 `elfi sync` 命令
    pub async fn sync(&self, doc_uri: &str) -> Result<SyncResult, ElfiError> {
        let handle = self.documents.get_handle(doc_uri).await?;
        handle.sync().await
    }
    
    /// 对应 `elfi watch` 命令
    pub async fn watch(&self, config: WatchConfig) -> Result<WatchHandle, ElfiError> {
        self.documents.start_watch_mode(config).await
    }
    
    /// 对应 `elfi transfer` 命令
    pub async fn transfer_ownership(&self, doc_uri: &str, block_id: &str, to_user: &str) 
        -> Result<(), ElfiError> {
        let handle = self.documents.get_handle(doc_uri).await?;
        handle.transfer_ownership(block_id, to_user).await
    }
    
    /// 对应 `elfi claim` 命令
    pub async fn claim_ownership(&self, doc_uri: &str, block_id: &str) 
        -> Result<(), ElfiError> {
        let handle = self.documents.get_handle(doc_uri).await?;
        handle.claim_ownership(block_id).await
    }
    
    /// 关闭并清理资源
    pub async fn shutdown(&self) -> Result<(), ElfiError> {
        self.documents.shutdown().await?;
        self.session.shutdown().await?;
        Ok(())
    }
}
```

## 2. SessionManager - 网络会话管理

```rust
// src/session.rs
use zenoh::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SessionManager {
    zenoh_session: Arc<zenoh::Session>,
    config: ZenohConfig,
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    metrics: Arc<Metrics>,
}

impl SessionManager {
    pub async fn new(config: ZenohConfig) -> Result<Self, ElfiError> {
        let session = zenoh::open(config.clone()).await
            .map_err(|e| ElfiError::Network(e))?;
            
        Ok(Self {
            zenoh_session: Arc::new(session),
            config,
            peers: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(Metrics::new()),
        })
    }
    
    /// 发布操作到网络
    pub async fn publish_operation(&self, doc_id: &str, operation: &[u8]) 
        -> Result<(), ElfiError> {
        let key = format!("/elfi/docs/{}/ops", doc_id);
        self.zenoh_session
            .put(&key, operation)
            .await
            .map_err(|e| ElfiError::Network(e))?;
        Ok(())
    }
    
    /// 订阅文档操作流
    pub async fn subscribe_operations(&self, doc_id: &str) 
        -> Result<flume::Receiver<Operation>, ElfiError> {
        let key = format!("/elfi/docs/{}/ops", doc_id);
        let (tx, rx) = flume::unbounded();
        
        let subscriber = self.zenoh_session
            .declare_subscriber(&key)
            .await
            .map_err(|e| ElfiError::Network(e))?;
            
        // 启动后台任务处理消息
        let tx = tx.clone();
        tokio::spawn(async move {
            while let Ok(sample) = subscriber.recv_async().await {
                if let Ok(operation) = Operation::decode(&sample.payload) {
                    let _ = tx.send_async(operation).await;
                }
            }
        });
        
        Ok(rx)
    }
    
    /// 网络发现和对等连接
    pub async fn discover_peers(&self) -> Result<Vec<PeerInfo>, ElfiError> {
        // 使用 Zenoh 的发现机制查找其他 elfi 实例
        let peers = self.zenoh_session
            .info()
            .peers_zid()
            .await;
            
        // 转换为 PeerInfo 结构
        let peer_infos = peers.iter()
            .map(|zid| PeerInfo {
                id: zid.to_string(),
                endpoint: format!("zenoh://{}", zid),
                last_seen: SystemTime::now(),
            })
            .collect();
            
        Ok(peer_infos)
    }
}
```

## 3. DocumentManager - 文档生命周期管理

```rust
// src/document.rs
use automerge::AutoCommit;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

pub struct DocumentManager {
    session: Arc<SessionManager>,
    documents: DashMap<String, Arc<DocumentHandle>>,
    parser: Arc<elfi_parser::Parser>,
}

pub struct DocumentHandle {
    doc_id: String,
    doc_uri: String,
    document: Arc<RwLock<AutoCommit>>,
    change_notifier: broadcast::Sender<ChangeEvent>,
    operation_receiver: flume::Receiver<Operation>,
    sync_state: Arc<Mutex<SyncState>>,
    metrics: Arc<DocumentMetrics>,
}

impl DocumentManager {
    pub fn new(session: &SessionManager) -> Self {
        Self {
            session: Arc::new(session.clone()),
            documents: DashMap::new(),
            parser: Arc::new(elfi_parser::Parser::new()),
        }
    }
    
    /// 打开或创建文档
    pub async fn open_document(&self, uri: &str) -> Result<DocumentHandle, ElfiError> {
        if let Some(existing) = self.documents.get(uri) {
            return Ok(existing.clone());
        }
        
        let handle = self.create_document_handle(uri).await?;
        self.documents.insert(uri.to_string(), Arc::new(handle.clone()));
        
        Ok(handle)
    }
    
    async fn create_document_handle(&self, uri: &str) -> Result<DocumentHandle, ElfiError> {
        // 解析 URI 获取文档 ID
        let doc_id = self.parse_document_uri(uri)?;
        
        // 尝试从网络或本地加载现有文档
        let document = match self.load_existing_document(&doc_id).await {
            Ok(doc) => doc,
            Err(_) => {
                // 如果不存在，创建新文档
                self.create_new_document(&doc_id).await?
            }
        };
        
        // 设置网络订阅
        let operation_receiver = self.session
            .subscribe_operations(&doc_id)
            .await?;
        
        let (change_notifier, _) = broadcast::channel(1024);
        
        let handle = DocumentHandle {
            doc_id: doc_id.clone(),
            doc_uri: uri.to_string(),
            document: Arc::new(RwLock::new(document)),
            change_notifier,
            operation_receiver,
            sync_state: Arc::new(Mutex::new(SyncState::Connected)),
            metrics: Arc::new(DocumentMetrics::new()),
        };
        
        // 启动同步任务
        self.start_sync_task(handle.clone()).await;
        
        Ok(handle)
    }
    
    /// 启动文件监视模式 (对应 elfi watch)
    pub async fn start_watch_mode(&self, config: WatchConfig) -> Result<WatchHandle, ElfiError> {
        use notify::{Watcher, RecursiveMode, Event};
        
        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx)
            .map_err(|e| ElfiError::Io(e.into()))?;
            
        watcher.watch(&config.watch_path, RecursiveMode::Recursive)
            .map_err(|e| ElfiError::Io(e.into()))?;
        
        let handle = WatchHandle {
            watcher,
            receiver: rx,
            config: config.clone(),
        };
        
        // 启动文件变更处理任务
        let documents = self.clone();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv() {
                if let Err(e) = documents.handle_file_change(event).await {
                    eprintln!("文件变更处理错误: {}", e);
                }
            }
        });
        
        Ok(handle)
    }
    
    async fn handle_file_change(&self, event: notify::Event) -> Result<(), ElfiError> {
        match event.kind {
            notify::EventKind::Modify(_) => {
                for path in event.paths {
                    if let Some(doc_uri) = self.map_file_to_document(&path) {
                        self.sync_file_to_document(&path, &doc_uri).await?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl DocumentHandle {
    /// 添加新区块
    pub async fn add_block(&self, block_type: BlockType, name: Option<String>) 
        -> Result<String, ElfiError> {
        let block_id = uuid::Uuid::new_v4().to_string();
        let block_name = name.unwrap_or_else(|| format!("block-{}", &block_id[..8]));
        
        let mut doc = self.document.write().await;
        
        // 使用 automerge 创建新区块
        doc.change(|doc| {
            let blocks = doc.get_or_create_map("blocks")?;
            let block = blocks.get_or_create_map(&block_id)?;
            
            block.put("id", &block_id)?;
            block.put("name", &block_name)?;
            block.put("type", block_type.as_str())?;
            block.put("content", "")?;
            
            // 初始化元数据
            let metadata = block.get_or_create_map("metadata")?;
            metadata.put("created", chrono::Utc::now().timestamp())?;
            metadata.put("author", std::env::var("USER").unwrap_or_default())?;
            
            Ok(())
        })?;
        
        // 通知变更
        let _ = self.change_notifier.send(ChangeEvent::BlockAdded {
            block_id: block_id.clone(),
        });
        
        // 发布到网络
        self.publish_local_changes().await?;
        
        Ok(block_id)
    }
    
    /// 修改区块内容
    pub async fn change<F>(&self, mut callback: F) -> Result<(), ElfiError>
    where
        F: FnMut(&mut AutoCommit) -> Result<(), automerge::AutomergeError>,
    {
        let mut doc = self.document.write().await;
        
        // 记录修改前的状态
        let before_heads = doc.get_heads();
        
        // 执行用户修改
        callback(&mut doc)?;
        
        // 检查是否有变更
        let after_heads = doc.get_heads();
        if before_heads != after_heads {
            // 通知本地订阅者
            let _ = self.change_notifier.send(ChangeEvent::DocumentChanged);
            
            // 发布变更到网络
            self.publish_local_changes().await?;
        }
        
        Ok(())
    }
    
    /// 订阅文档变更
    pub fn subscribe(&self) -> broadcast::Receiver<ChangeEvent> {
        self.change_notifier.subscribe()
    }
    
    /// 获取文档历史
    pub async fn get_history(&self) -> Result<Vec<HistoryEntry>, ElfiError> {
        let doc = self.document.read().await;
        let changes = doc.get_changes(&[]).map_err(ElfiError::DataModel)?;
        
        Ok(changes.into_iter()
            .map(|change| HistoryEntry {
                hash: format!("{:x}", change.hash()),
                timestamp: change.timestamp(),
                actor: change.actor_id().to_string(),
                message: change.message().unwrap_or_default().to_string(),
            })
            .collect())
    }
    
    /// 时间旅行：获取特定时间点的文档状态
    pub async fn view_at(&self, heads: &[ChangeHash]) -> Result<DocumentSnapshot, ElfiError> {
        let doc = self.document.read().await;
        let doc_at_heads = doc.fork_at(heads).map_err(ElfiError::DataModel)?;
        
        Ok(DocumentSnapshot {
            heads: heads.to_vec(),
            blocks: self.extract_blocks(&doc_at_heads).await?,
        })
    }
    
    /// 转移区块所有权
    pub async fn transfer_ownership(&self, block_id: &str, to_user: &str) 
        -> Result<(), ElfiError> {
        self.change(|doc| {
            let blocks = doc.get_or_create_map("blocks")?;
            let block = blocks.get_map(block_id)
                .ok_or_else(|| automerge::AutomergeError::InvalidPath)?;
            let metadata = block.get_or_create_map("metadata")?;
            
            metadata.put("owner", to_user)?;
            metadata.put("ownership_transferred_at", chrono::Utc::now().timestamp())?;
            
            Ok(())
        }).await
    }
    
    /// 声明区块所有权
    pub async fn claim_ownership(&self, block_id: &str) -> Result<(), ElfiError> {
        let current_user = std::env::var("USER").unwrap_or_default();
        self.transfer_ownership(block_id, &current_user).await
    }
}
```

## 4. 冲突解决机制

```rust
// src/document/conflict_resolver.rs
pub struct ConflictResolver;

impl ConflictResolver {
    /// 处理合并后的冲突
    pub async fn resolve_conflicts(
        &self,
        document: &mut AutoCommit,
        changes: &[automerge::Change]
    ) -> Result<Vec<ConflictResolution>, ElfiError> {
        let mut resolutions = Vec::new();
        
        for change in changes {
            for op in change.operations() {
                if let Some(conflict_info) = self.detect_conflict(document, op)? {
                    let resolution = self.resolve_by_block_type(document, &conflict_info).await?;
                    resolutions.push(resolution);
                }
            }
        }
        
        Ok(resolutions)
    }
    
    async fn resolve_by_block_type(
        &self,
        document: &mut AutoCommit,
        conflict: &ConflictInfo
    ) -> Result<ConflictResolution, ElfiError> {
        let block_type = self.get_block_type(document, &conflict.block_id)?;
        
        match block_type {
            BlockType::Code => {
                // 代码冲突：标记为需要手动解决
                self.mark_manual_resolution_required(document, conflict).await
            }
            BlockType::Markdown => {
                // Markdown：信任 CRDT 自动合并
                Ok(ConflictResolution::AutoResolved {
                    block_id: conflict.block_id.clone(),
                    strategy: "crdt_merge".to_string(),
                })
            }
            BlockType::Recipe => {
                // Recipe：结构化冲突解决
                self.resolve_recipe_conflict(document, conflict).await
            }
            _ => {
                // 其他类型：默认标记为需要手动解决
                self.mark_manual_resolution_required(document, conflict).await
            }
        }
    }
    
    async fn mark_manual_resolution_required(
        &self,
        document: &mut AutoCommit,
        conflict: &ConflictInfo
    ) -> Result<ConflictResolution, ElfiError> {
        document.change(|doc| {
            let blocks = doc.get_or_create_map("blocks")?;
            let block = blocks.get_map(&conflict.block_id)
                .ok_or_else(|| automerge::AutomergeError::InvalidPath)?;
            let metadata = block.get_or_create_map("metadata")?;
            
            metadata.put("conflict", true)?;
            metadata.put("conflict_versions", serde_json::to_string(&conflict.versions)?)?;
            metadata.put("conflict_detected_at", chrono::Utc::now().timestamp())?;
            
            Ok(())
        })?;
        
        Ok(ConflictResolution::ManualResolutionRequired {
            block_id: conflict.block_id.clone(),
            conflict_versions: conflict.versions.clone(),
        })
    }
}
```

## 5. 性能优化和缓存

```rust
// src/cache.rs
pub struct CacheManager {
    memory_cache: Arc<DashMap<String, CachedDocument>>,
    disk_cache: Option<sled::Db>,
    config: CacheConfig,
}

impl CacheManager {
    pub fn new(config: CacheConfig) -> Result<Self, ElfiError> {
        let disk_cache = if config.enable_disk_cache {
            Some(sled::open(&config.cache_dir).map_err(|e| ElfiError::Io(e.into()))?)
        } else {
            None
        };
        
        Ok(Self {
            memory_cache: Arc::new(DashMap::new()),
            disk_cache,
            config,
        })
    }
    
    pub async fn get_document(&self, doc_id: &str) -> Option<AutoCommit> {
        // L1: 内存缓存
        if let Some(cached) = self.memory_cache.get(doc_id) {
            if !cached.is_expired() {
                return Some(cached.document.clone());
            }
        }
        
        // L2: 磁盘缓存
        if let Some(ref disk_cache) = self.disk_cache {
            if let Ok(Some(data)) = disk_cache.get(doc_id.as_bytes()) {
                if let Ok(doc) = AutoCommit::load(&data) {
                    self.memory_cache.insert(doc_id.to_string(), CachedDocument {
                        document: doc.clone(),
                        expires_at: SystemTime::now() + Duration::from_secs(self.config.memory_ttl),
                    });
                    return Some(doc);
                }
            }
        }
        
        None
    }
    
    pub async fn store_document(&self, doc_id: &str, document: &AutoCommit) -> Result<(), ElfiError> {
        // 存储到内存缓存
        self.memory_cache.insert(doc_id.to_string(), CachedDocument {
            document: document.clone(),
            expires_at: SystemTime::now() + Duration::from_secs(self.config.memory_ttl),
        });
        
        // 存储到磁盘缓存
        if let Some(ref disk_cache) = self.disk_cache {
            let data = document.save().map_err(ElfiError::DataModel)?;
            disk_cache.insert(doc_id.as_bytes(), data).map_err(|e| ElfiError::Io(e.into()))?;
        }
        
        Ok(())
    }
}
```

这个重新设计的核心逻辑提供了：

1. **清晰的 Main 接口**：每个 CLI 命令都有对应的函数
2. **模块化架构**：SessionManager、DocumentManager 等独立组件  
3. **异步优先**：全面支持异步操作
4. **丰富的 API**：支持所有用例场景需求
5. **性能优化**：多级缓存和并发控制
6. **错误处理**：统一的错误类型和用户友好的错误信息
7. **可观察性**：内置指标收集和结构化日志
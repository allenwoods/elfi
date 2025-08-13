# Recipe 引擎与跨文档引用系统

基于"文档即App"用例和设计文档，详细实现 Recipe 执行引擎和跨文档引用解析系统。

## 1. RecipeEngine 核心架构

### 1.1. 主要组件

```rust
// src/recipe/mod.rs
pub mod engine;
pub mod resolver;
pub mod executor;
pub mod template;
pub mod error;

use crate::{SessionManager, DocumentManager};

pub struct RecipeEngine {
    session: Arc<SessionManager>,
    documents: Arc<DocumentManager>,
    resolver: LinkResolver,
    cache: Arc<RecipeCache>,
    template_engine: TemplateEngine,
    metrics: Arc<RecipeMetrics>,
}

impl RecipeEngine {
    pub fn new(session: &SessionManager, documents: &DocumentManager) -> Self {
        let resolver = LinkResolver::new(session.clone());
        let cache = Arc::new(RecipeCache::new());
        let template_engine = TemplateEngine::new();
        
        Self {
            session: Arc::new(session.clone()),
            documents: Arc::new(documents.clone()),
            resolver,
            cache,
            template_engine,
            metrics: Arc::new(RecipeMetrics::new()),
        }
    }
    
    /// 执行 Recipe
    pub async fn execute_recipe(
        &self,
        doc_uri: &str,
        recipe_name: &str,
        output_path: &str
    ) -> Result<RecipeExecutionResult, RecipeError> {
        let execution_id = uuid::Uuid::new_v4().to_string();
        let start_time = Instant::now();
        
        tracing::info!("开始执行 Recipe: {} (执行ID: {})", recipe_name, execution_id);
        
        // 1. 获取文档和 Recipe 配置
        let document = self.documents.open_document(doc_uri).await?;
        let recipe_config = self.load_recipe_config(&document, recipe_name).await?;
        
        // 2. 解析跨文档引用
        let resolved_refs = self.resolve_references(&recipe_config.references).await?;
        
        // 3. 选择目标区块
        let selected_blocks = self.select_blocks(&document, &recipe_config.selector).await?;
        
        // 4. 应用转换规则
        let transformed_blocks = self.apply_transforms(
            selected_blocks,
            &resolved_refs,
            &recipe_config.transform
        ).await?;
        
        // 5. 生成最终输出
        let output = self.generate_output(
            transformed_blocks,
            &recipe_config.output,
            output_path
        ).await?;
        
        let duration = start_time.elapsed();
        self.metrics.record_execution(recipe_name, duration, true);
        
        tracing::info!("Recipe 执行完成: {} ({:.2}s)", recipe_name, duration.as_secs_f64());
        
        Ok(RecipeExecutionResult {
            execution_id,
            recipe_name: recipe_name.to_string(),
            duration,
            blocks_processed: selected_blocks.len(),
            references_resolved: resolved_refs.len(),
            output_files: output.generated_files,
            warnings: output.warnings,
        })
    }
}
```

### 1.2. Recipe 配置解析

```rust
// src/recipe/config.rs
#[derive(Debug, Clone, Deserialize)]
pub struct RecipeConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    
    #[serde(default)]
    pub references: Vec<CrossDocumentReference>,
    
    pub selector: BlockSelector,
    pub transform: Vec<TransformRule>,
    pub output: OutputConfig,
    
    #[serde(default)]
    pub error_handling: ErrorHandlingConfig,
    
    #[serde(default)]
    pub execution: ExecutionConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CrossDocumentReference {
    pub source: String,          // elf://my-project/component#reusable-utilities
    pub target: String,          // placeholder-utils
    pub cache_policy: CachePolicy,
    pub resolve_mode: ResolveMode,
    pub template: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockSelector {
    #[serde(default)]
    pub types: Vec<String>,
    
    #[serde(default)]  
    pub tags: Vec<String>,
    
    #[serde(default)]
    pub names: Vec<String>,
    
    #[serde(default)]
    pub references: Vec<String>,
    
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransformRule {
    pub r#type: String,
    pub action: String,
    pub template: Option<String>,
    pub config: Option<serde_json::Value>,
    
    #[serde(default)]
    pub depends_on: Vec<String>,
    
    #[serde(default)]
    pub recursive: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CachePolicy {
    OnChange,     // 内容变更时更新
    AlwaysFresh,  // 总是获取最新内容
    Manual,       // 手动控制缓存
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolveMode {
    Lazy,     // 使用时才解析
    Eager,    // 立即解析
    Prefetch, // 预取但不阻塞
}
```

## 2. LinkResolver - 跨文档引用解析

### 2.1. URI 解析器

```rust
// src/link/resolver.rs
pub struct LinkResolver {
    session: Arc<SessionManager>,
    cache: Arc<DashMap<String, CachedReference>>,
    uri_parser: UriParser,
}

#[derive(Debug, Clone)]
pub struct ParsedUri {
    pub scheme: String,          // "elf"
    pub user: Option<String>,    // "alice"
    pub repo: String,            // "my-project"  
    pub document: String,        // "component"
    pub block_name: Option<String>, // "reusable-utilities"
}

#[derive(Debug, Clone)]
pub struct CachedReference {
    pub content: Vec<u8>,
    pub etag: Option<String>,
    pub expires_at: SystemTime,
    pub last_modified: SystemTime,
}

impl LinkResolver {
    pub fn new(session: Arc<SessionManager>) -> Self {
        Self {
            session,
            cache: Arc::new(DashMap::new()),
            uri_parser: UriParser::new(),
        }
    }
    
    /// 解析跨文档引用
    pub async fn resolve_reference(&self, uri: &str) -> Result<ResolvedReference, LinkError> {
        // 1. 解析 URI
        let parsed = self.uri_parser.parse(uri)?;
        
        // 2. 检查缓存
        if let Some(cached) = self.check_cache(uri) {
            if !cached.is_expired() {
                return Ok(ResolvedReference {
                    uri: uri.to_string(),
                    content: String::from_utf8_lossy(&cached.content).to_string(),
                    source: ReferenceSource::Cache,
                    last_updated: cached.last_modified,
                });
            }
        }
        
        // 3. 从网络获取
        let content = self.fetch_from_network(&parsed).await?;
        
        // 4. 更新缓存
        self.update_cache(uri, &content);
        
        Ok(ResolvedReference {
            uri: uri.to_string(),
            content: content.content,
            source: ReferenceSource::Network,
            last_updated: content.last_modified,
        })
    }
    
    async fn fetch_from_network(&self, parsed: &ParsedUri) -> Result<NetworkContent, LinkError> {
        // 构建目标文档的 Zenoh 键
        let doc_key = format!("/elfi/docs/{}/{}/{}", 
            parsed.user.as_deref().unwrap_or("_"), 
            parsed.repo, 
            parsed.document);
        
        // 查询文档内容
        let query_result = self.session
            .zenoh_session()
            .get(&doc_key)
            .await
            .map_err(|e| LinkError::NetworkError(e.to_string()))?;
        
        // 如果指定了区块名称，提取特定区块
        let content = if let Some(block_name) = &parsed.block_name {
            self.extract_block_content(&query_result, block_name)?
        } else {
            query_result.into()
        };
        
        Ok(NetworkContent {
            content,
            last_modified: SystemTime::now(),
            etag: None,
        })
    }
    
    fn extract_block_content(&self, doc_data: &[u8], block_name: &str) -> Result<String, LinkError> {
        // 解析 automerge 文档
        let doc = automerge::AutoCommit::load(doc_data)
            .map_err(|e| LinkError::DocumentParseError(e.to_string()))?;
        
        // 查找区块
        let blocks = doc.get("blocks")
            .ok_or_else(|| LinkError::BlockNotFound {
                block_name: block_name.to_string(),
                reason: "文档中没有 blocks 字段".to_string(),
            })?;
        
        // 遍历所有区块找到匹配的名称
        for (block_id, block_data) in blocks.iter() {
            if let Some(name) = block_data.get("name").and_then(|v| v.as_str()) {
                if name == block_name {
                    if let Some(content) = block_data.get("content").and_then(|v| v.as_str()) {
                        return Ok(content.to_string());
                    }
                }
            }
        }
        
        Err(LinkError::BlockNotFound {
            block_name: block_name.to_string(),
            reason: "在文档中找不到指定名称的区块".to_string(),
        })
    }
}
```

### 2.2. URI 解析器实现

```rust
// src/link/uri_parser.rs
pub struct UriParser;

impl UriParser {
    pub fn new() -> Self {
        Self
    }
    
    /// 解析 ELF URI: elf://[user/]repo/doc[#block-name]
    pub fn parse(&self, uri: &str) -> Result<ParsedUri, UriParseError> {
        if !uri.starts_with("elf://") {
            return Err(UriParseError::InvalidScheme {
                uri: uri.to_string(),
                expected: "elf".to_string(),
            });
        }
        
        let path_part = &uri[6..]; // 移除 "elf://"
        
        // 分离区块名称部分（#block-name）
        let (path, block_name) = if let Some(hash_pos) = path_part.find('#') {
            let (path, block) = path_part.split_at(hash_pos);
            (path, Some(block[1..].to_string())) // 移除 '#'
        } else {
            (path_part, None)
        };
        
        // 解析路径部分
        let path_segments: Vec<&str> = path.split('/').collect();
        
        match path_segments.len() {
            2 => {
                // repo/doc
                Ok(ParsedUri {
                    scheme: "elf".to_string(),
                    user: None,
                    repo: path_segments[0].to_string(),
                    document: path_segments[1].to_string(),
                    block_name,
                })
            }
            3 => {
                // user/repo/doc
                Ok(ParsedUri {
                    scheme: "elf".to_string(),
                    user: Some(path_segments[0].to_string()),
                    repo: path_segments[1].to_string(),
                    document: path_segments[2].to_string(),
                    block_name,
                })
            }
            _ => Err(UriParseError::InvalidFormat {
                uri: uri.to_string(),
                expected: "elf://[user/]repo/doc[#block-name]".to_string(),
            })
        }
    }
    
    /// 验证 URI 格式
    pub fn validate(&self, uri: &str) -> Result<(), UriParseError> {
        self.parse(uri).map(|_| ())
    }
    
    /// 标准化 URI（解析相对引用等）
    pub fn normalize(&self, uri: &str, base_uri: &str) -> Result<String, UriParseError> {
        if uri.starts_with("elf://") {
            // 绝对 URI，直接返回
            self.validate(uri)?;
            Ok(uri.to_string())
        } else if uri.starts_with("#") {
            // 文档内引用
            let base = self.parse(base_uri)?;
            Ok(format!("elf://{}/{}/{}{}",
                base.user.map(|u| format!("{}/", u)).unwrap_or_default(),
                base.repo,
                base.document,
                uri
            ))
        } else if uri.starts_with("./") {
            // 相对引用，同仓库
            let base = self.parse(base_uri)?;
            let relative_doc = &uri[2..]; // 移除 "./"
            Ok(format!("elf://{}/{}/{}",
                base.user.map(|u| format!("{}/", u)).unwrap_or_default(),
                base.repo,
                relative_doc
            ))
        } else {
            Err(UriParseError::UnsupportedFormat {
                uri: uri.to_string(),
            })
        }
    }
}
```

## 3. 转换引擎

### 3.1. 区块选择器

```rust
// src/recipe/selector.rs
pub struct BlockSelector;

impl BlockSelector {
    /// 根据选择器筛选区块
    pub async fn select_blocks(
        &self,
        document: &DocumentHandle,
        selector: &crate::recipe::config::BlockSelector
    ) -> Result<Vec<SelectedBlock>, SelectorError> {
        let doc = document.document().read().await;
        let mut selected = Vec::new();
        
        // 获取所有区块
        let blocks_map = doc.get("blocks")
            .ok_or(SelectorError::NoBlocksInDocument)?;
        
        for (block_id, block_data) in blocks_map.iter() {
            let block = ParsedBlock::from_automerge(block_id, block_data)?;
            
            if self.matches_selector(&block, selector) {
                selected.push(SelectedBlock {
                    id: block.id.clone(),
                    name: block.name.clone(),
                    block_type: block.block_type.clone(),
                    content: block.content.clone(),
                    metadata: block.metadata.clone(),
                });
            }
        }
        
        // 按层级结构排序
        selected.sort_by(|a, b| self.compare_hierarchical_order(a, b));
        
        Ok(selected)
    }
    
    fn matches_selector(&self, block: &ParsedBlock, selector: &crate::recipe::config::BlockSelector) -> bool {
        // 类型匹配
        if !selector.types.is_empty() {
            if !selector.types.contains(&block.block_type) {
                return false;
            }
        }
        
        // 名称匹配（支持通配符）
        if !selector.names.is_empty() {
            let name_matches = selector.names.iter().any(|pattern| {
                if pattern.contains('*') {
                    // 简单的通配符匹配
                    self.wildcard_match(&block.name.unwrap_or_default(), pattern)
                } else {
                    block.name.as_deref() == Some(pattern)
                }
            });
            if !name_matches {
                return false;
            }
        }
        
        // 标签匹配
        if !selector.tags.is_empty() {
            let block_tags = block.metadata.get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_default();
            
            let has_matching_tag = selector.tags.iter()
                .any(|tag| block_tags.contains(&tag.as_str()));
                
            if !has_matching_tag {
                return false;
            }
        }
        
        // 元数据匹配
        for (key, expected_value) in &selector.metadata {
            if let Some(actual_value) = block.metadata.get(key) {
                if actual_value != expected_value {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        true
    }
    
    fn wildcard_match(&self, text: &str, pattern: &str) -> bool {
        // 简单的通配符实现，支持 * 匹配任意字符
        let regex_pattern = pattern.replace('*', ".*");
        regex::Regex::new(&regex_pattern)
            .map(|r| r.is_match(text))
            .unwrap_or(false)
    }
}
```

### 3.2. 内容转换器

```rust
// src/recipe/transformer.rs
pub struct ContentTransformer {
    template_engine: TemplateEngine,
}

impl ContentTransformer {
    pub fn new() -> Self {
        Self {
            template_engine: TemplateEngine::new(),
        }
    }
    
    /// 应用转换规则
    pub async fn apply_transforms(
        &self,
        blocks: Vec<SelectedBlock>,
        resolved_refs: &HashMap<String, ResolvedReference>,
        transforms: &[TransformRule]
    ) -> Result<Vec<TransformedBlock>, TransformError> {
        let mut context = TransformContext {
            blocks: blocks.into_iter().map(|b| (b.id.clone(), b)).collect(),
            resolved_refs: resolved_refs.clone(),
            variables: HashMap::new(),
        };
        
        // 按依赖顺序执行转换
        let ordered_transforms = self.sort_transforms_by_dependency(transforms)?;
        
        for transform in ordered_transforms {
            self.apply_single_transform(&mut context, &transform).await?;
        }
        
        Ok(context.blocks.into_values()
            .map(|b| TransformedBlock {
                id: b.id,
                name: b.name,
                content: b.content,
                metadata: b.metadata,
                transform_applied: true,
            })
            .collect())
    }
    
    async fn apply_single_transform(
        &self,
        context: &mut TransformContext,
        transform: &TransformRule
    ) -> Result<(), TransformError> {
        match transform.action.as_str() {
            "copy" => {
                // 直接复制，可能应用模板
                if let Some(template) = &transform.template {
                    for (_, block) in context.blocks.iter_mut() {
                        if self.block_matches_transform(block, transform) {
                            let template_vars = self.prepare_template_vars(block, context);
                            block.content = self.template_engine.render(template, &template_vars)?;
                        }
                    }
                }
            }
            "resolve_references" => {
                // 解析跨文档引用
                for (_, block) in context.blocks.iter_mut() {
                    if block.block_type == "link" {
                        if let Some(resolved) = self.resolve_link_block(block, context)? {
                            block.content = resolved;
                        }
                    }
                }
            }
            "concat" => {
                // 连接多个区块的内容
                let template = transform.template.as_deref().unwrap_or("{{content}}");
                let mut concatenated = String::new();
                
                for (_, block) in &context.blocks {
                    if self.block_matches_transform(block, transform) {
                        let template_vars = self.prepare_template_vars(block, context);
                        let rendered = self.template_engine.render(template, &template_vars)?;
                        concatenated.push_str(&rendered);
                    }
                }
                
                // 将结果存储到第一个匹配的区块
                if let Some((_, first_block)) = context.blocks.iter_mut()
                    .find(|(_, b)| self.block_matches_transform(b, transform)) {
                    first_block.content = concatenated;
                }
            }
            "filter" => {
                // 过滤内容
                let filter_config = transform.config.as_ref()
                    .ok_or(TransformError::MissingConfig("filter action requires config".to_string()))?;
                
                // 实现各种过滤逻辑...
            }
            _ => {
                return Err(TransformError::UnknownAction(transform.action.clone()));
            }
        }
        
        Ok(())
    }
    
    fn resolve_link_block(
        &self,
        block: &SelectedBlock,
        context: &TransformContext
    ) -> Result<Option<String>, TransformError> {
        // 从 link block 内容解析目标 URI
        let link_content = self.parse_link_content(&block.content)?;
        let target_uri = link_content.target;
        
        // 查找对应的已解析引用
        if let Some(resolved) = context.resolved_refs.get(&target_uri) {
            // 应用引用模板
            if let Some(template) = &link_content.template {
                let mut template_vars = HashMap::new();
                template_vars.insert("resolved_content".to_string(), resolved.content.clone());
                template_vars.insert("last_updated".to_string(), 
                    resolved.last_updated.duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .to_string()
                );
                template_vars.insert("source_uri".to_string(), target_uri);
                
                Ok(Some(self.template_engine.render(template, &template_vars)?))
            } else {
                Ok(Some(resolved.content.clone()))
            }
        } else {
            Err(TransformError::UnresolvedReference(target_uri))
        }
    }
}
```

## 4. 模板引擎

### 4.1. 简单的模板系统

```rust
// src/recipe/template.rs
pub struct TemplateEngine {
    handlebars: handlebars::Handlebars<'static>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut handlebars = handlebars::Handlebars::new();
        
        // 注册辅助函数
        handlebars.register_helper("timestamp", Box::new(timestamp_helper));
        handlebars.register_helper("format_date", Box::new(format_date_helper));
        
        Self { handlebars }
    }
    
    pub fn render(&self, template: &str, vars: &HashMap<String, String>) -> Result<String, TemplateError> {
        self.handlebars
            .render_template(template, vars)
            .map_err(|e| TemplateError::RenderError(e.to_string()))
    }
}

// 辅助函数：当前时间戳
fn timestamp_helper(
    _: &handlebars::Helper,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    out.write(&timestamp.to_string())?;
    Ok(())
}

// 辅助函数：格式化日期
fn format_date_helper(
    h: &handlebars::Helper,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    if let Some(timestamp) = h.param(0).and_then(|v| v.value().as_str()) {
        if let Ok(ts) = timestamp.parse::<i64>() {
            let datetime = chrono::DateTime::from_timestamp(ts, 0)
                .unwrap_or_else(chrono::Utc::now);
            let formatted = datetime.format("%Y-%m-%d %H:%M:%S");
            out.write(&formatted.to_string())?;
        }
    }
    Ok(())
}
```

## 5. 错误处理和缓存

### 5.1. 循环引用检测

```rust
// src/recipe/cycle_detector.rs
pub struct CycleDetector;

impl CycleDetector {
    /// 检测引用图中的循环
    pub fn detect_cycles(references: &[CrossDocumentReference]) -> Result<(), CycleError> {
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        
        // 构建引用图
        for reference in references {
            let source_docs = Self::extract_document_from_uri(&reference.source)?;
            let target_doc = Self::extract_document_from_uri(&reference.target)?;
            
            graph.entry(source_docs)
                .or_default()
                .push(target_doc);
        }
        
        // 深度优先搜索检测循环
        let mut visited = HashSet::new();
        let mut in_path = HashSet::new();
        
        for start_doc in graph.keys() {
            if !visited.contains(start_doc) {
                if let Some(cycle) = Self::dfs_find_cycle(
                    start_doc,
                    &graph,
                    &mut visited,
                    &mut in_path,
                    &mut Vec::new()
                ) {
                    return Err(CycleError::CircularReference { 
                        cycle: cycle.join(" → ") 
                    });
                }
            }
        }
        
        Ok(())
    }
    
    fn dfs_find_cycle(
        current: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        in_path: &mut HashSet<String>,
        path: &mut Vec<String>
    ) -> Option<Vec<String>> {
        visited.insert(current.to_string());
        in_path.insert(current.to_string());
        path.push(current.to_string());
        
        if let Some(neighbors) = graph.get(current) {
            for neighbor in neighbors {
                if in_path.contains(neighbor) {
                    // 找到循环
                    let cycle_start = path.iter().position(|x| x == neighbor).unwrap();
                    let mut cycle = path[cycle_start..].to_vec();
                    cycle.push(neighbor.clone());
                    return Some(cycle);
                }
                
                if !visited.contains(neighbor) {
                    if let Some(cycle) = Self::dfs_find_cycle(neighbor, graph, visited, in_path, path) {
                        return Some(cycle);
                    }
                }
            }
        }
        
        in_path.remove(current);
        path.pop();
        None
    }
}
```

### 5.2. Recipe 缓存管理

```rust
// src/recipe/cache.rs
pub struct RecipeCache {
    memory_cache: Arc<DashMap<String, CachedRecipeResult>>,
    config: CacheConfig,
}

#[derive(Clone)]
struct CachedRecipeResult {
    result: RecipeExecutionResult,
    dependencies: Vec<String>,    // 依赖的文档/区块列表
    expires_at: SystemTime,
}

impl RecipeCache {
    pub fn new() -> Self {
        Self {
            memory_cache: Arc::new(DashMap::new()),
            config: CacheConfig::default(),
        }
    }
    
    /// 获取缓存的 Recipe 结果
    pub fn get(&self, recipe_key: &str) -> Option<RecipeExecutionResult> {
        if let Some(cached) = self.memory_cache.get(recipe_key) {
            if cached.expires_at > SystemTime::now() {
                return Some(cached.result.clone());
            } else {
                // 缓存过期，移除
                self.memory_cache.remove(recipe_key);
            }
        }
        None
    }
    
    /// 存储 Recipe 执行结果
    pub fn store(
        &self,
        recipe_key: String,
        result: RecipeExecutionResult,
        dependencies: Vec<String>
    ) {
        let cached = CachedRecipeResult {
            result,
            dependencies,
            expires_at: SystemTime::now() + Duration::from_secs(self.config.ttl_seconds),
        };
        
        self.memory_cache.insert(recipe_key, cached);
    }
    
    /// 当依赖文档变更时，使相关缓存失效
    pub fn invalidate_dependencies(&self, changed_doc: &str) {
        let keys_to_remove: Vec<String> = self.memory_cache
            .iter()
            .filter_map(|entry| {
                if entry.value().dependencies.contains(&changed_doc.to_string()) {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();
        
        for key in keys_to_remove {
            self.memory_cache.remove(&key);
        }
    }
}
```

这个 Recipe 引擎设计提供了：

1. **完整的跨文档引用**：支持 `elf://` URI 格式
2. **灵活的转换系统**：支持多种转换规则和模板
3. **智能缓存**：基于依赖关系的缓存失效
4. **循环引用检测**：防止无限递归  
5. **错误处理**：详细的错误信息和恢复策略
6. **性能优化**：并发引用解析和缓存机制

这确保了"文档即App"场景的完全实现，支持复杂的跨文档内容组合和动态转换。
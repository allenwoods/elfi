# 3.c recipe 模块开发计划

**阶段**: 第三阶段 - 应用层 (并行)
**关联文件**: [01-overview.md](./01-overview.md), [04-phase1-c-core.md](./04-phase1-c-core.md), [06-phase3-a-weave.md](./06-phase3-a-weave.md), [06-phase3-b-tangle.md](./06-phase3-b-tangle.md)

## 模块职责
Recipe系统实现、内容转换、跨文档引用管理。

## 数据结构设计

### RecipeEngine结构
```rust
pub struct RecipeEngine {
    handlebars: Handlebars<'static>,
    recipe_cache: DashMap<String, CompiledRecipe>,
    reference_resolver: ReferenceResolver,
}
```

### CompiledRecipe结构
```rust
pub struct CompiledRecipe {
    pub template: Template,
    pub dependencies: Vec<String>,
    pub output_format: String,
    pub metadata: RecipeMetadata,
}
```

## API接口定义

```rust
pub trait RecipeInterface {
    async fn compile_recipe(recipe_content: &str) -> Result<CompiledRecipe>;
    async fn execute_recipe(recipe_name: &str, context: &RecipeContext) -> Result<ExportResult>;
    async fn resolve_reference(reference: &str) -> Result<ResolvedContent>;
    async fn list_recipes(doc_uri: &str) -> Result<Vec<RecipeInfo>>;
    async fn export_document(doc_uri: &str, recipe_name: &str, output_path: &str) -> Result<ExportResult>;
}
```

## 功能点覆盖
- [ ] Recipe模板编译
- [ ] 内容转换执行
- [ ] 跨文档引用解析
- [ ] 依赖关系管理
- [ ] 缓存和优化
- [ ] 导出结果管理

## 依赖其他模块
- elfi-core: CoreInterface
- elfi-types: 所有数据类型

## 测试策略
- Recipe模板编译
- 内容转换功能
- 跨文档引用解析
- 复杂依赖关系测试
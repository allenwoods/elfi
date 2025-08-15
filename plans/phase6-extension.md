# 阶段6b: Extension 模块开发计划

**阶段**: 阶段6 - 扩展层 (并行)
**关联文件**: [01-overview.md](./01-overview.md), [phase3-core.md](./phase3-core.md), [phase6-recipe.md](./phase6-recipe.md), [phase7-cli.md](./phase7-cli.md)

## 模块职责
插件系统实现、多语言绑定、WebAssembly支持。

## 数据结构设计

### ExtensionManager结构
```rust
pub struct ExtensionManager {
    plugins: DashMap<String, Box<dyn Plugin>>,
    ffi_bindings: FfiBindings,
    wasm_runtime: WasmRuntime,
}
```

### Plugin trait
```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    async fn initialize(&mut self, context: PluginContext) -> Result<()>;
    async fn handle_block(&self, block: &Block) -> Result<ProcessedBlock>;
    async fn resolve_conflict(&self, conflict: &Conflict) -> Result<ConflictResolution>;
}
```

## API接口定义

```rust
pub trait ExtensionInterface {
    async fn load_plugin(path: &str) -> Result<PluginHandle>;
    async fn unload_plugin(name: &str) -> Result<()>;
    async fn list_plugins() -> Result<Vec<PluginInfo>>;
    fn create_ffi_bindings() -> Result<FfiBindings>;
    fn create_wasm_bindings() -> Result<WasmBindings>;
}
```

## 功能点覆盖
- [ ] 插件加载和管理
- [ ] C FFI绑定
- [ ] WebAssembly支持
- [ ] Python/Node.js绑定
- [ ] 插件API框架
- [ ] 安全沙箱机制

## 依赖其他模块
- elfi-core: 所有核心接口
- elfi-types: 所有数据类型

## 测试策略
- 插件加载和卸载
- FFI绑定功能
- WASM运行时测试
- 安全性测试
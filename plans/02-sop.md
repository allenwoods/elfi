# 2. ELFI 开发流程规范 (SOP)

**关联文件**: [01-overview.md](./01-overview.md), [08-integration.md](./08-integration.md)

## TDD开发流程

### 模块开发阶段流程

#### A. 获取背景信息
```bash
# 查看项目全貌
cat docs/src/implementations/00-overview.md
cat docs/src/designs/01-data_modeling.md

# 查看相关用例
cat docs/src/usecases/00-overview.md
```

#### B. 制定开发计划
1. 创建`plans/{module}.md`文件
2. 必须包含:
   - 数据结构设计
   - API接口定义  
   - 功能点覆盖清单
   - 依赖其他模块的接口

#### C. 编写单元测试
```rust
// tests/{module}_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试目标: src/{module}/core.rs 的 create_document 函数
    /// 依赖模块: elfi_storage::StorageInterface (使用Mock)
    #[tokio::test]
    async fn test_create_document() {
        // 使用真实的模块实现 + Mock依赖
        let storage = MockStorage::new();
        let module = MyModule::new(Box::new(storage));
        
        // 测试逻辑
        let result = module.create_document("test").await;
        assert!(result.is_ok());
    }
}
```

**测试要求**:
- 每个测试必须注释说明测试目标代码文件
- 必须使用真实模块实现，不能mock主要逻辑
- 依赖其他模块时使用Interface + Mock数据
- 如果Interface不存在，测试应报错提醒对应开发者

#### D. 运行测试并更新计划
```bash
cargo test {module}
```
根据错误信息更新`plans/{module}.md`中的TODO清单。

#### E. 实现模块功能
```rust
// src/{module}/mod.rs
pub trait ModuleInterface {
    async fn key_function(&self) -> Result<Output>;
}

pub struct ModuleImpl {
    dependency: Box<dyn DependencyInterface>,
}

impl ModuleInterface for ModuleImpl {
    async fn key_function(&self) -> Result<Output> {
        // 真实实现，不是简单返回期望结果
        let input = self.process_input();
        self.dependency.call_other_module(input).await
    }
}
```

**实现要求**:
- 真实的业务逻辑，不能硬编码返回值
- 常量和配置通过参数或配置文件传入
- 为后续扩展保留可修改的接入点

#### F. 重复D-E直到完成

#### G. 更新文档
```bash
# 按顺序更新
1. docs/src/designs/{module}.md    # 设计文档
2. docs/src/implementations/{module}.md  # 实现文档
3. docs/src/03-cheatsheet.md       # 命令参考
```

#### H. 检查注意事项
更新`CLAUDE.md`中需要后续开发注意的问题。

#### I. 验证用例满足
检查是否满足`docs/src/usecases/`中的需求，能否通过组合实现。

### 集成开发阶段流程

**前提**: 所有模块开发完成

#### 1. 集成测试设计
```rust
// tests/integration_test.rs

/// 端到端测试: 对话即文档用例
/// 涉及模块: Core + Storage + Weave
/// 测试文件: docs/src/usecases/conversation.elf
#[tokio::test]
async fn test_conversation_as_document() {
    // 加载测试用例文件
    let test_file = include_str!("../docs/src/usecases/conversation.elf");
    
    // 端到端测试流程
    let main = Main::new().await;
    let doc = main.open("test://conversation").await?;
    
    // 验证完整工作流
    // ...
}
```

#### 2. 集成问题处理
- 发现模块接口不足时: 在对应模块添加TODO，不修改实现
- 发现设计问题时: 记录到集成测试，不进行架构修改
- 优先通过组合解决，避免over-engineering

## 代码质量保证

### 测试覆盖要求
```bash
# 单元测试覆盖率
cargo tarpaulin --out Html
# 要求覆盖率 > 80%

# 集成测试验证  
cargo test --test integration
```

### 代码审查流程
```bash
# 运行所有质量检查
just test          # 单元测试
just lint           # 代码格式
just typecheck      # 类型检查
```

### 提交规范
```bash
# 提交信息格式
git commit -m "feat(core): 实现CRDT文档管理

- 添加Document结构体和基础CRUD操作
- 实现事件溯源的操作日志
- 添加单元测试覆盖所有公共API

🤖 Generated with Claude Code"
```

## 模块接口协作规范

### Interface定义模式
```rust
// elfi-{module}/src/interface.rs
pub trait ModuleInterface: Send + Sync {
    async fn primary_function(&self, input: Input) -> Result<Output>;
}

// elfi-{module}/src/mock.rs  
#[cfg(test)]
pub struct MockModule {
    responses: HashMap<String, Output>,
}

impl ModuleInterface for MockModule {
    async fn primary_function(&self, input: Input) -> Result<Output> {
        self.responses.get(&input.key).cloned()
            .ok_or_else(|| anyhow!("Mock not configured for {}", input.key))
    }
}
```

### 依赖注入模式
```rust
pub struct MyModule<S: StorageInterface> {
    storage: S,
    config: ModuleConfig,
}

impl<S: StorageInterface> MyModule<S> {
    pub fn new(storage: S, config: ModuleConfig) -> Self {
        Self { storage, config }
    }
}
```

### 错误处理统一
```rust
// elfi-types/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ElfiError {
    #[error("Module not implemented: {module}")]
    NotImplemented { module: String },
    
    #[error("Dependency error: {source}")]
    Dependency { source: Box<dyn std::error::Error> },
}
```

## 文档维护流程

### 实现完成后的文档更新顺序
1. **设计文档**: `docs/src/designs/{module}.md`
2. **实现文档**: `docs/src/implementations/{module}.md`  
3. **API参考**: `docs/src/03-cheatsheet.md`
4. **项目上下文**: `CLAUDE.md`

### 文档内容要求
- 设计文档: 架构决策和设计原理
- 实现文档: 具体实现细节和使用方法
- API参考: 所有命令的完整列表
- 项目上下文: 开发注意事项和约定

### 文档验证
```bash
# 构建文档验证
cd docs && just build

# 检查链接有效性
cd docs && just validate
```

## 性能和质量指标

### 单元测试指标
- 测试覆盖率 > 80%
- 所有公共API有测试
- 边界条件覆盖完整

### 集成测试指标  
- 三大用例端到端通过
- 并发场景稳定性测试
- 内存泄漏检测通过

### 性能基准
```rust
#[bench]
fn bench_document_sync(b: &mut Bencher) {
    b.iter(|| {
        // 关键路径性能测试
    });
}
```

关键性能指标:
- 文档同步延迟 < 100ms
- 内存使用 < 100MB (单文档)
- 并发用户数 > 10
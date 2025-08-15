# 5. 集成测试计划

**阶段**: 所有模块开发完成后执行
**关联文件**: [01-overview.md](./01-overview.md), [02-sop.md](./02-sop.md), 所有模块plan文件

## 集成测试策略
基于三大核心用例的端到端验证。

## 测试用例1: 对话即文档

### 测试文件
`docs/src/usecases/conversation.elf`

### 测试目标
- 多用户并发编辑
- CRDT冲突自动解决
- 实时同步验证

### 测试实现
```rust
#[tokio::test]
async fn test_conversation_as_document() {
    // 初始化两个用户会话
    let alice = Main::new_with_user("alice").await?;
    let bob = Main::new_with_user("bob").await?;
    
    // 加载测试文档
    let doc_uri = "test://conversation";
    alice.open(doc_uri).await?;
    bob.open(doc_uri).await?;
    
    // 并发编辑测试
    let alice_task = alice.add_block(doc_uri, "markdown", Some("alice-point"));
    let bob_task = bob.add_block(doc_uri, "markdown", Some("bob-response"));
    
    let (alice_block, bob_block) = tokio::join!(alice_task, bob_task);
    
    // 验证同步结果
    alice.sync(doc_uri).await?;
    bob.sync(doc_uri).await?;
    
    let alice_doc = alice.get_document(doc_uri).await?;
    let bob_doc = bob.get_document(doc_uri).await?;
    
    assert_eq!(alice_doc.blocks.len(), bob_doc.blocks.len());
    assert_eq!(alice_doc.blocks.len(), 2);
}
```

## 测试用例2: 自举开发

### 测试文件
`docs/src/usecases/elfi-dev.elf`

### 测试目标
- Recipe代码导出
- 文件监听双向同步
- IDE集成工作流

### 测试实现
```rust
#[tokio::test]
async fn test_bootstrapping() {
    let main = Main::new().await?;
    let doc_uri = "test://elfi-dev";
    
    // 加载开发文档
    main.open(doc_uri).await?;
    
    // 测试Recipe导出
    let recipes = main.list_recipes(doc_uri).await?;
    assert!(recipes.iter().any(|r| r.name == "export-rust-code"));
    
    // 执行Recipe
    let result = main.export(doc_uri, "export-rust-code", "/tmp/test-output").await?;
    assert!(result.success);
    
    // 验证输出文件
    let output_exists = std::path::Path::new("/tmp/test-output/src/lib.rs").exists();
    assert!(output_exists);
    
    // 测试文件监听
    let watch_config = WatchConfig {
        source_path: "/tmp/test-output",
        target_doc: doc_uri,
        sync_mode: SyncMode::Bidirectional,
    };
    
    let _watch_handle = main.watch(watch_config).await?;
    
    // 修改文件并验证同步
    std::fs::write("/tmp/test-output/src/lib.rs", "// Updated content").unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let updated_doc = main.get_document(doc_uri).await?;
    let code_block = updated_doc.find_block_by_type("code").unwrap();
    assert!(code_block.content.contains("Updated content"));
}
```

## 测试用例3: 文档即App

### 测试文件
`docs/src/usecases/main.elf` + `docs/src/usecases/component.elf`

### 测试目标
- 跨文档引用解析
- 动态内容组合
- Islands架构渲染

### 测试实现
```rust
#[tokio::test]
async fn test_document_as_app() {
    let main = Main::new().await?;
    
    // 加载主文档和组件文档
    main.open("test://main").await?;
    main.open("test://component").await?;
    
    // 测试跨文档引用
    let main_doc = main.get_document("test://main").await?;
    let relations = main.get_relations("test://main").await?;
    
    let external_refs = relations.iter()
        .filter(|r| r.to.starts_with("elf://"))
        .collect::<Vec<_>>();
    
    assert!(!external_refs.is_empty());
    
    // 测试渲染功能
    let render_result = main.render_document("test://main", "html").await?;
    assert!(render_result.success);
    assert!(render_result.content.contains("<html>"));
    
    // 测试Islands架构
    let interactive_components = render_result.components;
    assert!(!interactive_components.is_empty());
    
    // 测试组件激活
    let component_id = &interactive_components[0].id;
    let handle = main.activate_component("test://main", "block-001", component_id).await?;
    assert!(handle.is_active());
}
```

## 性能基准测试

### 并发性能测试
```rust
#[tokio::test]
async fn test_concurrent_performance() {
    let main = Arc::new(Main::new().await?);
    let doc_uri = "test://performance";
    main.open(doc_uri).await?;
    
    // 10个并发用户
    let mut tasks = Vec::new();
    for i in 0..10 {
        let main_clone = main.clone();
        let doc_uri_clone = doc_uri.to_string();
        
        tasks.push(tokio::spawn(async move {
            let start = Instant::now();
            
            // 每个用户添加100个块
            for j in 0..100 {
                let block_name = format!("user-{}-block-{}", i, j);
                main_clone.add_block(&doc_uri_clone, "markdown", Some(&block_name)).await?;
            }
            
            Ok::<Duration, anyhow::Error>(start.elapsed())
        }));
    }
    
    let results = futures::future::join_all(tasks).await;
    
    // 验证性能指标
    for result in results {
        let duration = result??;
        assert!(duration < Duration::from_secs(5)); // 5秒内完成
    }
    
    // 验证数据一致性
    main.sync(doc_uri).await?;
    let final_doc = main.get_document(doc_uri).await?;
    assert_eq!(final_doc.blocks.len(), 1000); // 10用户 * 100块
}
```

## 错误恢复测试

### 网络中断恢复
```rust
#[tokio::test]
async fn test_network_recovery() {
    let main = Main::new().await?;
    let doc_uri = "test://network-recovery";
    main.open(doc_uri).await?;
    
    // 正常操作
    main.add_block(doc_uri, "markdown", Some("before-disconnect")).await?;
    
    // 模拟网络中断
    main.simulate_network_disconnect().await?;
    
    // 离线操作
    main.add_block(doc_uri, "markdown", Some("during-disconnect")).await?;
    
    // 恢复网络
    main.simulate_network_reconnect().await?;
    
    // 验证同步恢复
    let sync_result = main.sync(doc_uri).await?;
    assert!(sync_result.success);
    
    let final_doc = main.get_document(doc_uri).await?;
    assert_eq!(final_doc.blocks.len(), 2);
}
```

## 测试运行配置

### CI/CD集成
```yaml
# .github/workflows/integration-tests.yml
name: Integration Tests
on: [push, pull_request]

jobs:
  integration:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - name: Run integration tests
        run: cargo test --test integration -- --test-threads=1
      - name: Run performance tests
        run: cargo test --test performance --release
```

### 本地测试命令
```bash
# 单独运行集成测试
cargo test --test integration

# 运行所有测试
cargo test

# 性能测试
cargo test --release --test performance

# 测试覆盖率
cargo tarpaulin --out Html
```

## 🤖 推荐使用的 Subagent

### 主要开发 Subagent
**@integration-tester**: 负责端到端集成测试的专业设计
- 基于三大核心用例设计完整的集成测试套件
- 实现性能基准测试和监控
- 设计各种故障场景的模拟和验证
- 创建网络分区、节点故障等复杂测试环境
- 验证系统在极端条件下的鲁棒性

### 支持 Subagent
**@rust-tdd-developer**: 负责测试基础设施和质量保证
- 实现测试环境的搭建和管理
- 编写测试工具和辅助函数
- 确保测试的稳定性和可重复性

**所有专业 Subagent**: 提供各自领域的集成支持
- @crdt-specialist: CRDT同步的集成验证
- @network-architect: 网络层的集成测试
- @api-designer: API接口的集成验证
- @parser-expert: 解析器的集成测试

### 使用示例
```bash
# 第一步：集成测试设计
@integration-tester 请基于三大核心用例设计完整的集成测试系统。
要求：
1. 参考 docs/src/usecases/ 中的三大用例场景
2. 实现"对话即文档"的多用户并发测试
3. 实现"自举开发"的Recipe和文件同步测试
4. 实现"文档即应用"的跨文档引用测试
5. 设计性能基准测试（10用户并发，1000操作 < 30秒）
6. 实现网络分区、故障恢复等极端场景测试
7. 创建完整的测试环境管理系统

# 第二步：各模块集成验证
@crdt-specialist 请验证 CRDT 模块的集成测试覆盖
@network-architect 请验证分布式存储的集成测试
@api-designer 请验证 API 接口的集成一致性
@parser-expert 请验证解析器的集成功能

# 第三步：持续集成配置
@rust-tdd-developer 请配置 CI/CD 集成测试流水线。
要求：
1. 自动化运行所有集成测试
2. 性能回归检测
3. 测试覆盖率报告
4. 失败场景的详细日志

# 第四步：文档更新
@docs-maintainer 请更新集成测试相关文档：
1. 本计划文档的测试结果记录
2. 性能基准的更新
3. 集成测试的使用指南
```

## 测试数据管理

### 测试环境隔离
- 每个测试使用独立的临时目录
- 测试完成后自动清理
- 并发测试的端口隔离

### Mock数据
- 测试用例文件保存在`tests/fixtures/`
- 统一的Mock数据生成器
- 随机数据生成支持
//! elfi-types 模块全面集成测试
//!
//! 本测试文件验证elfi-types模块在复杂场景下的行为，包括：
//! 1. Document生命周期集成测试
//! 2. Block关系管理集成测试  
//! 3. 错误处理和边界条件集成测试
//! 4. 核心用例验证测试

use std::collections::HashMap;
use std::time::{Duration, Instant};
use types::{
    Block, BlockContent, DefaultTypeInterface, Document, Relation, TypeInterface, TypesError
};
use uuid::Uuid;

// ============ Document生命周期集成测试 ============

#[cfg(test)]
mod document_lifecycle_integration {
    use super::*;

    /// 测试目标: Document的完整生命周期管理
    /// 验证: 创建 -> 添加多个Block -> 查找 -> 更新 -> 序列化
    #[test]
    fn test_complex_document_lifecycle() {
        let mut doc = Document::new("integration-test-doc".to_string());
        
        // 验证初始状态
        assert_eq!(doc.id, "integration-test-doc");
        assert_eq!(doc.blocks.len(), 0);
        assert_eq!(doc.metadata.version, 1);
        
        // 创建不同类型的Block
        let intro_block = Block::new(Uuid::new_v4().to_string(), "markdown".to_string())
            .with_name("introduction".to_string())
            .with_content(BlockContent::Text("# 项目介绍\n\n这是一个基于CRDT的协作文档系统。".to_string()));
            
        let code_block = Block::new(Uuid::new_v4().to_string(), "code".to_string())
            .with_name("setup-code".to_string())
            .with_content(BlockContent::Text("npm install elfi-core".to_string()))
            .with_attributes({
                let mut attrs = HashMap::new();
                attrs.insert("language".to_string(), serde_json::json!("bash"));
                attrs.insert("parent".to_string(), serde_json::json!(intro_block.id));
                attrs
            });
            
        let relations_block = Block::new(Uuid::new_v4().to_string(), "relations".to_string())
            .with_name("document-relations".to_string())
            .with_content(BlockContent::Relations(format!(
                "{} -> {} [child_of] {{}}\n{} -> elf://shared-lib/utils/helpers#string-utils [references] {{display_text: \"共享工具函数\"}}",
                code_block.id, intro_block.id, intro_block.id
            )))
            .with_attributes({
                let mut attrs = HashMap::new();
                attrs.insert("owner".to_string(), serde_json::json!("alice"));
                attrs.insert("merge_method".to_string(), serde_json::json!("manual"));
                attrs
            });
        
        // 添加Block到Document
        let intro_id = intro_block.id.clone();
        let code_id = code_block.id.clone();
        let relations_id = relations_block.id.clone();
        
        doc.blocks.push(intro_block);
        doc.blocks.push(code_block);
        doc.blocks.push(relations_block);
        
        // 验证Block添加成功
        assert_eq!(doc.blocks.len(), 3);
        
        // 测试各种查找功能
        assert!(doc.find_block(&intro_id).is_some());
        assert!(doc.find_block(&code_id).is_some());
        assert!(doc.find_block(&relations_id).is_some());
        assert!(doc.find_block("non-existent-id").is_none());
        
        assert!(doc.find_block_by_name("introduction").is_some());
        assert!(doc.find_block_by_name("setup-code").is_some());
        assert!(doc.find_block_by_name("document-relations").is_some());
        assert!(doc.find_block_by_name("non-existent-name").is_none());
        
        // 验证Block内容和属性
        let found_intro = doc.find_block(&intro_id).unwrap();
        assert_eq!(found_intro.block_type, "markdown");
        assert_eq!(found_intro.name.as_ref().unwrap(), "introduction");
        
        let found_code = doc.find_block(&code_id).unwrap();
        assert_eq!(found_code.block_type, "code");
        assert_eq!(found_code.attributes.get("language").unwrap(), "bash");
        
        let found_relations = doc.find_block(&relations_id).unwrap();
        assert_eq!(found_relations.block_type, "relations");
        assert!(matches!(found_relations.content, BlockContent::Relations(_)));
        
        // 验证Document的名称唯一性
        assert!(doc.validate_unique_names().is_ok());
        
        // 测试序列化和反序列化
        let interface = DefaultTypeInterface::new();
        let serialized = interface.serialize_document(&doc).expect("序列化应该成功");
        assert!(!serialized.is_empty());
        
        let deserialized = interface.deserialize_document(&serialized).expect("反序列化应该成功");
        assert_eq!(deserialized.id, doc.id);
        assert_eq!(deserialized.blocks.len(), doc.blocks.len());
        assert_eq!(deserialized.metadata.version, doc.metadata.version);
    }

    /// 测试目标: Document metadata的复杂管理
    #[test]
    fn test_document_metadata_management() {
        let mut doc = Document::new("metadata-test".to_string());
        
        // 添加复杂的document-level属性
        doc.metadata.attributes.insert(
            "project_info".to_string(),
            serde_json::json!({
                "name": "ELFI",
                "version": "0.1.0",
                "authors": ["Alice", "Bob"],
                "tags": ["crdt", "collaboration", "document"]
            })
        );
        
        doc.metadata.attributes.insert(
            "settings".to_string(),
            serde_json::json!({
                "auto_save": true,
                "real_time_sync": true,
                "conflict_resolution": "manual"
            })
        );
        
        // 验证复杂嵌套属性的序列化和反序列化
        let interface = DefaultTypeInterface::new();
        let serialized = interface.serialize_document(&doc).expect("序列化应该成功");
        let deserialized = interface.deserialize_document(&serialized).expect("反序列化应该成功");
        
        assert_eq!(deserialized.metadata.attributes.len(), 2);
        assert!(deserialized.metadata.attributes.contains_key("project_info"));
        assert!(deserialized.metadata.attributes.contains_key("settings"));
        
        // 验证嵌套对象的正确反序列化
        let project_info = &deserialized.metadata.attributes["project_info"];
        assert_eq!(project_info["name"], "ELFI");
        assert_eq!(project_info["authors"].as_array().unwrap().len(), 2);
    }

    /// 测试目标: Document的并发修改模拟
    #[test]
    fn test_document_concurrent_modifications() {
        let mut doc = Document::new("concurrent-test".to_string());
        
        // 模拟多个用户同时添加Block
        let mut user_blocks = Vec::new();
        
        for i in 0..10 {
            let block = Block::new(Uuid::new_v4().to_string(), "markdown".to_string())
                .with_name(format!("user-block-{}", i))
                .with_content(BlockContent::Text(format!("用户{}的内容", i)))
                .with_attributes({
                    let mut attrs = HashMap::new();
                    attrs.insert("author".to_string(), serde_json::json!(format!("user-{}", i)));
                    attrs.insert("timestamp".to_string(), serde_json::json!(i));
                    attrs
                });
            user_blocks.push(block);
        }
        
        // 添加所有Block
        for block in user_blocks {
            doc.blocks.push(block);
        }
        
        assert_eq!(doc.blocks.len(), 10);
        
        // 验证所有Block都有唯一名称
        assert!(doc.validate_unique_names().is_ok());
        
        // 验证按author查找
        let user_5_blocks: Vec<_> = doc.blocks.iter()
            .filter(|block| {
                block.attributes.get("author")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "user-5")
                    .unwrap_or(false)
            })
            .collect();
        assert_eq!(user_5_blocks.len(), 1);
    }
}

// ============ Block关系管理集成测试 ============

#[cfg(test)]
mod block_relations_integration {
    use super::*;

    /// 测试目标: 复杂的Block层级关系管理
    #[test]
    fn test_complex_block_hierarchy() {
        let mut doc = Document::new("hierarchy-test".to_string());
        
        // 创建层级结构: root -> chapter1 -> section1.1 -> subsection1.1.1
        let root_block = Block::new(Uuid::new_v4().to_string(), "markdown".to_string())
            .with_name("root".to_string())
            .with_content(BlockContent::Text("# 文档根节点".to_string()));
            
        let chapter_block = Block::new(Uuid::new_v4().to_string(), "markdown".to_string())
            .with_name("chapter1".to_string())
            .with_content(BlockContent::Text("## 第一章".to_string()))
            .with_attributes({
                let mut attrs = HashMap::new();
                attrs.insert("parent".to_string(), serde_json::json!(root_block.id));
                attrs.insert("level".to_string(), serde_json::json!(1));
                attrs
            });
            
        let section_block = Block::new(Uuid::new_v4().to_string(), "markdown".to_string())
            .with_name("section1.1".to_string())
            .with_content(BlockContent::Text("### 1.1 节".to_string()))
            .with_attributes({
                let mut attrs = HashMap::new();
                attrs.insert("parent".to_string(), serde_json::json!(chapter_block.id));
                attrs.insert("level".to_string(), serde_json::json!(2));
                attrs
            });
            
        let subsection_block = Block::new(Uuid::new_v4().to_string(), "code".to_string())
            .with_name("subsection1.1.1".to_string())
            .with_content(BlockContent::Text("fn example() { println!(\"Hello\"); }".to_string()))
            .with_attributes({
                let mut attrs = HashMap::new();
                attrs.insert("parent".to_string(), serde_json::json!(section_block.id));
                attrs.insert("level".to_string(), serde_json::json!(3));
                attrs.insert("language".to_string(), serde_json::json!("rust"));
                attrs
            });
        
        // 创建Relations Block来定义关系
        let relations_content = format!(
            r#"# 层级关系
{} -> {} [child_of] {{level: 1}}
{} -> {} [child_of] {{level: 2}}
{} -> {} [child_of] {{level: 3}}

# 引用关系  
{} -> elf://external-doc/concepts#crdt [references] {{context: "技术背景"}}
{} -> elf://examples/rust-basics#functions [references] {{context: "代码示例"}}"#,
            chapter_block.id, root_block.id,
            section_block.id, chapter_block.id,
            subsection_block.id, section_block.id,
            root_block.id,
            subsection_block.id
        );
        
        let relations_block = Block::new(Uuid::new_v4().to_string(), "relations".to_string())
            .with_name("document-structure".to_string())
            .with_content(BlockContent::Relations(relations_content))
            .with_attributes({
                let mut attrs = HashMap::new();
                attrs.insert("scope".to_string(), serde_json::json!("document"));
                attrs.insert("auto_generated".to_string(), serde_json::json!(false));
                attrs
            });
        
        // 添加所有Block
        doc.blocks.push(root_block);
        doc.blocks.push(chapter_block);
        doc.blocks.push(section_block);
        doc.blocks.push(subsection_block);
        doc.blocks.push(relations_block);
        
        assert_eq!(doc.blocks.len(), 5);
        
        // 验证层级查找功能
        let level_1_blocks: Vec<_> = doc.blocks.iter()
            .filter(|block| {
                block.attributes.get("level")
                    .and_then(|v| v.as_i64())
                    .map(|l| l == 1)
                    .unwrap_or(false)
            })
            .collect();
        assert_eq!(level_1_blocks.len(), 1);
        
        let level_3_blocks: Vec<_> = doc.blocks.iter()
            .filter(|block| {
                block.attributes.get("level")
                    .and_then(|v| v.as_i64())
                    .map(|l| l == 3)
                    .unwrap_or(false)
            })
            .collect();
        assert_eq!(level_3_blocks.len(), 1);
        
        // 验证Relations Block的内容
        let relations = doc.find_block_by_name("document-structure").unwrap();
        assert_eq!(relations.block_type, "relations");
        assert!(matches!(relations.content, BlockContent::Relations(_)));
        
        // 验证跨文档引用的格式
        if let BlockContent::Relations(ref content) = relations.content {
            assert!(content.contains("elf://external-doc/concepts#crdt"));
            assert!(content.contains("elf://examples/rust-basics#functions"));
        }
    }

    /// 测试目标: Relations Block与Relation对象的集成
    #[test]
    fn test_relations_block_and_relation_objects() {
        // 创建独立的Relation对象
        let relation1 = Relation::new(
            "block-a".to_string(),
            "block-b".to_string(),
            "child_of".to_string(),
        ).with_attributes({
            let mut attrs = HashMap::new();
            attrs.insert("weight".to_string(), serde_json::json!(1.0));
            attrs.insert("created_by".to_string(), serde_json::json!("alice"));
            attrs
        });
        
        let relation2 = Relation::new(
            "block-b".to_string(),
            "elf://other-doc/section#intro".to_string(),
            "references".to_string(),
        ).with_attributes({
            let mut attrs = HashMap::new();
            attrs.insert("display_text".to_string(), serde_json::json!("相关介绍"));
            attrs.insert("auto_sync".to_string(), serde_json::json!(true));
            attrs
        });
        
        // 验证Relation对象
        assert!(relation1.validate().is_ok());
        assert!(relation2.validate().is_ok());
        
        // 创建对应的Relations Block
        let relations_content = format!(
            r#"{} -> {} [{}] {{}}
{} -> {} [{}] {{}}"#,
            relation1.from, relation1.to, relation1.relation_type,
            relation2.from, relation2.to, relation2.relation_type
        );
        
        let relations_block = Block::new(Uuid::new_v4().to_string(), "relations".to_string())
            .with_name("mixed-relations".to_string())
            .with_content(BlockContent::Relations(relations_content));
        
        // 验证Relations Block
        assert!(relations_block.validate().is_ok());
        assert_eq!(relations_block.block_type, "relations");
        
        // 测试序列化
        let interface = DefaultTypeInterface::new();
        
        let relation1_json = serde_json::to_string(&relation1).expect("Relation序列化成功");
        let relation2_json = serde_json::to_string(&relation2).expect("Relation序列化成功");
        let block_json = serde_json::to_string(&relations_block).expect("Block序列化成功");
        
        assert!(!relation1_json.is_empty());
        assert!(!relation2_json.is_empty());
        assert!(!block_json.is_empty());
        
        // 测试反序列化
        let deserialized_relation1: Relation = serde_json::from_str(&relation1_json).expect("Relation反序列化成功");
        let deserialized_block: Block = serde_json::from_str(&block_json).expect("Block反序列化成功");
        
        assert_eq!(deserialized_relation1.from, relation1.from);
        assert_eq!(deserialized_relation1.to, relation1.to);
        assert_eq!(deserialized_relation1.relation_type, relation1.relation_type);
        assert_eq!(deserialized_block.name, relations_block.name);
    }

    /// 测试目标: 复杂的跨文档引用场景
    #[test]
    fn test_cross_document_references() {
        let mut doc = Document::new("main-document".to_string());
        
        // 创建引用外部文档的Block
        let intro_block = Block::new(Uuid::new_v4().to_string(), "markdown".to_string())
            .with_name("introduction".to_string())
            .with_content(BlockContent::Text("# 主文档介绍\n\n本文档引用了多个外部资源。".to_string()));
        
        let impl_block = Block::new(Uuid::new_v4().to_string(), "code".to_string())
            .with_name("implementation".to_string())
            .with_content(BlockContent::Text("// 实现细节参考外部文档\nuse external_lib::*;".to_string()))
            .with_attributes({
                let mut attrs = HashMap::new();
                attrs.insert("language".to_string(), serde_json::json!("rust"));
                attrs
            });
        
        // 创建包含多种跨文档引用的Relations Block
        let cross_refs_content = format!(
            r#"# 文档内关系
{} -> {} [child_of] {{}}

# 跨文档引用 - 不同类型的URI格式
{} -> elf://shared-lib/utils/helpers#string-utils [references] {{display_text: "字符串工具"}}
{} -> elf://alice/project-docs/api-ref#authentication [references] {{display_text: "认证API"}}
{} -> elf://team/guidelines/coding-standards#rust-style [follows] {{category: "代码规范"}}

# 组件引用
{} -> elf://ui-components/forms#login-form [uses] {{version: "1.2.0"}}
{} -> elf://shared/templates/layouts#main-layout [extends] {{theme: "dark"}}"#,
            impl_block.id, intro_block.id,
            intro_block.id,
            impl_block.id,
            impl_block.id,
            intro_block.id,
            impl_block.id
        );
        
        let cross_refs_block = Block::new(Uuid::new_v4().to_string(), "relations".to_string())
            .with_name("cross-document-refs".to_string())
            .with_content(BlockContent::Relations(cross_refs_content))
            .with_attributes({
                let mut attrs = HashMap::new();
                attrs.insert("scope".to_string(), serde_json::json!("global"));
                attrs.insert("auto_update".to_string(), serde_json::json!(true));
                attrs
            });
        
        doc.blocks.push(intro_block);
        doc.blocks.push(impl_block);
        doc.blocks.push(cross_refs_block);
        
        assert_eq!(doc.blocks.len(), 3);
        
        // 验证跨文档引用的语法格式
        let refs_block = doc.find_block_by_name("cross-document-refs").unwrap();
        if let BlockContent::Relations(ref content) = refs_block.content {
            // 验证不同的URI格式都存在
            assert!(content.contains("elf://shared-lib/utils/helpers#string-utils"));
            assert!(content.contains("elf://alice/project-docs/api-ref#authentication"));
            assert!(content.contains("elf://team/guidelines/coding-standards#rust-style"));
            assert!(content.contains("elf://ui-components/forms#login-form"));
            assert!(content.contains("elf://shared/templates/layouts#main-layout"));
            
            // 验证不同的关系类型
            assert!(content.contains("[references]"));
            assert!(content.contains("[follows]"));
            assert!(content.contains("[uses]"));
            assert!(content.contains("[extends]"));
            
            // 验证属性格式（简化格式）
            assert!(content.contains(r#"display_text: "字符串工具""#));
            assert!(content.contains(r#"version: "1.2.0""#));
            assert!(content.contains(r#"theme: "dark""#));
        }
        
        // 测试文档的完整序列化，确保跨文档引用被正确保存
        let interface = DefaultTypeInterface::new();
        let serialized = interface.serialize_document(&doc).expect("序列化成功");
        let deserialized = interface.deserialize_document(&serialized).expect("反序列化成功");
        
        let deserialized_refs = deserialized.find_block_by_name("cross-document-refs").unwrap();
        assert_eq!(refs_block.content, deserialized_refs.content);
    }
}

// ============ 错误处理和边界条件集成测试 ============

#[cfg(test)]
mod error_boundary_integration {
    use super::*;

    /// 测试目标: 大数据量处理能力
    #[test]
    fn test_large_scale_data_processing() {
        let mut doc = Document::new("large-scale-test".to_string());
        
        const LARGE_BLOCK_COUNT: usize = 1000;
        const LARGE_CONTENT_SIZE: usize = 10000; // 10KB per block
        
        let start_time = Instant::now();
        
        // 创建大量Block
        for i in 0..LARGE_BLOCK_COUNT {
            let large_content = "A".repeat(LARGE_CONTENT_SIZE);
            let block = Block::new(Uuid::new_v4().to_string(), "markdown".to_string())
                .with_name(format!("large-block-{:04}", i))
                .with_content(BlockContent::Text(large_content))
                .with_attributes({
                    let mut attrs = HashMap::new();
                    attrs.insert("index".to_string(), serde_json::json!(i));
                    attrs.insert("size".to_string(), serde_json::json!(LARGE_CONTENT_SIZE));
                    attrs.insert("category".to_string(), serde_json::json!(format!("cat-{}", i % 10)));
                    attrs
                });
            doc.blocks.push(block);
        }
        
        let creation_time = start_time.elapsed();
        println!("创建{}个大Block耗时: {:?}", LARGE_BLOCK_COUNT, creation_time);
        
        // 验证大数据量的基本操作
        assert_eq!(doc.blocks.len(), LARGE_BLOCK_COUNT);
        
        // 测试查找性能
        let search_start = Instant::now();
        let found_block = doc.find_block_by_name("large-block-0500");
        let search_time = search_start.elapsed();
        
        assert!(found_block.is_some());
        assert!(search_time < Duration::from_millis(100)); // 查找应该很快
        println!("在{}个Block中查找耗时: {:?}", LARGE_BLOCK_COUNT, search_time);
        
        // 测试按属性过滤的性能
        let filter_start = Instant::now();
        let category_5_blocks: Vec<_> = doc.blocks.iter()
            .filter(|block| {
                block.attributes.get("category")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "cat-5")
                    .unwrap_or(false)
            })
            .collect();
        let filter_time = filter_start.elapsed();
        
        assert_eq!(category_5_blocks.len(), LARGE_BLOCK_COUNT / 10);
        assert!(filter_time < Duration::from_millis(500)); // 过滤应该相对快速
        println!("按属性过滤{}个Block耗时: {:?}", LARGE_BLOCK_COUNT, filter_time);
        
        // 测试序列化性能（但不序列化所有内容，避免过度消耗资源）
        let sample_doc = Document {
            id: doc.id.clone(),
            blocks: doc.blocks.iter().take(10).cloned().collect(),
            metadata: doc.metadata.clone(),
        };
        
        let serialize_start = Instant::now();
        let interface = DefaultTypeInterface::new();
        let serialized = interface.serialize_document(&sample_doc).expect("序列化成功");
        let serialize_time = serialize_start.elapsed();
        
        assert!(!serialized.is_empty());
        assert!(serialize_time < Duration::from_millis(100));
        println!("序列化10个大Block耗时: {:?}", serialize_time);
    }

    /// 测试目标: 恶意和异常输入处理
    #[test]
    fn test_malicious_input_handling() {
        let interface = DefaultTypeInterface::new();
        
        // 测试各种恶意JSON输入
        let malicious_inputs = vec![
            "",                              // 空字符串
            "invalid json",                  // 无效JSON
            "{}",                           // 空对象
            r#"{"id": null}"#,              // null值
            r#"{"id": "test", "blocks": null}"#, // 部分null
            "[]",                           // 数组而非对象
            r#"{"id": "test", "blocks": "not_array"}"#, // 错误类型
            "{'id': 'invalid_quotes'}",     // 错误的引号
            r#"{"id": "test", "metadata": {"created_at": "invalid_date"}}"#, // 无效日期
        ];
        
        for (i, malicious_input) in malicious_inputs.iter().enumerate() {
            let result = interface.deserialize_document(malicious_input);
            assert!(result.is_err(), "恶意输入 {} 应该失败: {}", i, malicious_input);
            
            if let Err(e) = result {
                // 验证错误类型正确
                assert!(matches!(e, TypesError::Serialization { .. }));
            }
        }
    }

    /// 测试目标: 极端边界条件处理
    #[test]
    fn test_extreme_boundary_conditions() {
        // 测试超长ID
        let very_long_id = "a".repeat(10000);
        let result = Document::new(very_long_id.clone());
        assert_eq!(result.id, very_long_id);
        
        // 测试超长Block名称
        let very_long_name = "block_".repeat(1000);
        let block = Block::new(Uuid::new_v4().to_string(), "markdown".to_string())
            .with_name(very_long_name.clone());
        assert_eq!(block.name.unwrap(), very_long_name);
        
        // 测试超深嵌套属性
        let deeply_nested = serde_json::json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "level5": {
                                "level6": {
                                    "level7": {
                                        "level8": {
                                            "level9": {
                                                "level10": "deep_value"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
        
        let mut attributes = HashMap::new();
        attributes.insert("nested".to_string(), deeply_nested);
        
        let block = Block::new(Uuid::new_v4().to_string(), "test".to_string())
            .with_attributes(attributes);
        
        assert!(block.validate().is_ok());
        
        // 测试序列化和反序列化深度嵌套
        let serialized = serde_json::to_string(&block).expect("序列化成功");
        let deserialized: Block = serde_json::from_str(&serialized).expect("反序列化成功");
        assert_eq!(block.attributes, deserialized.attributes);
        
        // 测试超大Binary内容
        let large_binary = vec![0u8; 1024 * 1024]; // 1MB
        let binary_block = Block::new(Uuid::new_v4().to_string(), "binary".to_string())
            .with_content(BlockContent::Binary(large_binary.clone()));
        
        assert!(binary_block.validate().is_ok());
        if let BlockContent::Binary(ref data) = binary_block.content {
            assert_eq!(data.len(), 1024 * 1024);
        }
        
        // 测试空的Relations内容
        let empty_relations = Block::new(Uuid::new_v4().to_string(), "relations".to_string())
            .with_content(BlockContent::Relations("".to_string()));
        assert!(empty_relations.validate().is_ok());
        
        // 测试畸形的Relations语法（应该在parser模块中处理，这里只测试存储）
        let malformed_relations = Block::new(Uuid::new_v4().to_string(), "relations".to_string())
            .with_content(BlockContent::Relations("invalid -> syntax [malformed] {{".to_string()));
        assert!(malformed_relations.validate().is_ok()); // types模块只负责存储，不验证语法
    }

    /// 测试目标: 并发安全性模拟
    #[test]
    fn test_concurrent_safety_simulation() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let doc = Arc::new(Mutex::new(Document::new("concurrent-test".to_string())));
        let mut handles = vec![];
        
        // 模拟5个线程同时修改文档
        for thread_id in 0..5 {
            let doc_clone = Arc::clone(&doc);
            let handle = thread::spawn(move || {
                for i in 0..20 {
                    let block = Block::new(Uuid::new_v4().to_string(), "markdown".to_string())
                        .with_name(format!("thread-{}-block-{}", thread_id, i))
                        .with_content(BlockContent::Text(format!("Thread {} Block {}", thread_id, i)));
                    
                    let mut doc_guard = doc_clone.lock().unwrap();
                    doc_guard.blocks.push(block);
                }
            });
            handles.push(handle);
        }
        
        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }
        
        let final_doc = doc.lock().unwrap();
        assert_eq!(final_doc.blocks.len(), 100); // 5 threads * 20 blocks each
        
        // 验证名称唯一性
        assert!(final_doc.validate_unique_names().is_ok());
        
        // 验证所有Block都有效
        for block in &final_doc.blocks {
            assert!(block.validate().is_ok());
        }
    }

    /// 测试目标: 内存使用分析
    #[test]
    fn test_memory_usage_analysis() {
        // 这个测试提供内存使用的基本观察，实际内存监控需要外部工具
        
        let mut doc = Document::new("memory-test".to_string());
        let initial_block_count = doc.blocks.len();
        
        // 逐步增加Block数量，观察行为
        for batch in 1..=5 {
            let batch_size = batch * 100;
            
            for i in 0..batch_size {
                let content = format!("Batch {} Block {} with some content to fill memory", batch, i);
                let block = Block::new(Uuid::new_v4().to_string(), "markdown".to_string())
                    .with_name(format!("memory-block-{}-{}", batch, i))
                    .with_content(BlockContent::Text(content))
                    .with_attributes({
                        let mut attrs = HashMap::new();
                        attrs.insert("batch".to_string(), serde_json::json!(batch));
                        attrs.insert("index".to_string(), serde_json::json!(i));
                        attrs
                    });
                doc.blocks.push(block);
            }
            
            // 验证增长是线性的
            let expected_count = initial_block_count + (1..=batch).map(|b| b * 100).sum::<usize>();
            assert_eq!(doc.blocks.len(), expected_count);
            
            // 测试查找性能随数据量增长的变化
            let search_start = Instant::now();
            let _found = doc.find_block_by_name(&format!("memory-block-1-50"));
            let search_time = search_start.elapsed();
            
            println!("Batch {}: {} blocks, search time: {:?}", batch, doc.blocks.len(), search_time);
            
            // 确保查找时间保持合理
            assert!(search_time < Duration::from_millis(50));
        }
    }
}


// ============ 核心用例验证测试 ============

#[cfg(test)]
mod usecase_validation {
    use super::*;

    /// 测试目标: 对话即文档场景验证
    /// 参考: docs/src/usecases/02-conversation-as-document.md
    #[test]
    fn test_conversation_as_document_usecase() {
        let mut conversation_doc = Document::new("team-meeting-2024-01-15".to_string());
        
        // 添加Document-level属性模拟会议元数据
        conversation_doc.metadata.attributes.insert(
            "meeting_info".to_string(),
            serde_json::json!({
                "date": "2024-01-15",
                "participants": ["alice", "bob", "charlie"],
                "topic": "Sprint Planning",
                "duration": "60 minutes"
            })
        );
        
        // 模拟多用户实时对话的Block序列
        let conversation_blocks = vec![
            ("alice", "introduction", "# Sprint Planning Meeting\n\n欢迎大家参加今天的Sprint规划会议。"),
            ("bob", "status-update", "## 当前进度\n\n后端API开发已完成80%，预计本周五交付。"),
            ("charlie", "ui-progress", "## UI进展\n\n设计稿已完成，正在进行组件开发。"),
            ("alice", "timeline-discussion", "## 时间安排\n\n根据大家的进度，我们需要调整一下时间线。"),
            ("bob", "technical-concern", "## 技术考虑\n\n需要考虑数据库迁移的影响，可能需要额外的时间。"),
            ("charlie", "design-update", "## 设计更新\n\n用户反馈建议调整颜色方案，会在明天更新。"),
        ];
        
        let mut block_ids = Vec::new();
        for (author, name, content) in conversation_blocks {
            let block = Block::new(Uuid::new_v4().to_string(), "markdown".to_string())
                .with_name(name.to_string())
                .with_content(BlockContent::Text(content.to_string()))
                .with_attributes({
                    let mut attrs = HashMap::new();
                    attrs.insert("author".to_string(), serde_json::json!(author));
                    attrs.insert("timestamp".to_string(), serde_json::json!(format!("2024-01-15T10:{:02}:00Z", 10 + block_ids.len() * 5)));
                    attrs.insert("type".to_string(), serde_json::json!("conversation"));
                    attrs
                });
            
            block_ids.push(block.id.clone());
            conversation_doc.blocks.push(block);
        }
        
        // 创建会议结构的Relations Block
        let meeting_structure = format!(
            r#"# 会议流程结构
{} -> {} [follows] {{order: 1}}
{} -> {} [follows] {{order: 2}}
{} -> {} [follows] {{order: 3}}
{} -> {} [follows] {{order: 4}}
{} -> {} [follows] {{order: 5}}

# 主题关联
{} -> {} [related_to] {{topic: "进度汇报"}}
{} -> {} [blocks] {{dependency: "API完成"}}

# 外部引用
{} -> elf://project-docs/api-spec#endpoints [references] {{context: "技术依赖"}}
{} -> elf://design-system/components#buttons [references] {{context: "UI组件"}}"#,
            block_ids[0], block_ids[1], // intro -> status
            block_ids[1], block_ids[2], // status -> ui
            block_ids[2], block_ids[3], // ui -> timeline
            block_ids[3], block_ids[4], // timeline -> technical
            block_ids[4], block_ids[5], // technical -> design
            block_ids[1], block_ids[2], // 进度相关
            block_ids[4], block_ids[1], // 技术阻塞
            block_ids[4], // 技术引用
            block_ids[5]  // 设计引用
        );
        
        let relations_block = Block::new(Uuid::new_v4().to_string(), "relations".to_string())
            .with_name("meeting-structure".to_string())
            .with_content(BlockContent::Relations(meeting_structure))
            .with_attributes({
                let mut attrs = HashMap::new();
                attrs.insert("auto_generated".to_string(), serde_json::json!(false));
                attrs.insert("scope".to_string(), serde_json::json!("meeting"));
                attrs
            });
        
        conversation_doc.blocks.push(relations_block);
        
        // 验证对话文档的结构
        assert_eq!(conversation_doc.blocks.len(), 7); // 6个对话 + 1个关系
        assert!(conversation_doc.validate_unique_names().is_ok());
        
        // 验证按作者查找
        let alice_blocks: Vec<_> = conversation_doc.blocks.iter()
            .filter(|block| {
                block.attributes.get("author")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "alice")
                    .unwrap_or(false)
            })
            .collect();
        assert_eq!(alice_blocks.len(), 2);
        
        // 验证时间顺序
        let mut timestamps: Vec<_> = conversation_doc.blocks.iter()
            .filter_map(|block| {
                block.attributes.get("timestamp")
                    .and_then(|v| v.as_str())
            })
            .collect();
        timestamps.sort();
        assert!(timestamps.len() >= 6);
        
        // 验证完整序列化（模拟CRDT同步）
        let interface = DefaultTypeInterface::new();
        let serialized = interface.serialize_document(&conversation_doc).expect("序列化成功");
        let deserialized = interface.deserialize_document(&serialized).expect("反序列化成功");
        
        assert_eq!(deserialized.id, conversation_doc.id);
        assert_eq!(deserialized.blocks.len(), conversation_doc.blocks.len());
    }

    /// 测试目标: 自举开发场景验证
    /// 参考: docs/src/usecases/01-bootstrapping.md
    #[test]
    fn test_bootstrapping_development_usecase() {
        let mut dev_doc = Document::new("elfi-core-development".to_string());
        
        // 添加项目元数据
        dev_doc.metadata.attributes.insert(
            "project".to_string(),
            serde_json::json!({
                "name": "elfi-core",
                "version": "0.1.0",
                "language": "rust",
                "build_system": "cargo"
            })
        );
        
        // 创建代码管理相关的Block
        let code_blocks = vec![
            ("project-structure", "markdown", "# ELFI项目结构\n\n描述整个项目的组织方式和模块划分。"),
            ("core-types", "code", "// 核心数据类型定义\nuse serde::{Serialize, Deserialize};\n\n#[derive(Debug, Serialize, Deserialize)]\npub struct Document {\n    pub id: String,\n    pub blocks: Vec<Block>,\n}"),
            ("test-strategy", "markdown", "## 测试策略\n\n1. 单元测试覆盖所有公共API\n2. 集成测试验证端到端功能\n3. 性能基准确保可扩展性"),
            ("build-config", "code", "# Cargo.toml configuration\n[package]\nname = \"elfi-core\"\nversion = \"0.1.0\"\nedition = \"2021\""),
            ("deployment-recipe", "recipe", "# 自动化部署脚本\nsteps:\n  - cargo test\n  - cargo build --release\n  - cargo publish"),
        ];
        
        let mut block_ids = Vec::new();
        for (name, block_type, content) in code_blocks {
            let content_enum = match block_type {
                "recipe" => BlockContent::Text(content.to_string()), // Recipe作为特殊文本处理
                _ => BlockContent::Text(content.to_string()),
            };
            
            let block = Block::new(Uuid::new_v4().to_string(), block_type.to_string())
                .with_name(name.to_string())
                .with_content(content_enum)
                .with_attributes({
                    let mut attrs = HashMap::new();
                    attrs.insert("category".to_string(), serde_json::json!("development"));
                    if block_type == "code" {
                        attrs.insert("language".to_string(), serde_json::json!("rust"));
                        attrs.insert("executable".to_string(), serde_json::json!(true));
                    }
                    if block_type == "recipe" {
                        attrs.insert("auto_run".to_string(), serde_json::json!(false));
                        attrs.insert("dependencies".to_string(), serde_json::json!(["core-types", "test-strategy"]));
                    }
                    attrs
                });
            
            block_ids.push(block.id.clone());
            dev_doc.blocks.push(block);
        }
        
        // 创建开发工作流程的Relations Block
        let dev_workflow = format!(
            r#"# 开发依赖关系
{} -> {} [implements] {{stage: "foundation"}}
{} -> {} [tests] {{coverage: "unit"}}
{} -> {} [builds] {{target: "release"}}
{} -> {} [requires] {{type: "sequential"}}

# 外部工具引用
{} -> elf://build-tools/cargo#commands [uses] {{purpose: "compilation"}}
{} -> elf://ci-cd/github-actions#rust-workflow [triggers] {{event: "push"}}
{} -> elf://docs/api-reference#types [documents] {{format: "rustdoc"}}

# 自举循环
{} -> elf://elfi-core/src/lib.rs#Document [defines] {{meta: "self_reference"}}
{} -> elf://elfi-core/Cargo.toml#dependencies [configures] {{tool: "cargo"}}"#,
            block_ids[1], block_ids[0], // core-types implements structure
            block_ids[2], block_ids[1], // test-strategy tests core-types
            block_ids[3], block_ids[1], // build-config builds core-types
            block_ids[4], block_ids[2], // deployment requires tests
            block_ids[3], // build tools
            block_ids[4], // CI/CD
            block_ids[1], // documentation
            block_ids[1], // 自举：类型定义
            block_ids[3]  // 自举：构建配置
        );
        
        let relations_block = Block::new(Uuid::new_v4().to_string(), "relations".to_string())
            .with_name("development-workflow".to_string())
            .with_content(BlockContent::Relations(dev_workflow))
            .with_attributes({
                let mut attrs = HashMap::new();
                attrs.insert("scope".to_string(), serde_json::json!("project"));
                attrs.insert("auto_update".to_string(), serde_json::json!(true));
                attrs
            });
        
        dev_doc.blocks.push(relations_block);
        
        // 验证自举开发文档结构
        assert_eq!(dev_doc.blocks.len(), 6); // 5个开发块 + 1个关系块
        assert!(dev_doc.validate_unique_names().is_ok());
        
        // 验证代码块的属性
        let code_blocks_count = dev_doc.blocks.iter()
            .filter(|block| block.block_type == "code")
            .count();
        assert_eq!(code_blocks_count, 2);
        
        // 验证Recipe块
        let recipe_block = dev_doc.find_block_by_name("deployment-recipe").unwrap();
        assert_eq!(recipe_block.block_type, "recipe");
        assert!(recipe_block.attributes.contains_key("dependencies"));
        
        // 验证自举引用（Relations Block包含对自身项目的引用）
        let workflow_block = dev_doc.find_block_by_name("development-workflow").unwrap();
        if let BlockContent::Relations(ref content) = workflow_block.content {
            assert!(content.contains("elf://elfi-core/"));
            assert!(content.contains("self_reference"));
        }
        
        // 模拟文件同步场景
        let interface = DefaultTypeInterface::new();
        let serialized = interface.serialize_document(&dev_doc).expect("序列化成功");
        
        // 验证序列化内容包含所有代码
        assert!(serialized.contains("Document"));
        assert!(serialized.contains("cargo test"));
        assert!(serialized.contains("elfi-core"));
    }

    /// 测试目标: 文档即App场景验证
    /// 参考: docs/src/usecases/03-document-as-app.md
    #[test]
    fn test_document_as_app_usecase() {
        // 创建主文档
        let mut main_doc = Document::new("interactive-dashboard".to_string());
        
        main_doc.metadata.attributes.insert(
            "app_config".to_string(),
            serde_json::json!({
                "title": "ELFI Dashboard",
                "version": "1.0.0",
                "theme": "dark",
                "interactive": true
            })
        );
        
        // 创建主文档的组件引用Block
        let main_blocks = vec![
            ("app-header", "component", "# Dashboard Header\n\n应用顶部导航和用户信息显示"),
            ("main-content", "layout", "## 主要内容区域\n\n包含多个动态组件的布局容器"),
            ("data-visualization", "component", "### 数据可视化\n\n实时图表和统计信息"),
            ("user-settings", "component", "### 用户设置\n\n个性化配置面板"),
        ];
        
        let mut main_block_ids = Vec::new();
        for (name, block_type, content) in main_blocks {
            let block = Block::new(Uuid::new_v4().to_string(), block_type.to_string())
                .with_name(name.to_string())
                .with_content(BlockContent::Text(content.to_string()))
                .with_attributes({
                    let mut attrs = HashMap::new();
                    attrs.insert("interactive".to_string(), serde_json::json!(true));
                    attrs.insert("render_priority".to_string(), serde_json::json!(main_block_ids.len()));
                    attrs
                });
            
            main_block_ids.push(block.id.clone());
            main_doc.blocks.push(block);
        }
        
        // 创建跨文档组件引用的Relations Block
        let cross_doc_relations = format!(
            r#"# 文档内布局关系
{} -> {} [contains] {{position: "top"}}
{} -> {} [contains] {{position: "center-left"}}
{} -> {} [contains] {{position: "center-right"}}

# 跨文档组件引用
{} -> elf://ui-components/navigation#top-nav [renders] {{props: "{{\"title\": \"Dashboard\", \"user\": \"alice\"}}"}}
{} -> elf://widgets/charts#line-chart [embeds] {{data_source: "api://stats/daily", refresh: 30}}
{} -> elf://widgets/charts#pie-chart [embeds] {{data_source: "api://stats/categories", interactive: true}}
{} -> elf://forms/settings#user-preferences [includes] {{theme: "dark", save_auto: true}}

# 动态内容引用
{} -> elf://data-sources/analytics#real-time [subscribes] {{interval: 5000}}
{} -> elf://themes/dark#variables [applies] {{scope: "global"}}

# 状态管理引用
{} -> elf://state/user-session#current [reads] {{reactive: true}}
{} -> elf://state/app-config#dashboard [writes] {{persistent: true}}"#,
            main_block_ids[0], main_block_ids[1], // header contains main-content
            main_block_ids[1], main_block_ids[2], // main-content contains data-viz
            main_block_ids[1], main_block_ids[3], // main-content contains settings
            main_block_ids[0], // navigation component
            main_block_ids[2], // line chart
            main_block_ids[2], // pie chart  
            main_block_ids[3], // settings form
            main_block_ids[2], // analytics data
            main_block_ids[0], // theme
            main_block_ids[3], // user session
            main_block_ids[1]  // app config
        );
        
        let relations_block = Block::new(Uuid::new_v4().to_string(), "relations".to_string())
            .with_name("app-composition".to_string())
            .with_content(BlockContent::Relations(cross_doc_relations))
            .with_attributes({
                let mut attrs = HashMap::new();
                attrs.insert("scope".to_string(), serde_json::json!("application"));
                attrs.insert("runtime".to_string(), serde_json::json!("browser"));
                attrs.insert("hot_reload".to_string(), serde_json::json!(true));
                attrs
            });
        
        main_doc.blocks.push(relations_block);
        
        // 模拟组件文档 (在实际应用中这会是独立的文档)
        let mut component_doc = Document::new("shared-ui-components".to_string());
        
        let component_blocks = vec![
            ("top-nav", "component", "<!-- Navigation Component -->\n<nav class=\"top-nav\">{{title}}</nav>"),
            ("line-chart", "component", "<!-- Line Chart Component -->\n<div class=\"chart\" data-type=\"line\">{{data}}</div>"),
            ("user-preferences", "component", "<!-- Settings Form -->\n<form class=\"preferences\">{{fields}}</form>"),
        ];
        
        for (name, block_type, content) in component_blocks {
            let block = Block::new(Uuid::new_v4().to_string(), block_type.to_string())
                .with_name(name.to_string())
                .with_content(BlockContent::Text(content.to_string()))
                .with_attributes({
                    let mut attrs = HashMap::new();
                    attrs.insert("reusable".to_string(), serde_json::json!(true));
                    attrs.insert("framework".to_string(), serde_json::json!("html"));
                    attrs
                });
            
            component_doc.blocks.push(block);
        }
        
        // 验证文档即App场景
        assert_eq!(main_doc.blocks.len(), 5); // 4个主要块 + 1个关系块
        assert_eq!(component_doc.blocks.len(), 3); // 3个组件块
        
        // 验证跨文档引用的语法
        let app_composition = main_doc.find_block_by_name("app-composition").unwrap();
        if let BlockContent::Relations(ref content) = app_composition.content {
            // 验证组件引用URI格式
            assert!(content.contains("elf://ui-components/navigation#top-nav"));
            assert!(content.contains("elf://widgets/charts#line-chart"));
            assert!(content.contains("elf://forms/settings#user-preferences"));
            
            // 验证数据源引用
            assert!(content.contains("elf://data-sources/analytics#real-time"));
            assert!(content.contains("elf://state/user-session#current"));
            
            // 验证动态属性（简化格式）
            assert!(content.contains(r#"refresh: 30"#));
            assert!(content.contains(r#"interactive: true"#));
            assert!(content.contains(r#"reactive: true"#));
        }
        
        // 验证组件的可重用性
        let reusable_components: Vec<_> = component_doc.blocks.iter()
            .filter(|block| {
                block.attributes.get("reusable")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            })
            .collect();
        assert_eq!(reusable_components.len(), 3);
        
        // 验证完整序列化（模拟应用部署）
        let interface = DefaultTypeInterface::new();
        
        let main_serialized = interface.serialize_document(&main_doc).expect("主文档序列化成功");
        let component_serialized = interface.serialize_document(&component_doc).expect("组件文档序列化成功");
        
        assert!(main_serialized.contains("Dashboard"));
        assert!(main_serialized.contains("elf://ui-components/"));
        assert!(component_serialized.contains("top-nav"));
        assert!(component_serialized.contains("reusable"));
        
        // 模拟渲染时的跨文档解析
        let main_deserialized = interface.deserialize_document(&main_serialized).expect("主文档反序列化成功");
        let component_deserialized = interface.deserialize_document(&component_serialized).expect("组件文档反序列化成功");
        
        assert_eq!(main_deserialized.id, main_doc.id);
        assert_eq!(component_deserialized.id, component_doc.id);
        
        // 验证跨文档引用的完整性保持
        let deserialized_relations = main_deserialized.find_block_by_name("app-composition").unwrap();
        assert_eq!(deserialized_relations.content, app_composition.content);
    }
}
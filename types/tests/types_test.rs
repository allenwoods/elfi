//! types 模块单元测试
//!
//! 开发者职责：
//! 1. 为每个公共API编写测试
//! 2. 覆盖边界条件
//! 3. 使用真实实现+Mock依赖

// ============ 开发者测试区域 开始 ============

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use types::interface::TypeInterface;
    use types::{Block, BlockContent, Document, Relation};
    use uuid::Uuid;

    // ===== Document 测试 =====

    /// 测试目标: src/document.rs 的 Document::new 函数
    #[test]
    fn test_document_creation() {
        let doc = Document::new("test-doc".to_string());
        assert_eq!(doc.id, "test-doc");
        assert_eq!(doc.blocks.len(), 0);
        assert!(doc.metadata.created_at.len() > 0);
        assert!(doc.metadata.updated_at.len() > 0);
        assert_eq!(doc.metadata.version, 1);
    }

    /// 测试目标: src/document.rs 的 find_block 函数
    #[test]
    fn test_document_find_block() {
        let mut doc = Document::new("test-doc".to_string());
        let block_id = Uuid::new_v4().to_string();
        let block = Block::new(block_id.clone(), "markdown".to_string());
        doc.blocks.push(block);

        let found_block = doc.find_block(&block_id);
        assert!(found_block.is_some());
        assert_eq!(found_block.unwrap().id, block_id);

        let not_found = doc.find_block("non-existent");
        assert!(not_found.is_none());
    }

    /// 测试目标: Document 的 find_block_by_name 功能
    #[test]
    fn test_document_find_block_by_name() {
        let mut doc = Document::new("test-doc".to_string());
        let mut block = Block::new(Uuid::new_v4().to_string(), "markdown".to_string());
        block.name = Some("test-block".to_string());
        doc.blocks.push(block);

        let found_block = doc.find_block_by_name("test-block");
        assert!(found_block.is_some());
        assert_eq!(found_block.unwrap().name.as_ref().unwrap(), "test-block");

        let not_found = doc.find_block_by_name("non-existent");
        assert!(not_found.is_none());
    }

    // ===== Block 测试 =====

    /// 测试目标: src/block.rs 的 Block::new 函数
    #[test]
    fn test_block_creation() {
        let block_id = Uuid::new_v4().to_string();
        let block = Block::new(block_id.clone(), "markdown".to_string());
        assert_eq!(block.id, block_id);
        assert_eq!(block.block_type, "markdown");
        assert!(block.name.is_none());
        assert_eq!(block.attributes.len(), 0);
        assert!(matches!(block.content, BlockContent::Text(ref s) if s.is_empty()));
    }

    /// 测试目标: src/block.rs 的 Block::with_name 函数
    #[test]
    fn test_block_with_name() {
        let block_id = Uuid::new_v4().to_string();
        let block =
            Block::new(block_id.clone(), "code".to_string()).with_name("test-block".to_string());
        assert_eq!(block.name, Some("test-block".to_string()));
    }

    /// 测试目标: src/block.rs 的 Block::with_content 函数
    #[test]
    fn test_block_with_content() {
        let block_id = Uuid::new_v4().to_string();
        let content = "# Hello World\nThis is markdown content.".to_string();
        let block = Block::new(block_id, "markdown".to_string())
            .with_content(BlockContent::Text(content.clone()));
        assert!(matches!(block.content, BlockContent::Text(ref s) if s == &content));
    }

    /// 测试目标: src/block.rs 的 Block::with_attributes 函数
    #[test]
    fn test_block_with_attributes() {
        let block_id = Uuid::new_v4().to_string();
        let mut attributes = HashMap::new();
        attributes.insert(
            "language".to_string(),
            serde_json::Value::String("rust".to_string()),
        );
        attributes.insert(
            "author".to_string(),
            serde_json::Value::String("alice".to_string()),
        );

        let block = Block::new(block_id, "code".to_string()).with_attributes(attributes.clone());
        assert_eq!(block.attributes, attributes);
    }

    /// 测试目标: src/block.rs 的 Block::validate 函数 - 成功情况
    #[test]
    fn test_block_validation_success() {
        let block_id = Uuid::new_v4().to_string();
        let block = Block::new(block_id, "markdown".to_string());
        assert!(block.validate().is_ok());
    }

    /// 测试目标: src/block.rs 的 Block::validate 函数 - 错误情况
    #[test]
    fn test_block_validation_error() {
        // 测试空ID
        let block = Block {
            id: "".to_string(),
            name: None,
            block_type: "markdown".to_string(),
            attributes: HashMap::new(),
            content: BlockContent::Text("".to_string()),
        };
        assert!(block.validate().is_err());

        // 测试空类型
        let block = Block {
            id: Uuid::new_v4().to_string(),
            name: None,
            block_type: "".to_string(),
            attributes: HashMap::new(),
            content: BlockContent::Text("".to_string()),
        };
        assert!(block.validate().is_err());
    }

    // ===== Relation 测试 =====

    /// 测试目标: src/relation.rs 的 Relation::new 函数
    #[test]
    fn test_relation_creation() {
        let relation = Relation::new(
            "block1".to_string(),
            "block2".to_string(),
            "child_of".to_string(),
        );
        assert_eq!(relation.from, "block1");
        assert_eq!(relation.to, "block2");
        assert_eq!(relation.relation_type, "child_of");
        assert_eq!(relation.attributes.len(), 0);
    }

    /// 测试目标: src/relation.rs 的 Relation::with_attributes 函数
    #[test]
    fn test_relation_with_attributes() {
        let mut attributes = HashMap::new();
        attributes.insert("weight".to_string(), serde_json::json!(1.0));

        let relation = Relation::new(
            "block1".to_string(),
            "block2".to_string(),
            "references".to_string(),
        )
        .with_attributes(attributes.clone());
        assert_eq!(relation.attributes, attributes);
    }

    /// 测试目标: src/relation.rs 的 Relation::validate 函数 - 成功情况
    #[test]
    fn test_relation_validation_success() {
        let relation = Relation::new(
            "block1".to_string(),
            "block2".to_string(),
            "child_of".to_string(),
        );
        assert!(relation.validate().is_ok());
    }

    /// 测试目标: src/relation.rs 的 Relation::validate 函数 - 错误情况
    #[test]
    fn test_relation_validation_error() {
        // 测试空from
        let relation = Relation::new("".to_string(), "block2".to_string(), "child_of".to_string());
        assert!(relation.validate().is_err());

        // 测试空to
        let relation = Relation::new("block1".to_string(), "".to_string(), "child_of".to_string());
        assert!(relation.validate().is_err());

        // 测试空关系类型
        let relation = Relation::new("block1".to_string(), "block2".to_string(), "".to_string());
        assert!(relation.validate().is_err());

        // 测试相同的from和to（自引用）
        let relation = Relation::new(
            "block1".to_string(),
            "block1".to_string(),
            "child_of".to_string(),
        );
        assert!(relation.validate().is_err());
    }

    // ===== BlockContent 测试 =====

    /// 测试目标: BlockContent 枚举的不同变体
    #[test]
    fn test_block_content_variants() {
        // 测试Text内容
        let text_content = BlockContent::Text("Hello World".to_string());
        assert!(matches!(text_content, BlockContent::Text(_)));

        // 测试Relations内容
        let relations_content =
            BlockContent::Relations("block1 -> block2 [child_of] {}".to_string());
        assert!(matches!(relations_content, BlockContent::Relations(_)));

        // 测试Binary内容
        let binary_content = BlockContent::Binary(vec![1, 2, 3, 4]);
        assert!(matches!(binary_content, BlockContent::Binary(_)));
    }

    // ===== Interface和Mock测试 =====

    /// 测试目标: src/interface.rs 的 TypeInterface trait
    /// 依赖模块: 无 (使用Default实现)
    #[test]
    fn test_type_interface_validate_block() {
        use types::DefaultTypeInterface;
        let interface = DefaultTypeInterface::new();
        let block = Block::new(Uuid::new_v4().to_string(), "markdown".to_string());
        assert!(interface.validate_block(&block).is_ok());
    }

    /// 测试目标: TypeInterface 的 validate_relation
    #[test]
    fn test_type_interface_validate_relation() {
        use types::DefaultTypeInterface;
        let interface = DefaultTypeInterface::new();
        let relation = Relation::new(
            "block1".to_string(),
            "block2".to_string(),
            "child_of".to_string(),
        );
        assert!(interface.validate_relation(&relation).is_ok());
    }

    /// 测试目标: TypeInterface 的序列化/反序列化往返测试
    #[test]
    fn test_serialization_roundtrip() {
        use types::DefaultTypeInterface;
        let interface = DefaultTypeInterface::new();
        let doc = Document::new("test-doc".to_string());

        let serialized = interface
            .serialize_document(&doc)
            .expect("Serialization should succeed");
        assert!(!serialized.is_empty());

        let deserialized = interface
            .deserialize_document(&serialized)
            .expect("Deserialization should succeed");
        assert_eq!(deserialized.id, doc.id);
    }

    // ===== 边界条件和错误处理测试 =====

    /// 测试目标: 边界条件测试 - 空文档
    #[test]
    fn test_empty_document() {
        let doc = Document::new("".to_string()); // 空ID
        assert_eq!(doc.id, "");
        assert_eq!(doc.blocks.len(), 0);
    }

    /// 测试目标: 错误条件测试 - UUID格式验证
    #[test]
    fn test_uuid_validation() {
        // 测试有效的UUID
        let valid_uuid = Uuid::new_v4().to_string();
        let block = Block::new(valid_uuid, "markdown".to_string());
        assert!(block.validate().is_ok());

        // 测试无效的UUID格式 - 这应该在create时处理，但我们测试validate行为
        let invalid_block = Block {
            id: "invalid-uuid".to_string(),
            name: None,
            block_type: "markdown".to_string(),
            attributes: HashMap::new(),
            content: BlockContent::Text("".to_string()),
        };
        assert!(invalid_block.validate().is_err());
    }

    /// 测试目标: 文档中重复的块名称检测
    #[test]
    fn test_duplicate_block_names() {
        let mut doc = Document::new("test-doc".to_string());

        let block1 = Block::new(Uuid::new_v4().to_string(), "markdown".to_string())
            .with_name("duplicate-name".to_string());
        let block2 = Block::new(Uuid::new_v4().to_string(), "code".to_string())
            .with_name("duplicate-name".to_string());

        doc.blocks.push(block1);
        doc.blocks.push(block2);

        let result = doc.validate_unique_names();
        assert!(result.is_err());
    }

    /// 测试目标: 大量数据处理性能
    #[test]
    fn test_large_document_performance() {
        let mut doc = Document::new("large-doc".to_string());

        // 添加1000个块
        for i in 0..1000 {
            let block = Block::new(Uuid::new_v4().to_string(), "markdown".to_string())
                .with_name(format!("block-{}", i))
                .with_content(BlockContent::Text(format!("Content for block {}", i)));
            doc.blocks.push(block);
        }

        assert_eq!(doc.blocks.len(), 1000);

        // 测试查找性能 - 应该很快找到
        let target_name = "block-500";
        let found = doc.find_block_by_name(target_name);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name.as_ref().unwrap(), target_name);
    }

    /// 测试目标: Relations Block 的特殊处理
    #[test]
    fn test_relations_block_content() {
        let relations_content = r#"
            # 文档内关系
            setup-code -> introduction [child_of] {}
            introduction -> summary [parent_of] {weight: 1.0}
            
            # 跨文档引用
            introduction -> elf://shared-lib/utils/helpers#string-utils [references] {display_text: "共享工具函数"}
        "#;

        let block = Block::new(Uuid::new_v4().to_string(), "relations".to_string())
            .with_name("document-relations".to_string())
            .with_content(BlockContent::Relations(relations_content.to_string()));

        assert!(matches!(block.content, BlockContent::Relations(_)));
        assert_eq!(block.block_type, "relations");
    }

    /// 测试目标: 深层嵌套属性处理
    #[test]
    fn test_nested_attributes() {
        let mut attributes = HashMap::new();

        // 创建嵌套的JSON对象
        let nested_obj = serde_json::json!({
            "metadata": {
                "author": "Alice",
                "tags": ["important", "draft"],
                "config": {
                    "render": true,
                    "cache": false
                }
            }
        });

        attributes.insert("complex".to_string(), nested_obj);

        let block = Block::new(Uuid::new_v4().to_string(), "markdown".to_string())
            .with_attributes(attributes);

        // 验证能正确处理复杂的嵌套结构
        assert!(block.validate().is_ok());
        assert!(block.attributes.contains_key("complex"));
    }
}

// ===== 序列化/反序列化专项测试 =====
#[cfg(test)]
mod serialization_tests {
    use std::collections::HashMap;
    use types::{Block, BlockContent, Document, Relation};
    use uuid::Uuid;

    /// 测试目标: Document 的 serde 序列化支持
    #[test]
    fn test_document_serde() {
        let doc = Document::new("test-doc".to_string());

        // 序列化到JSON
        let json = serde_json::to_string(&doc).expect("Should serialize to JSON");
        assert!(!json.is_empty());

        // 从JSON反序列化
        let deserialized: Document =
            serde_json::from_str(&json).expect("Should deserialize from JSON");
        assert_eq!(deserialized.id, doc.id);
        assert_eq!(deserialized.blocks.len(), doc.blocks.len());
    }

    /// 测试目标: Block 的 serde 序列化支持
    #[test]
    fn test_block_serde() {
        let mut attributes = HashMap::new();
        attributes.insert(
            "language".to_string(),
            serde_json::Value::String("rust".to_string()),
        );

        let block = Block::new(Uuid::new_v4().to_string(), "code".to_string())
            .with_name("test-code".to_string())
            .with_attributes(attributes)
            .with_content(BlockContent::Text("fn main() {}".to_string()));

        // 序列化到JSON
        let json = serde_json::to_string(&block).expect("Should serialize to JSON");
        assert!(!json.is_empty());

        // 从JSON反序列化
        let deserialized: Block =
            serde_json::from_str(&json).expect("Should deserialize from JSON");
        assert_eq!(deserialized.id, block.id);
        assert_eq!(deserialized.name, block.name);
        assert_eq!(deserialized.block_type, block.block_type);
        assert_eq!(deserialized.attributes, block.attributes);
    }

    /// 测试目标: Relation 的 serde 序列化支持
    #[test]
    fn test_relation_serde() {
        let mut attributes = HashMap::new();
        attributes.insert("weight".to_string(), serde_json::json!(0.8));

        let relation = Relation::new(
            "block1".to_string(),
            "block2".to_string(),
            "references".to_string(),
        )
        .with_attributes(attributes);

        // 序列化到JSON
        let json = serde_json::to_string(&relation).expect("Should serialize to JSON");
        assert!(!json.is_empty());

        // 从JSON反序列化
        let deserialized: Relation =
            serde_json::from_str(&json).expect("Should deserialize from JSON");
        assert_eq!(deserialized.from, relation.from);
        assert_eq!(deserialized.to, relation.to);
        assert_eq!(deserialized.relation_type, relation.relation_type);
        assert_eq!(deserialized.attributes, relation.attributes);
    }

    /// 测试目标: BlockContent 各种变体的序列化
    #[test]
    fn test_block_content_serde() {
        // 测试Text内容
        let text_content = BlockContent::Text("Hello World".to_string());
        let json = serde_json::to_string(&text_content).expect("Should serialize");
        let deserialized: BlockContent = serde_json::from_str(&json).expect("Should deserialize");
        assert!(matches!(deserialized, BlockContent::Text(ref s) if s == "Hello World"));

        // 测试Relations内容
        let relations_content =
            BlockContent::Relations("block1 -> block2 [child_of] {}".to_string());
        let json = serde_json::to_string(&relations_content).expect("Should serialize");
        let deserialized: BlockContent = serde_json::from_str(&json).expect("Should deserialize");
        assert!(matches!(deserialized, BlockContent::Relations(_)));

        // 测试Binary内容
        let binary_content = BlockContent::Binary(vec![1, 2, 3, 4, 5]);
        let json = serde_json::to_string(&binary_content).expect("Should serialize");
        let deserialized: BlockContent = serde_json::from_str(&json).expect("Should deserialize");
        assert!(matches!(deserialized, BlockContent::Binary(ref v) if v == &vec![1, 2, 3, 4, 5]));
    }
}

// ============ 开发者测试区域 结束 ============

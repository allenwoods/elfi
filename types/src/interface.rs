//! Interface trait 定义
//!
//! 开发者实现区域 - 请定义types模块的公共接口

use crate::{Block, Document, Relation, TypesError};

// ============ 开发者实现区域 开始 ============

/// Types 模块的主要接口
pub trait TypeInterface: Send + Sync {
    /// 验证块
    fn validate_block(&self, block: &Block) -> Result<(), TypesError>;

    /// 序列化文档
    fn serialize_document(&self, doc: &Document) -> Result<String, TypesError>;

    /// 反序列化文档
    fn deserialize_document(&self, content: &str) -> Result<Document, TypesError>;

    /// 验证关系
    fn validate_relation(&self, relation: &Relation) -> Result<(), TypesError>;
}

/// 默认的TypeInterface实现
pub struct DefaultTypeInterface;

impl DefaultTypeInterface {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultTypeInterface {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeInterface for DefaultTypeInterface {
    fn validate_block(&self, block: &Block) -> Result<(), TypesError> {
        block.validate()
    }

    fn serialize_document(&self, doc: &Document) -> Result<String, TypesError> {
        serde_json::to_string_pretty(doc).map_err(TypesError::from)
    }

    fn deserialize_document(&self, content: &str) -> Result<Document, TypesError> {
        serde_json::from_str(content).map_err(TypesError::from)
    }

    fn validate_relation(&self, relation: &Relation) -> Result<(), TypesError> {
        relation.validate()
    }
}

// ============ 开发者实现区域 结束 ============

//! Interface trait 定义
//! 
//! 开发者实现区域 - 请定义types模块的公共接口

use crate::{Document, Block, Relation, TypesError};

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

// ============ 开发者实现区域 结束 ============
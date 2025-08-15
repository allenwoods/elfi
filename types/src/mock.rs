//! Mock 实现用于测试
//! 
//! 开发者测试区域 - 请实现Mock类型

#[cfg(test)]
use crate::{Document, Block, Relation, TypesError, interface::TypeInterface};

// ============ 开发者测试区域 开始 ============

#[cfg(test)]
pub struct MockTypeInterface {
    // TODO: 添加Mock需要的字段
}

#[cfg(test)]
impl MockTypeInterface {
    pub fn new() -> Self {
        // TODO: 实现Mock初始化
        todo!("实现Mock初始化")
    }
}

#[cfg(test)]
impl TypeInterface for MockTypeInterface {
    fn validate_block(&self, _block: &Block) -> Result<(), TypesError> {
        // TODO: 实现Mock块验证
        Ok(())
    }
    
    fn serialize_document(&self, _doc: &Document) -> Result<String, TypesError> {
        // TODO: 实现Mock序列化
        Ok("mock_serialized".to_string())
    }
    
    fn deserialize_document(&self, _content: &str) -> Result<Document, TypesError> {
        // TODO: 实现Mock反序列化
        todo!("实现Mock反序列化")
    }
    
    fn validate_relation(&self, _relation: &Relation) -> Result<(), TypesError> {
        // TODO: 实现Mock关系验证
        Ok(())
    }
}

// ============ 开发者测试区域 结束 ============
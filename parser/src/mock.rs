//! Parser Mock 实现
//! 开发者测试区域

#[cfg(test)]
use crate::interface::ParserInterface;
#[cfg(test)]
use types::{Document, Block, Relation, TypesError};

// ============ 开发者测试区域 开始 ============

#[cfg(test)]
pub struct MockParser;

#[cfg(test)]
impl ParserInterface for MockParser {
    fn parse_file(&self, _content: &str) -> Result<Document, TypesError> {
        todo!("实现Mock解析")
    }
    
    fn parse_block(&self, _content: &str, _block_type: &str) -> Result<Block, TypesError> {
        todo!("实现Mock块解析")
    }
    
    fn parse_relations(&self, _content: &str) -> Result<Vec<Relation>, TypesError> {
        Ok(vec![])
    }
}

// ============ 开发者测试区域 结束 ============
//! Parser Interface 定义
//! 开发者实现区域

use types::{Document, Block, Relation, TypesError};

// ============ 开发者实现区域 开始 ============

pub trait ParserInterface: Send + Sync {
    fn parse_file(&self, content: &str) -> Result<Document, TypesError>;
    fn parse_block(&self, content: &str, block_type: &str) -> Result<Block, TypesError>;
    fn parse_relations(&self, content: &str) -> Result<Vec<Relation>, TypesError>;
}

// ============ 开发者实现区域 结束 ============
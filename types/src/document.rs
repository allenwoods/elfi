//! Document 数据结构定义
//!
//! 开发者实现区域 - 请实现Document结构体和相关方法

use crate::{Block, TypesError};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============ 开发者实现区域 开始 ============

/// Document 结构体
/// 参考 plans/04-phase1-a-types.md 中的设计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub blocks: Vec<Block>,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub created_at: String,
    pub updated_at: String,
    pub version: u64,
    pub attributes: HashMap<String, serde_json::Value>,
}

impl Document {
    /// 创建新文档
    pub fn new(id: String) -> Self {
        let now = Utc::now().to_rfc3339();
        let metadata = DocumentMetadata {
            created_at: now.clone(),
            updated_at: now,
            version: 1,
            attributes: HashMap::new(),
        };

        Self {
            id,
            blocks: Vec::new(),
            metadata,
        }
    }

    /// 查找块
    pub fn find_block(&self, block_id: &str) -> Option<&Block> {
        self.blocks.iter().find(|block| block.id == block_id)
    }

    /// 根据块名称查找块
    pub fn find_block_by_name(&self, block_name: &str) -> Option<&Block> {
        self.blocks.iter().find(|block| {
            block
                .name
                .as_ref()
                .map(|name| name == block_name)
                .unwrap_or(false)
        })
    }

    /// 验证文档中块名称的唯一性
    pub fn validate_unique_names(&self) -> Result<(), TypesError> {
        let mut seen_names = std::collections::HashSet::new();

        for block in &self.blocks {
            if let Some(name) = &block.name
                && !seen_names.insert(name.clone())
            {
                return Err(TypesError::DocumentValidation {
                    message: format!("Duplicate block name: {}", name),
                });
            }
        }

        Ok(())
    }
}

// ============ 开发者实现区域 结束 ============

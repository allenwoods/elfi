//! Block 数据结构定义
//!
//! 开发者实现区域 - 请实现Block结构体和相关方法

use crate::TypesError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============ 开发者实现区域 开始 ============

/// Block 结构体 - 4字段设计
/// 参考 plans/04-phase1-a-types.md 中的设计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    pub name: Option<String>,
    pub block_type: String,
    pub attributes: HashMap<String, serde_json::Value>,
    pub content: BlockContent,
}

/// Block 内容枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BlockContent {
    Text(String),
    Relations(String), // Relations 语法
    Binary(Vec<u8>),
}

impl Block {
    /// 创建新块
    pub fn new(id: String, block_type: String) -> Self {
        Self {
            id,
            name: None,
            block_type,
            attributes: HashMap::new(),
            content: BlockContent::Text(String::new()),
        }
    }

    /// 设置块名称（建造者模式）
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// 设置块内容（建造者模式）
    pub fn with_content(mut self, content: BlockContent) -> Self {
        self.content = content;
        self
    }

    /// 设置块属性（建造者模式）
    pub fn with_attributes(mut self, attributes: HashMap<String, serde_json::Value>) -> Self {
        self.attributes = attributes;
        self
    }

    /// 验证块的有效性
    pub fn validate(&self) -> Result<(), TypesError> {
        // 验证ID不能为空
        if self.id.is_empty() {
            return Err(TypesError::BlockValidation {
                message: "Block ID cannot be empty".to_string(),
            });
        }

        // 验证UUID格式（如果不是空字符串）
        if !self.id.is_empty() && Uuid::parse_str(&self.id).is_err() {
            return Err(TypesError::BlockValidation {
                message: format!("Invalid UUID format for block ID: {}", self.id),
            });
        }

        // 验证类型不能为空
        if self.block_type.is_empty() {
            return Err(TypesError::BlockValidation {
                message: "Block type cannot be empty".to_string(),
            });
        }

        // 验证名称如果存在不能为空字符串
        if let Some(name) = &self.name
            && name.is_empty()
        {
            return Err(TypesError::BlockValidation {
                message: "Block name cannot be empty string if specified".to_string(),
            });
        }

        Ok(())
    }
}

// ============ 开发者实现区域 结束 ============

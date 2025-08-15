//! Relation 数据结构定义
//!
//! 开发者实现区域 - 请实现Relation结构体和相关方法

use crate::TypesError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============ 开发者实现区域 开始 ============

/// Relation 结构体
/// 参考 plans/04-phase1-a-types.md 中的设计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub from: String,
    pub to: String,
    pub relation_type: String,
    pub attributes: HashMap<String, serde_json::Value>,
}

impl Relation {
    /// 创建新关系
    pub fn new(from: String, to: String, relation_type: String) -> Self {
        Self {
            from,
            to,
            relation_type,
            attributes: HashMap::new(),
        }
    }

    /// 设置关系属性（建造者模式）
    pub fn with_attributes(mut self, attributes: HashMap<String, serde_json::Value>) -> Self {
        self.attributes = attributes;
        self
    }

    /// 验证关系的有效性
    pub fn validate(&self) -> Result<(), TypesError> {
        // 验证from不能为空
        if self.from.is_empty() {
            return Err(TypesError::RelationValidation {
                message: "Relation 'from' field cannot be empty".to_string(),
            });
        }

        // 验证to不能为空
        if self.to.is_empty() {
            return Err(TypesError::RelationValidation {
                message: "Relation 'to' field cannot be empty".to_string(),
            });
        }

        // 验证关系类型不能为空
        if self.relation_type.is_empty() {
            return Err(TypesError::RelationValidation {
                message: "Relation type cannot be empty".to_string(),
            });
        }

        // 验证不能自引用（from和to相同）
        if self.from == self.to {
            return Err(TypesError::RelationValidation {
                message: format!("Self-reference not allowed: {} -> {}", self.from, self.to),
            });
        }

        Ok(())
    }
}

// ============ 开发者实现区域 结束 ============

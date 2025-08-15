//! 错误类型定义
//!
//! 开发者实现区域 - 请实现TypesError和相关错误处理

// ============ 开发者实现区域 开始 ============

/// Types 模块的错误类型
#[derive(Debug, thiserror::Error)]
pub enum TypesError {
    #[error("Document validation failed: {message}")]
    DocumentValidation { message: String },

    #[error("Block validation failed: {message}")]
    BlockValidation { message: String },

    #[error("Relation validation failed: {message}")]
    RelationValidation { message: String },

    #[error("Serialization error: {source}")]
    Serialization { source: serde_json::Error },

    #[error("Not found: {item}")]
    NotFound { item: String },

    #[error("Invalid format: {details}")]
    InvalidFormat { details: String },
}

impl From<serde_json::Error> for TypesError {
    fn from(error: serde_json::Error) -> Self {
        TypesError::Serialization { source: error }
    }
}

// ============ 开发者实现区域 结束 ============

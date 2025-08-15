//! types - 核心数据结构定义
//!
//! 参考: plans/04-phase1-a-types.md
//!
//! 开发者可修改区域：
//! - 所有结构体定义和trait实现
//! - 序列化/反序列化逻辑
//! - 错误处理和验证
//!
//! 不可修改：
//! - 公共API签名（需讨论）
//! - 依赖版本（需通过cargo add管理）

// ============ 开发者实现区域 开始 ============

pub mod block;
pub mod document;
pub mod error;
pub mod interface;
pub mod relation;

#[cfg(test)]
pub mod mock;

// 重新导出主要类型
pub use block::{Block, BlockContent};
pub use document::{Document, DocumentMetadata};
pub use error::TypesError;
pub use interface::{DefaultTypeInterface, TypeInterface};
pub use relation::Relation;

// ============ 开发者实现区域 结束 ============

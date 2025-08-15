//! # ELFI Types 模块
//!
//! ELFI Types 模块提供了整个ELFI系统的核心数据结构，包括文档、块、关系等核心抽象。
//!
//! ## 核心类型
//!
//! - [`Document`]: 文档容器，管理一组相关的块
//! - [`Block`]: 基础内容单元，支持多种内容类型
//! - [`Relation`]: 描述块之间关系的对象  
//! - [`TypeInterface`]: 可扩展的类型处理接口
//!
//! ## 设计特性
//!
//! - **类型安全**: 基于Rust强类型系统，编译时错误捕获
//! - **CRDT友好**: 扁平存储结构，支持分布式协作
//! - **可扩展**: 通过trait系统支持插件化扩展
//! - **高性能**: 优化的数据结构，支持大规模文档
//!
//! ## 使用示例
//!
//! ```rust
//! use types::{Document, Block, BlockContent};
//! use uuid::Uuid;
//!
//! // 创建文档
//! let mut doc = Document::new(Uuid::new_v4().to_string());
//!
//! // 创建块
//! let block = Block::new(Uuid::new_v4().to_string(), "markdown".to_string())
//!     .with_name("introduction".to_string())
//!     .with_content(BlockContent::Text("# Hello ELFI".to_string()));
//!
//! // 添加到文档
//! doc.blocks.push(block);
//! ```

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

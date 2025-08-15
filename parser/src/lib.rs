//! parser - .elf文件语法解析
//! 
//! 参考: plans/04-phase1-b-parser.md
//! 
//! 开发者可修改区域：
//! - 解析器实现
//! - 语法验证逻辑
//! - 错误处理
//! 
//! 不可修改：
//! - 公共API签名（需讨论）
//! - 依赖版本（需通过cargo add管理）

// ============ 开发者实现区域 开始 ============

pub mod elf_parser;
pub mod block_parser;
pub mod relations_parser;
pub mod interface;

#[cfg(test)]
pub mod mock;

pub use elf_parser::ElfParser;
pub use interface::ParserInterface;

// ============ 开发者实现区域 结束 ============

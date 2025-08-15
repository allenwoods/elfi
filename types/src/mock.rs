//! Mock 实现用于测试
//!
//! 开发者测试区域 - 请实现Mock类型

#[cfg(test)]
use crate::{Block, Document, Relation, TypesError, interface::TypeInterface};
#[cfg(test)]
use std::sync::{Arc, Mutex};

// ============ 开发者测试区域 开始 ============

#[cfg(test)]
pub struct MockTypeInterface {
    /// 记录方法调用次数，用于验证Mock行为
    pub validation_calls: Arc<Mutex<usize>>,
    pub serialization_calls: Arc<Mutex<usize>>,
    pub deserialization_calls: Arc<Mutex<usize>>,

    /// 控制返回结果
    pub should_fail_validation: bool,
    pub should_fail_serialization: bool,
    pub should_fail_deserialization: bool,
}

#[cfg(test)]
impl MockTypeInterface {
    pub fn new() -> Self {
        Self {
            validation_calls: Arc::new(Mutex::new(0)),
            serialization_calls: Arc::new(Mutex::new(0)),
            deserialization_calls: Arc::new(Mutex::new(0)),
            should_fail_validation: false,
            should_fail_serialization: false,
            should_fail_deserialization: false,
        }
    }

    /// 创建一个会失败验证的Mock
    pub fn with_validation_failure() -> Self {
        Self {
            validation_calls: Arc::new(Mutex::new(0)),
            serialization_calls: Arc::new(Mutex::new(0)),
            deserialization_calls: Arc::new(Mutex::new(0)),
            should_fail_validation: true,
            should_fail_serialization: false,
            should_fail_deserialization: false,
        }
    }

    /// 创建一个会失败序列化的Mock
    pub fn with_serialization_failure() -> Self {
        Self {
            validation_calls: Arc::new(Mutex::new(0)),
            serialization_calls: Arc::new(Mutex::new(0)),
            deserialization_calls: Arc::new(Mutex::new(0)),
            should_fail_validation: false,
            should_fail_serialization: true,
            should_fail_deserialization: false,
        }
    }

    /// 获取验证调用次数
    pub fn validation_call_count(&self) -> usize {
        *self.validation_calls.lock().unwrap()
    }

    /// 获取序列化调用次数
    pub fn serialization_call_count(&self) -> usize {
        *self.serialization_calls.lock().unwrap()
    }

    /// 获取反序列化调用次数
    pub fn deserialization_call_count(&self) -> usize {
        *self.deserialization_calls.lock().unwrap()
    }
}

#[cfg(test)]
impl Default for MockTypeInterface {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl TypeInterface for MockTypeInterface {
    fn validate_block(&self, block: &Block) -> Result<(), TypesError> {
        {
            let mut calls = self.validation_calls.lock().unwrap();
            *calls += 1;
        }

        if self.should_fail_validation {
            return Err(TypesError::BlockValidation {
                message: "Mock validation failure".to_string(),
            });
        }

        // 实际调用真实的验证逻辑
        block.validate()
    }

    fn serialize_document(&self, doc: &Document) -> Result<String, TypesError> {
        {
            let mut calls = self.serialization_calls.lock().unwrap();
            *calls += 1;
        }

        if self.should_fail_serialization {
            // 创建一个IO错误然后转换为serde_json::Error
            let io_error =
                std::io::Error::new(std::io::ErrorKind::Other, "Mock serialization failure");
            return Err(TypesError::Serialization {
                source: serde_json::Error::io(io_error),
            });
        }

        // 对于Mock，我们可以返回一个简单的JSON表示
        // 但为了测试往返，我们使用真实的序列化
        serde_json::to_string_pretty(doc).map_err(TypesError::from)
    }

    fn deserialize_document(&self, content: &str) -> Result<Document, TypesError> {
        {
            let mut calls = self.deserialization_calls.lock().unwrap();
            *calls += 1;
        }

        if self.should_fail_deserialization {
            return Err(TypesError::InvalidFormat {
                details: "Mock deserialization failure".to_string(),
            });
        }

        // 对于Mock，我们使用真实的反序列化以支持往返测试
        serde_json::from_str(content).map_err(TypesError::from)
    }

    fn validate_relation(&self, relation: &Relation) -> Result<(), TypesError> {
        {
            let mut calls = self.validation_calls.lock().unwrap();
            *calls += 1;
        }

        if self.should_fail_validation {
            return Err(TypesError::RelationValidation {
                message: "Mock relation validation failure".to_string(),
            });
        }

        // 实际调用真实的验证逻辑
        relation.validate()
    }
}

// ============ 开发者测试区域 结束 ============

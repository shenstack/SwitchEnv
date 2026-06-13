use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("平台错误: {0}")]
    Platform(#[from] crate::platforms::PlatformError),
    #[error("数据库错误: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("迁移错误: {0}")]
    Migration(String),
    #[error("剪贴板错误: {0}")]
    Clipboard(String),
    #[error("未找到: {0}")]
    NotFound(String),
    #[error("验证错误: {0}")]
    Validation(String),
    #[error("其他错误: {0}")]
    Other(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

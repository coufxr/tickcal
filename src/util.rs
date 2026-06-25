//! 共享工具函数

use std::path::PathBuf;

/// 获取应用配置目录
///
/// - Debug 模式：当前工作目录（项目根目录），方便开发调试
/// - Release 模式：系统配置目录（Windows: %APPDATA%/calendar/）
pub fn config_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from(".")
    } else {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("calendar")
    }
}

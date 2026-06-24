//! 日历模块（MVU 模式）
//!
//! - message: 所有可能的 UI 事件
//! - model: 日历应用状态
//! - update: 纯函数处理消息 + UI 刷新
//! - bridge: Slint ↔ MVU 桥接（通道 + Timer + 回调注册）

pub mod bridge;
mod message;
mod model;
mod update;

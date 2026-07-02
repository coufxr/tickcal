//! 平台特定逻辑
//!
//! 封装各平台差异（macOS 激活策略、Windows DPI、Linux 桌面集成等），
//! 对上层 `main.rs` 提供统一接口。

use std::path::PathBuf;

/// 获取应用名称（用于路径、菜单等），自动从 Cargo.toml 的 `name` 字段读取
pub fn app_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

/// 获取应用数据目录（数据库文件）
pub fn data_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from(".")
    } else {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(app_name())
    }
}

/// 初始化平台特定行为
///
/// 应在应用启动时尽早调用。
pub fn init() {
    #[cfg(target_os = "macos")]
    init_macos();

    #[cfg(target_os = "windows")]
    init_windows();

    #[cfg(target_os = "linux")]
    init_linux();

    log::info!("[platform] init: target_os={}", std::env::consts::OS);
}

// ─── macOS ──────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
fn init_macos() {
    // macOS: Slint 默认以 Regular 应用类型运行，Dock 图标和菜单栏自动生效。
    // Slint Rust API 未暴露 NSApplication activationPolicy 设置接口，
    // 如需自定义请通过 objc 运行时调用 [NSApp setActivationPolicy:]。
    log::info!("[platform] macOS initialised");

    if let Ok(macos_ver) = std::env::var("MACOSX_DEPLOYMENT_TARGET") {
        log::info!("[platform] macOS deployment target: {}", macos_ver);
    }
}

// ─── Windows ────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn init_windows() {
    log::info!("[platform] Windows init: DPI awareness via manifest");
    // DPI 感知通过 build.rs 嵌入的清单文件启用
}

// ─── Linux ──────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn init_linux() {
    // Linux: 检测桌面环境
    if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        log::info!("[platform] Linux DE: {}", desktop);
    }
    if let Ok(session) = std::env::var("XDG_SESSION_TYPE") {
        log::info!("[platform] Linux session type: {}", session);
    }
}

// ─── 跨平台工具 ────────────────────────────────────────────────

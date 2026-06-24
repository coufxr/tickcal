//! 应用生命跨度管理
//!
//! 负责启动时加载设置和关闭时保存设置。

use slint::{CloseRequestResponse, ComponentHandle};

use crate::AppWindow;
use crate::settings;
use crate::settings::AppSettings;

/// 应用启动：加载设置
pub fn on_start() -> AppSettings {
    settings::load()
}

/// 应用关闭：保存当前 UI 设置到磁盘
pub fn on_close(ui: &AppWindow) {
    let weak = ui.as_weak();
    ui.window().on_close_requested(move || {
        if let Some(ui) = weak.upgrade() {
            let s = AppSettings {
                dark_mode: ui.get_persisted_dark_mode(),
                accent_index: ui.get_persisted_accent_index(),
                week_start_day: ui.get_persisted_week_start_day(),
                cell_size_index: ui.get_persisted_cell_size_index(),
            };
            settings::save(&s);
        }
        CloseRequestResponse::HideWindow
    });
}

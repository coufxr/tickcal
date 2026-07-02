//! 应用生命跨度管理
//!
//! 负责启动时加载设置和关闭时保存设置。

use std::sync::Arc;

use slint::{CloseRequestResponse, ComponentHandle};

use crate::db::Database;
use crate::settings::AppSettings;
use crate::{AppWindow, settings};

/// 应用启动：加载设置
pub fn on_start(db: &Database) -> AppSettings {
    settings::load(db)
}

/// 从 UI 读取当前设置并保存到数据库
pub fn save_settings(ui: &AppWindow, db: &Database) {
    let s = AppSettings {
        dark_mode: ui.get_persisted_dark_mode(),
        accent_index: ui.get_persisted_accent_index(),
        week_start_day: ui.get_persisted_week_start_day(),
        cell_size_index: ui.get_persisted_cell_size_index(),
    };
    settings::save(s, db);
}

/// 应用关闭：显示退出确认对话框
pub fn on_close(ui: &AppWindow, db: Arc<Database>) {
    let weak = ui.as_weak();
    ui.window().on_close_requested(move || {
        if let Some(ui) = weak.upgrade() {
            save_settings(&ui, &db);
            ui.set_quit_open(true);
        }
        CloseRequestResponse::KeepWindowShown
    });
}

//! 应用生命跨度管理
//!
//! 负责启动时初始化（加载设置、应用到 UI、构建初始日历视图）
//! 和关闭时清理（保存设置到磁盘）。

use std::cell::RefCell;
use std::rc::Rc;

use slint::{CloseRequestResponse, ComponentHandle};

use crate::AppWindow;
use crate::model::today_ymd;
use crate::settings;
use crate::state::CalendarState;
use crate::vm::{build_month_vm, set_weekdays_ui};

/// 应用启动：加载设置、应用到 UI、构建初始日历视图
///
/// 返回 CalendarState，已用设置中的 week_start_day 初始化
pub fn on_start(ui: &AppWindow) -> Rc<RefCell<CalendarState>> {
    let today = today_ymd();

    // 加载持久化设置并应用到 UI
    let app_settings = settings::load();
    ui.invoke_apply_settings(
        app_settings.dark_mode,
        app_settings.accent_index,
        app_settings.week_start_day,
        app_settings.cell_size_index,
    );

    // 初始化日历视图
    let start_day = app_settings.week_start_day as usize;
    set_weekdays_ui(ui, start_day);
    ui.set_month_data(build_month_vm((today.0, today.1), today, today, start_day));

    // 创建日历状态
    Rc::new(RefCell::new(CalendarState::new(start_day, today)))
}

/// 注册窗口关闭事件：关闭时将当前 UI 设置保存到磁盘
pub fn on_close(ui: &AppWindow) {
    let weak = ui.as_weak();
    ui.window().on_close_requested(move || {
        if let Some(ui) = weak.upgrade() {
            let s = settings::AppSettings {
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

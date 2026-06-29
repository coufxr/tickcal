//! 应用核心业务逻辑
//!
//! 作为逻辑入口，按模块组织回调注册和 UI 刷新。

use std::cell::RefCell;
use std::rc::Rc;

use slint::{ComponentHandle, Weak};

use crate::AppWindow;
use crate::models::{CalendarModel, TaskModel};

pub mod calendar;
pub mod task;

/// 安全地升级 Weak<AppWindow> 并执行闭包
fn with_ui<F: FnOnce(&AppWindow)>(weak: &Weak<AppWindow>, f: F) {
    if let Some(ui) = weak.upgrade() {
        f(&ui);
    }
}

/// 初始化应用逻辑：刷新 UI + 注册回调
pub fn init(
    ui: &AppWindow,
    calendar_model: &Rc<RefCell<CalendarModel>>,
    task_model: &Rc<RefCell<TaskModel>>,
) {
    // 刷新 UI
    calendar::refresh_calendar_ui(ui, &calendar_model.borrow());
    task::refresh_task_ui(ui, &task_model.borrow());

    // 注册回调
    calendar::register_calendar_callbacks(ui, calendar_model, task_model);
    task::register_task_callbacks(ui, task_model, calendar_model);

    // 设置版本号
    ui.set_app_version(env!("CARGO_PKG_VERSION").into());

    // 菜单栏回调
    let weak = ui.as_weak();
    ui.on_quit(move || {
        if let Some(ui) = weak.upgrade() {
            crate::lifespan::save_settings(&ui);
            slint::quit_event_loop().ok();
        }
    });

    let weak = ui.as_weak();
    ui.on_open_settings(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_settings_open(true);
        }
    });

    let weak = ui.as_weak();
    ui.on_about(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_about_open(true);
        }
    });
}

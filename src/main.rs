// Release 模式下隐藏控制台窗口（Windows）
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::sync::Arc;

mod app_logic;
mod db;
mod lifespan;
mod model;
mod models;
mod settings;

mod util;

use models::{CalendarModel, TaskModel};

// 引入 Slint 编译器生成的模块（包含 AppWindow 等 UI 类型）
slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let ui = AppWindow::new()?;
    let app_settings = lifespan::on_start();

    ui.invoke_apply_settings(
        app_settings.dark_mode,
        app_settings.accent_index,
        app_settings.week_start_day,
        app_settings.cell_size_index,
    );

    // 初始化数据库
    let db = Arc::new(db::Database::new());

    // 创建共享 Model
    let calendar_model = Rc::new(RefCell::new(CalendarModel::new()));
    let task_model = Rc::new(RefCell::new(TaskModel::new(db)));

    // 初始化日历模型：应用设置 + 同步 task 日期
    calendar_model.borrow_mut().apply_settings(app_settings);
    let task_dates = task_model.borrow().task_date_set();
    calendar_model.borrow_mut().set_task_dates(task_dates);

    // 启动时默认选中今天
    let today = {
        let cm = calendar_model.borrow();
        format!("{:04}-{:02}-{:02}", cm.today.0, cm.today.1, cm.today.2)
    };
    task_model.borrow_mut().select_date(today);

    // 初始化系统托盘（Slint 原生 SystemTrayIcon）
    let tray = AppTray::new()?;
    {
        let ui_weak = ui.as_weak();
        tray.on_toggle_window(move || {
            if let Some(w) = ui_weak.upgrade() {
                if w.window().is_visible() {
                    w.hide().ok();
                } else {
                    w.show().ok();
                }
            }
        });
    }
    {
        let ui_weak = ui.as_weak();
        tray.on_quit(move || {
            if let Some(w) = ui_weak.upgrade() {
                lifespan::save_settings(&w);
            }
            slint::quit_event_loop().ok();
        });
    }

    // 初始化应用逻辑
    app_logic::init(&ui, &calendar_model, &task_model);

    lifespan::on_close(&ui);
    ui.run()?;

    Ok(())
}

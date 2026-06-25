// Release 模式下隐藏控制台窗口（Windows）
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

mod app_logic;
mod lifespan;
mod model;
mod models;
mod services;
mod settings;
mod util;

use models::{CalendarModel, TodoModel};

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

    // 创建共享 Model
    let calendar_model = Rc::new(RefCell::new(CalendarModel::new()));
    let todo_model = Rc::new(RefCell::new(TodoModel::new()));

    // 从磁盘加载 todo 数据
    services::store::load(&mut todo_model.borrow_mut());

    // 初始化日历模型：应用设置 + 同步 todo 日期
    calendar_model.borrow_mut().apply_settings(app_settings);
    let todo_dates = todo_model.borrow().todo_date_set();
    calendar_model.borrow_mut().set_todo_dates(todo_dates);

    // 初始化应用逻辑：刷新 UI + 注册回调
    app_logic::init(&ui, &calendar_model, &todo_model);

    lifespan::on_close(&ui);
    ui.run()?;

    Ok(())
}

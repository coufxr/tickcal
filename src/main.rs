// Release 模式下隐藏控制台窗口（Windows）
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

use slint::ComponentHandle;

mod calendar;
mod lifespan;
mod model;
mod settings;

// 引入 Slint 编译器生成的模块（包含 AppWindow 等 UI 类型）
slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志（debug 模式下输出到 stderr，release 模式需设置 RUST_LOG 环境变量）
    env_logger::init();

    let ui = AppWindow::new()?;

    // 启动：加载设置
    let app_settings = lifespan::on_start();

    // 应用设置到 UI
    ui.invoke_apply_settings(
        app_settings.dark_mode,
        app_settings.accent_index,
        app_settings.week_start_day,
        app_settings.cell_size_index,
    );

    // 初始化日历 MVU（通道 + Timer + 回调）
    let _bridge = calendar::bridge::setup(&ui, app_settings);

    // 关闭：保存设置到磁盘
    lifespan::on_close(&ui);

    // 进入事件循环（阻塞直到窗口关闭）
    ui.run()?;

    Ok(())
}

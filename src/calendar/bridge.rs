//! Slint ↔ MVU 桥接层
//!
//! 负责创建消息通道、Timer 轮询、注册 Slint 回调。
//! 将 Slint 的事件驱动模型桥接到 MVU 的消息驱动模型。

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

use slint::{ComponentHandle, Timer, TimerMode};

use crate::AppWindow;
use crate::settings::AppSettings;

use super::message::CalendarMessage;
use super::model::CalendarModel;
use super::update::{refresh_ui, update};

/// Slint ↔ MVU 桥接器
///
/// 持有消息通道和 Timer，生命周期与应用一致。
/// Drop 时 Timer 自动停止。
pub struct Bridge {
    _timer: Timer,
}

/// 初始化 MVU 桥接：创建通道、Model、Timer，注册回调
pub fn setup(ui: &AppWindow, app_settings: AppSettings) -> Bridge {
    // 创建消息通道
    let (tx, rx) = mpsc::channel::<CalendarMessage>();

    // 创建 Model 并从设置初始化
    let model = Rc::new(RefCell::new(CalendarModel::new()));
    update(
        &mut model.borrow_mut(),
        CalendarMessage::ApplySettings(app_settings),
    );
    refresh_ui(ui, &model.borrow());

    // Timer 轮询消息通道，处理消息并刷新 UI
    let rx = Rc::new(RefCell::new(rx));
    let ui_weak = ui.as_weak();
    let model_ref = Rc::clone(&model);
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_millis(100), move || {
        let rx = rx.borrow();
        let mut changed = false;
        while let Ok(msg) = rx.try_recv() {
            update(&mut model_ref.borrow_mut(), msg);
            changed = true;
        }
        drop(rx);
        if changed && let Some(ui) = ui_weak.upgrade() {
            refresh_ui(&ui, &model_ref.borrow());
        }
    });

    // 注册 Slint 回调 → 发送消息到通道
    register_callbacks(ui, &tx);

    Bridge { _timer: timer }
}

/// 注册所有 Slint 回调，将事件发送到消息通道
fn register_callbacks(ui: &AppWindow, tx: &mpsc::Sender<CalendarMessage>) {
    let tx_prev = tx.clone();
    ui.on_prev_month(move || {
        let _ = tx_prev.send(CalendarMessage::PrevMonth);
    });

    let tx_next = tx.clone();
    ui.on_next_month(move || {
        let _ = tx_next.send(CalendarMessage::NextMonth);
    });

    let tx_today = tx.clone();
    ui.on_go_today(move || {
        let _ = tx_today.send(CalendarMessage::GoToday);
    });

    let tx_click = tx.clone();
    ui.on_day_clicked(move |year, month, day| {
        let _ = tx_click.send(CalendarMessage::SelectDay {
            year: year as isize,
            month: month as usize,
            day: day as usize,
        });
    });

    let tx_week = tx.clone();
    ui.on_week_start_changed(move |day| {
        let _ = tx_week.send(CalendarMessage::WeekStartChanged(day as usize));
    });
}

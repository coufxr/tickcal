//! 日历模块：回调注册 + UI 刷新

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::model::{self, DayCell, get_weekday_names};
use crate::models::CalendarModel;
use crate::{AppWindow, CalendarDay, CalendarMonth, WeekdayRow};

/// 将 Model 状态推送到 Slint UI
pub fn refresh_calendar_ui(ui: &AppWindow, model: &CalendarModel) {
    let month_view = model::build_month_view(
        model.display.0,
        model.display.1,
        model.selected,
        model.today,
        model.week_start_day,
    );
    ui.set_month_data(CalendarMonth {
        year: month_view.year as i32,
        month: month_view.month as i32,
        days: ModelRc::new(VecModel::from(
            month_view
                .days
                .into_iter()
                .map(|day| from_day_cell(day, &model.todo_dates))
                .collect::<Vec<_>>(),
        )),
    });

    let weekdays = get_weekday_names(model.week_start_day);
    ui.set_weekdays(WeekdayRow {
        label: ModelRc::new(VecModel::from(
            weekdays
                .iter()
                .map(|s| SharedString::from(*s))
                .collect::<Vec<_>>(),
        )),
    });
}

/// 将内部 DayCell 转换为 Slint 端的 CalendarDay
fn from_day_cell(day: DayCell, todo_dates: &HashSet<String>) -> CalendarDay {
    let date_key = format!("{:04}-{:02}-{:02}", day.year, day.month, day.day);
    CalendarDay {
        year: day.year as i32,
        month: day.month as i32,
        day: day.day as i32,
        is_today: day.is_today,
        is_selected: day.is_selected,
        is_weekend: day.is_weekend,
        is_current_month: day.is_current_month,
        has_todo: todo_dates.contains(&date_key),
    }
}

/// 注册所有日历相关的 Slint 回调
pub fn register_calendar_callbacks(
    ui: &AppWindow,
    model: &Rc<RefCell<CalendarModel>>,
    todo_model: &Rc<RefCell<crate::models::TodoModel>>,
) {
    let weak = ui.as_weak();
    let m = Rc::clone(model);
    ui.on_prev_month(move || {
        if let Some(ui) = weak.upgrade() {
            {
                m.borrow_mut().shift_month(-1);
            }
            refresh_calendar_ui(&ui, &m.borrow());
        }
    });

    let weak = ui.as_weak();
    let m = Rc::clone(model);
    ui.on_next_month(move || {
        if let Some(ui) = weak.upgrade() {
            {
                m.borrow_mut().shift_month(1);
            }
            refresh_calendar_ui(&ui, &m.borrow());
        }
    });

    let weak = ui.as_weak();
    let m = Rc::clone(model);
    ui.on_go_today(move || {
        if let Some(ui) = weak.upgrade() {
            {
                m.borrow_mut().go_today();
            }
            refresh_calendar_ui(&ui, &m.borrow());
        }
    });

    let weak = ui.as_weak();
    let m = Rc::clone(model);
    let tm = Rc::clone(todo_model);
    ui.on_day_clicked(move |year, month, day| {
        if let Some(ui) = weak.upgrade() {
            {
                let mut model = m.borrow_mut();
                model.select_day(year as isize, month as usize, day as usize);
            }
            refresh_calendar_ui(&ui, &m.borrow());

            // 跨模块：通知 todo 选中日期变更
            let date = format!("{:04}-{:02}-{:02}", year, month, day);
            {
                tm.borrow_mut().select_date(date);
            }
            super::todo::refresh_todo_ui(&ui, &tm.borrow());
        }
    });

    let weak = ui.as_weak();
    let m = Rc::clone(model);
    ui.on_week_start_changed(move |day| {
        if let Some(ui) = weak.upgrade() {
            {
                m.borrow_mut().week_start_day = day as usize;
            }
            refresh_calendar_ui(&ui, &m.borrow());
        }
    });
}

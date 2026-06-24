//! 视图模型层
//!
//! 负责内部数据模型与 Slint UI 类型之间的转换，
//! 以及注册所有 UI 交互回调（月份导航、日期选择、起始日切换）。

use std::cell::RefCell;
use std::rc::Rc;

use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::model::{self, DayCell, Ymd, get_weekday_names};
use crate::state::CalendarState;
use crate::{AppWindow, calendar_day, calendar_month, weekday_row};

/// 将内部 DayCell 模型转换为 Slint 端的 calendar_day 类型
fn to_calendar_day(day: DayCell) -> calendar_day {
    calendar_day {
        year: day.year as i32,
        month: day.month as i32,
        day: day.day as i32,
        is_today: day.is_today,
        is_selected: day.is_selected,
        is_weekend: day.is_weekend,
        is_current_month: day.is_current_month,
    }
}

/// 构建月份视图模型，供 Slint UI 渲染使用
///
/// - display: 当前显示的年月 (year, month)
/// - selected: 用户选中的日期
/// - today: 今天的日期
/// - week_start_day: 每周起始日（0=星期日，1=星期一）
pub fn build_month_vm(
    display: (isize, usize),
    selected: Ymd,
    today: Ymd,
    week_start_day: usize,
) -> calendar_month {
    let month_view = model::build_month_view(display.0, display.1, selected, today, week_start_day);
    calendar_month {
        year: month_view.year as i32,
        month: month_view.month as i32,
        days: ModelRc::new(VecModel::from(
            month_view
                .days
                .into_iter()
                .map(to_calendar_day)
                .collect::<Vec<_>>(),
        )),
    }
}

/// 设置 UI 中的星期标题行
pub fn set_weekdays_ui(ui: &AppWindow, start_day: usize) {
    let weekdays = get_weekday_names(start_day);
    ui.set_weekdays(weekday_row {
        label: ModelRc::new(VecModel::from(
            weekdays
                .iter()
                .map(|s| SharedString::from(*s))
                .collect::<Vec<_>>(),
        )),
    });
}

/// 刷新 UI 月份数据（从 CalendarState 读取当前状态并更新 UI）
fn refresh_month(ui: &AppWindow, vm: &Rc<RefCell<CalendarState>>) {
    let state = vm.borrow();
    ui.set_month_data(build_month_vm(
        state.display(),
        state.selected(),
        state.today(),
        state.week_start_day(),
    ));
}

/// 注册所有日历交互回调
///
/// - prev_month / next_month：月份导航
/// - go_today：跳转到今天
/// - day_clicked：点击选择日期
/// - week_start_changed：切换每周起始日
pub fn register_callbacks(ui: &AppWindow, vm: &Rc<RefCell<CalendarState>>) {
    // 切换到上一个月
    let weak = ui.as_weak();
    let vm_ref = Rc::clone(vm);
    ui.on_prev_month(move || {
        if let Some(ui) = weak.upgrade() {
            vm_ref.borrow_mut().prev_month();
            refresh_month(&ui, &vm_ref);
        }
    });

    // 切换到下一个月
    let weak = ui.as_weak();
    let vm_ref = Rc::clone(vm);
    ui.on_next_month(move || {
        if let Some(ui) = weak.upgrade() {
            vm_ref.borrow_mut().next_month();
            refresh_month(&ui, &vm_ref);
        }
    });

    // 跳转到今天
    let weak = ui.as_weak();
    let vm_ref = Rc::clone(vm);
    ui.on_go_today(move || {
        if let Some(ui) = weak.upgrade() {
            vm_ref.borrow_mut().go_today();
            refresh_month(&ui, &vm_ref);
        }
    });

    // 点击某一天
    let weak = ui.as_weak();
    let vm_ref = Rc::clone(vm);
    ui.on_day_clicked(move |year, month, day| {
        if let Some(ui) = weak.upgrade() {
            vm_ref
                .borrow_mut()
                .select_day(year as isize, month as usize, day as usize);
            refresh_month(&ui, &vm_ref);
        }
    });

    // 切换每周起始日（星期日/星期一）
    let weak = ui.as_weak();
    let vm_ref = Rc::clone(vm);
    ui.on_week_start_changed(move |day| {
        if let Some(ui) = weak.upgrade() {
            let start = day as usize;
            vm_ref.borrow_mut().set_week_start_day(start);
            set_weekdays_ui(&ui, start);
            refresh_month(&ui, &vm_ref);
        }
    });
}

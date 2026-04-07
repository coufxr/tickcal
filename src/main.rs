#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use slint::{ModelRc, SharedString, VecModel};

mod model;
mod view_model;

use model::{DayCell, MonthView};
use view_model::CalendarViewModel;

slint::include_modules!();

// View 层数据映射：Model -> Slint generated struct。
fn to_calendar_day(day: DayCell) -> calendar_day {
    calendar_day {
        year: day.year as i32,
        month: day.month as i32,
        day: day.day as i32,
        is_today: day.is_today,
        is_selected: day.is_selected,
        is_weekend: day.is_weekend,
    }
}

fn to_calendar_month(month: MonthView) -> calendar_month {
    calendar_month {
        year: month.year as i32,
        month: month.month as i32,
        days: ModelRc::new(VecModel::from(
            month
                .days
                .into_iter()
                .map(to_calendar_day)
                .collect::<Vec<_>>(),
        )),
    }
}

// View 层静态文案数据。
fn make_weekday_row() -> weekday_row {
    weekday_row {
        label: ModelRc::new(VecModel::from(vec![
            SharedString::from("日"),
            SharedString::from("一"),
            SharedString::from("二"),
            SharedString::from("三"),
            SharedString::from("四"),
            SharedString::from("五"),
            SharedString::from("六"),
        ])),
    }
}

// 从 ViewModel 拉取最新状态并刷新 UI。
fn refresh(ui: &AppWindow, vm: &Rc<RefCell<CalendarViewModel>>) {
    let month = vm.borrow().month_view();
    ui.set_month_data(to_calendar_month(month));
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let vm = Rc::new(RefCell::new(CalendarViewModel::new()));

    ui.set_weekdays(make_weekday_row());
    refresh(&ui, &vm);

    // 事件绑定：View -> ViewModel command -> 刷新 View。
    {
        let weak = ui.as_weak();
        let vm = Rc::clone(&vm);
        ui.on_prev_month(move || {
            if let Some(ui) = weak.upgrade() {
                vm.borrow_mut().prev_month();
                refresh(&ui, &vm);
            }
        });
    }

    {
        let weak = ui.as_weak();
        let vm = Rc::clone(&vm);
        ui.on_next_month(move || {
            if let Some(ui) = weak.upgrade() {
                vm.borrow_mut().next_month();
                refresh(&ui, &vm);
            }
        });
    }

    {
        let weak = ui.as_weak();
        let vm = Rc::clone(&vm);
        ui.on_go_today(move || {
            if let Some(ui) = weak.upgrade() {
                vm.borrow_mut().go_today();
                refresh(&ui, &vm);
            }
        });
    }

    {
        let weak = ui.as_weak();
        let vm = Rc::clone(&vm);
        ui.on_day_clicked(move |year, month, day| {
            if let Some(ui) = weak.upgrade() {
                vm.borrow_mut().select_day(year, month, day);
                refresh(&ui, &vm);
            }
        });
    }

    ui.run()?;

    Ok(())
}

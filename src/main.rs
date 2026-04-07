#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use chrono::Datelike;
use slint::{ModelRc, SharedString, VecModel};
use tyme4rs::tyme::Tyme;
use tyme4rs::tyme::solar::{SolarDay, SolarMonth};

slint::include_modules!();

fn today_ymd() -> (isize, usize, usize) {
    let today = chrono::Local::now().date_naive();
    (
        today.year() as isize,
        today.month() as usize,
        today.day() as usize,
    )
}

fn to_calendar_day(
    day: SolarDay,
    selected: (isize, usize, usize),
    today: (isize, usize, usize),
) -> calendar_day {
    let year = day.get_year();
    let month = day.get_month();
    let day_num = day.get_day();
    let week_index = day.get_week().get_index();

    calendar_day {
        year: year as i32,
        month: month as i32,
        day: day_num as i32,
        is_today: year == today.0 && month == today.1 && day_num == today.2,
        is_selected: year == selected.0 && month == selected.1 && day_num == selected.2,
        is_weekend: week_index == 0 || week_index == 6,
    }
}

fn build_calendar_month(
    year: isize,
    month: usize,
    selected: (isize, usize, usize),
    today: (isize, usize, usize),
) -> calendar_month {
    let current = SolarMonth::from_ym(year, month);
    let prev = current.next(-1);
    let next = current.next(1);

    let leading = SolarDay::from_ymd(year, month, 1).get_week().get_index();
    let mut days: Vec<calendar_day> = Vec::new();

    let prev_count = prev.get_day_count();
    for i in 0..leading {
        let d = prev_count - leading + i + 1;
        let solar = SolarDay::from_ymd(prev.get_year(), prev.get_month(), d);
        days.push(to_calendar_day(solar, selected, today));
    }

    let current_count = current.get_day_count();
    for d in 1..=current_count {
        let solar = SolarDay::from_ymd(year, month, d);
        days.push(to_calendar_day(solar, selected, today));
    }

    let trailing = 42usize.saturating_sub(days.len());
    for d in 1..=trailing {
        let solar = SolarDay::from_ymd(next.get_year(), next.get_month(), d);
        days.push(to_calendar_day(solar, selected, today));
    }

    calendar_month {
        year: year as i32,
        month: month as i32,
        days: ModelRc::new(VecModel::from(days)),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let today = today_ymd();

    ui.set_weekdays(weekday_row {
        label: ModelRc::new(VecModel::from(vec![
            SharedString::from("日"),
            SharedString::from("一"),
            SharedString::from("二"),
            SharedString::from("三"),
            SharedString::from("四"),
            SharedString::from("五"),
            SharedString::from("六"),
        ])),
    });

    let state = Rc::new(RefCell::new(((today.0, today.1), today)));
    {
        let (display, selected) = *state.borrow();
        ui.set_month_data(build_calendar_month(display.0, display.1, selected, today));
    }

    {
        let weak = ui.as_weak();
        let state = Rc::clone(&state);
        ui.on_prev_month(move || {
            if let Some(ui) = weak.upgrade() {
                let mut s = state.borrow_mut();
                let m = SolarMonth::from_ym(s.0.0, s.0.1).next(-1);
                s.0.0 = m.get_year();
                s.0.1 = m.get_month();
                ui.set_month_data(build_calendar_month(s.0.0, s.0.1, s.1, today));
            }
        });
    }

    {
        let weak = ui.as_weak();
        let state = Rc::clone(&state);
        ui.on_next_month(move || {
            if let Some(ui) = weak.upgrade() {
                let mut s = state.borrow_mut();
                let m = SolarMonth::from_ym(s.0.0, s.0.1).next(1);
                s.0.0 = m.get_year();
                s.0.1 = m.get_month();
                ui.set_month_data(build_calendar_month(s.0.0, s.0.1, s.1, today));
            }
        });
    }

    {
        let weak = ui.as_weak();
        let state = Rc::clone(&state);
        ui.on_go_today(move || {
            if let Some(ui) = weak.upgrade() {
                let mut s = state.borrow_mut();
                s.0 = (today.0, today.1);
                s.1 = today;
                ui.set_month_data(build_calendar_month(s.0.0, s.0.1, s.1, today));
            }
        });
    }

    {
        let weak = ui.as_weak();
        let state = Rc::clone(&state);
        ui.on_day_clicked(move |year, month, day| {
            if let Some(ui) = weak.upgrade() {
                let mut s = state.borrow_mut();
                s.1 = (year as isize, month.max(1) as usize, day.max(1) as usize);
                ui.set_month_data(build_calendar_month(s.0.0, s.0.1, s.1, today));
            }
        });
    }

    ui.run()?;

    Ok(())
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

mod model;

use crate::model::build_month_view;
use model::{DayCell, Ymd, get_weekday_names, today_ymd};

slint::include_modules!();

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

fn build_month_vm(
    display: (isize, usize),
    selected: Ymd,
    today: Ymd,
    week_start_day: usize,
) -> calendar_month {
    let month_view = build_month_view(display.0, display.1, selected, today, week_start_day);
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

fn set_weekdays_ui(ui: &AppWindow, start_day: usize) {
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

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let today = today_ymd();

    set_weekdays_ui(&ui, 0);
    ui.set_month_data(build_month_vm((today.0, today.1), today, today, 0));

    let vm = Rc::new(RefCell::new(CalendarState::new()));

    let weak = ui.as_weak();
    let vm_ref = Rc::clone(&vm);
    ui.on_prev_month(move || {
        if let Some(ui) = weak.upgrade() {
            vm_ref.borrow_mut().prev_month();
            ui.set_month_data(build_month_vm(
                vm_ref.borrow().display(),
                vm_ref.borrow().selected(),
                vm_ref.borrow().today(),
                vm_ref.borrow().week_start_day(),
            ));
        }
    });

    let weak = ui.as_weak();
    let vm_ref = Rc::clone(&vm);
    ui.on_next_month(move || {
        if let Some(ui) = weak.upgrade() {
            vm_ref.borrow_mut().next_month();
            ui.set_month_data(build_month_vm(
                vm_ref.borrow().display(),
                vm_ref.borrow().selected(),
                vm_ref.borrow().today(),
                vm_ref.borrow().week_start_day(),
            ));
        }
    });

    let weak = ui.as_weak();
    let vm_ref = Rc::clone(&vm);
    ui.on_go_today(move || {
        if let Some(ui) = weak.upgrade() {
            vm_ref.borrow_mut().go_today();
            ui.set_month_data(build_month_vm(
                vm_ref.borrow().display(),
                vm_ref.borrow().selected(),
                vm_ref.borrow().today(),
                vm_ref.borrow().week_start_day(),
            ));
        }
    });

    let weak = ui.as_weak();
    let vm_ref = Rc::clone(&vm);
    ui.on_day_clicked(move |year, month, day| {
        if let Some(ui) = weak.upgrade() {
            vm_ref
                .borrow_mut()
                .select_day(year as isize, month as usize, day as usize);
            ui.set_month_data(build_month_vm(
                vm_ref.borrow().display(),
                vm_ref.borrow().selected(),
                vm_ref.borrow().today(),
                vm_ref.borrow().week_start_day(),
            ));
        }
    });

    let weak = ui.as_weak();
    let vm_ref = Rc::clone(&vm);
    ui.on_week_start_changed(move |day| {
        if let Some(ui) = weak.upgrade() {
            let start = day as usize;
            vm_ref.borrow_mut().set_week_start_day(start);
            set_weekdays_ui(&ui, start);
            ui.set_month_data(build_month_vm(
                vm_ref.borrow().display(),
                vm_ref.borrow().selected(),
                vm_ref.borrow().today(),
                start,
            ));
        }
    });

    ui.run()?;

    Ok(())
}

struct CalendarState {
    display: (isize, usize),
    selected: Ymd,
    today: Ymd,
    week_start_day: usize,
}

impl CalendarState {
    fn new() -> Self {
        let today = today_ymd();
        Self {
            display: (today.0, today.1),
            selected: today,
            today,
            week_start_day: 0,
        }
    }

    fn display(&self) -> (isize, usize) {
        self.display
    }

    fn selected(&self) -> Ymd {
        self.selected
    }

    fn today(&self) -> Ymd {
        self.today
    }

    fn week_start_day(&self) -> usize {
        self.week_start_day
    }

    fn set_week_start_day(&mut self, day: usize) {
        self.week_start_day = day;
    }

    fn prev_month(&mut self) {
        self.shift_month(-1);
    }

    fn next_month(&mut self) {
        self.shift_month(1);
    }

    fn go_today(&mut self) {
        self.display = (self.today.0, self.today.1);
        self.selected = self.today;
    }

    fn select_day(&mut self, year: isize, month: usize, day: usize) {
        self.selected = (year, month.max(1), day.max(1));
    }

    fn shift_month(&mut self, offset: isize) {
        use tyme4rs::tyme::Tyme;
        use tyme4rs::tyme::solar::SolarMonth;
        let month = SolarMonth::from_ym(self.display.0, self.display.1).next(offset);
        self.display = (month.get_year(), month.get_month());
    }
}

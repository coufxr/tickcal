use chrono::Datelike;
use tyme4rs::tyme::Tyme;
use tyme4rs::tyme::solar::{SolarDay, SolarMonth};

pub type Ymd = (isize, usize, usize);

#[derive(Clone, Copy)]
pub struct DayCell {
    pub year: isize,
    pub month: usize,
    pub day: usize,
    pub is_today: bool,
    pub is_selected: bool,
    pub is_weekend: bool,
    pub is_current_month: bool,
}

pub struct MonthView {
    pub year: isize,
    pub month: usize,
    pub days: Vec<DayCell>,
}

pub fn today_ymd() -> Ymd {
    let today = chrono::Local::now().date_naive();
    (
        today.year() as isize,
        today.month() as usize,
        today.day() as usize,
    )
}

pub fn get_weekday_names(start_day: usize) -> Vec<&'static str> {
    let all = ["日", "一", "二", "三", "四", "五", "六"];
    let mut result = Vec::with_capacity(7);
    for i in 0..7 {
        result.push(all[(start_day + i) % 7]);
    }
    result
}

fn to_day_cell(
    day: SolarDay,
    selected: Ymd,
    today: Ymd,
    current_month: usize,
    current_year: isize,
) -> DayCell {
    let year = day.get_year();
    let month = day.get_month();
    let day_num = day.get_day();
    let week_index = day.get_week().get_index();

    DayCell {
        year,
        month,
        day: day_num,
        is_today: year == today.0 && month == today.1 && day_num == today.2,
        is_selected: year == selected.0 && month == selected.1 && day_num == selected.2,
        is_weekend: week_index == 0 || week_index == 6,
        is_current_month: year == current_year && month == current_month,
    }
}

pub fn build_month_view(
    year: isize,
    month: usize,
    selected: Ymd,
    today: Ymd,
    week_start_day: usize,
) -> MonthView {
    let current = SolarMonth::from_ym(year, month);
    let prev = current.next(-1);
    let next = current.next(1);

    let raw_index = SolarDay::from_ymd(year, month, 1).get_week().get_index();
    let leading = (raw_index + 7 - week_start_day) % 7;
    let mut days: Vec<DayCell> = Vec::new();

    let prev_count = prev.get_day_count();
    for i in 0..leading {
        let d = prev_count - leading + i + 1;
        let solar = SolarDay::from_ymd(prev.get_year(), prev.get_month(), d);
        days.push(to_day_cell(solar, selected, today, month, year));
    }

    let current_count = current.get_day_count();
    for d in 1..=current_count {
        let solar = SolarDay::from_ymd(year, month, d);
        days.push(to_day_cell(solar, selected, today, month, year));
    }

    let trailing = 42usize.saturating_sub(days.len());
    for d in 1..=trailing {
        let solar = SolarDay::from_ymd(next.get_year(), next.get_month(), d);
        days.push(to_day_cell(solar, selected, today, month, year));
    }

    MonthView { year, month, days }
}

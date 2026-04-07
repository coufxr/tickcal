use chrono::Datelike;
use tyme4rs::tyme::Tyme;
use tyme4rs::tyme::solar::{SolarDay, SolarMonth};

pub type Ymd = (isize, usize, usize);

#[derive(Clone, Copy)]
pub struct DayCell {
    // 具体公历日期（包含跨月补位的日期）
    pub year: isize,
    pub month: usize,
    pub day: usize,
    // 供 View 直接渲染状态样式
    pub is_today: bool,
    pub is_selected: bool,
    pub is_weekend: bool,
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

fn to_day_cell(day: SolarDay, selected: Ymd, today: Ymd) -> DayCell {
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
    }
}

pub fn build_month_view(year: isize, month: usize, selected: Ymd, today: Ymd) -> MonthView {
    let current = SolarMonth::from_ym(year, month);
    let prev = current.next(-1);
    let next = current.next(1);

    let leading = SolarDay::from_ymd(year, month, 1).get_week().get_index();
    let mut days: Vec<DayCell> = Vec::new();

    // 头部补齐上个月日期，确保第一行从周日开始。
    let prev_count = prev.get_day_count();
    for i in 0..leading {
        let d = prev_count - leading + i + 1;
        let solar = SolarDay::from_ymd(prev.get_year(), prev.get_month(), d);
        days.push(to_day_cell(solar, selected, today));
    }

    // 当前月日期。
    let current_count = current.get_day_count();
    for d in 1..=current_count {
        let solar = SolarDay::from_ymd(year, month, d);
        days.push(to_day_cell(solar, selected, today));
    }

    // 尾部补齐到固定 6 行 * 7 列，便于 UI 布局稳定。
    let trailing = 42usize.saturating_sub(days.len());
    for d in 1..=trailing {
        let solar = SolarDay::from_ymd(next.get_year(), next.get_month(), d);
        days.push(to_day_cell(solar, selected, today));
    }

    MonthView { year, month, days }
}

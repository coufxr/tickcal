use tyme4rs::tyme::Tyme;
use tyme4rs::tyme::solar::SolarMonth;

use crate::model::{MonthView, Ymd, build_month_view, today_ymd};

pub struct CalendarViewModel {
    // 当前界面显示的年月。
    display: (isize, usize),
    // 当前选中的具体日期（可为跨月日期）。
    selected: Ymd,
    // “今天”基准日期，应用运行期间固定。
    today: Ymd,
}

impl CalendarViewModel {
    pub fn new() -> Self {
        let today = today_ymd();
        Self {
            display: (today.0, today.1),
            selected: today,
            today,
        }
    }

    pub fn month_view(&self) -> MonthView {
        build_month_view(self.display.0, self.display.1, self.selected, self.today)
    }

    pub fn prev_month(&mut self) {
        self.shift_month(-1);
    }

    pub fn next_month(&mut self) {
        self.shift_month(1);
    }

    pub fn go_today(&mut self) {
        self.display = (self.today.0, self.today.1);
        self.selected = self.today;
    }

    pub fn select_day(&mut self, year: i32, month: i32, day: i32) {
        // 防御式兜底，避免 UI 传入无效 0 值。
        self.selected = (year as isize, month.max(1) as usize, day.max(1) as usize);
    }

    fn shift_month(&mut self, offset: isize) {
        let month = SolarMonth::from_ym(self.display.0, self.display.1).next(offset);
        self.display = (month.get_year(), month.get_month());
    }
}

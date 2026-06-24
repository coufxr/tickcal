//! 日历状态管理器
//!
//! 持有当前显示的年月、选中的日期、今天日期和每周起始日，
//! 所有 UI 回调通过 Rc<RefCell<CalendarState>> 共享同一份状态。

use crate::model::Ymd;

pub struct CalendarState {
    /// 当前显示的年月 (year, month)
    display: (isize, usize),
    /// 用户选中的日期 (year, month, day)
    selected: Ymd,
    /// 今天的日期
    today: Ymd,
    /// 每周起始日（0=星期日，1=星期一）
    week_start_day: usize,
}

impl CalendarState {
    pub fn new(week_start_day: usize, today: Ymd) -> Self {
        Self {
            display: (today.0, today.1),
            selected: today,
            today,
            week_start_day,
        }
    }

    pub fn display(&self) -> (isize, usize) {
        self.display
    }

    pub fn selected(&self) -> Ymd {
        self.selected
    }

    pub fn today(&self) -> Ymd {
        self.today
    }

    pub fn week_start_day(&self) -> usize {
        self.week_start_day
    }

    pub fn set_week_start_day(&mut self, day: usize) {
        self.week_start_day = day;
    }

    pub fn prev_month(&mut self) {
        self.shift_month(-1);
    }

    pub fn next_month(&mut self) {
        self.shift_month(1);
    }

    /// 跳转到今天所在的月份，并选中今天
    pub fn go_today(&mut self) {
        self.display = (self.today.0, self.today.1);
        self.selected = self.today;
    }

    /// 选中指定日期
    pub fn select_day(&mut self, year: isize, month: usize, day: usize) {
        self.selected = (year, month.max(1), day.max(1));
    }

    /// 偏移当前显示的月份（正数向后，负数向前）
    fn shift_month(&mut self, offset: isize) {
        use tyme4rs::tyme::Tyme;
        use tyme4rs::tyme::solar::SolarMonth;
        let month = SolarMonth::from_ym(self.display.0, self.display.1).next(offset);
        self.display = (month.get_year(), month.get_month());
    }
}

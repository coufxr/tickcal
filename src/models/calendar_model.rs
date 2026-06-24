//! 日历状态 + 业务逻辑

use std::collections::HashSet;

use crate::model::{Ymd, today_ymd};
use crate::settings::AppSettings;

/// 日历应用状态
pub struct CalendarModel {
    /// 当前显示的年月 (year, month)
    pub display: (isize, usize),
    /// 用户选中的日期 (year, month, day)
    pub selected: Ymd,
    /// 今天的日期
    pub today: Ymd,
    /// 每周起始日（0=星期日，1=星期一）
    pub week_start_day: usize,
    /// 有待办的日期集合（格式 "YYYY-MM-DD"）
    pub todo_dates: HashSet<String>,
}

impl CalendarModel {
    pub fn new() -> Self {
        let today = today_ymd();
        Self {
            display: (today.0, today.1),
            selected: today,
            today,
            week_start_day: 0,
            todo_dates: HashSet::new(),
        }
    }

    /// 偏移当前显示的月份（正数向后，负数向前）
    pub fn shift_month(&mut self, offset: isize) {
        use tyme4rs::tyme::Tyme;
        use tyme4rs::tyme::solar::SolarMonth;
        let month = SolarMonth::from_ym(self.display.0, self.display.1).next(offset);
        self.display = (month.get_year(), month.get_month());
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

    /// 应用设置
    pub fn apply_settings(&mut self, settings: AppSettings) {
        self.week_start_day = settings.week_start_day as usize;
    }

    /// 更新 todo 日期集合
    pub fn set_todo_dates(&mut self, dates: HashSet<String>) {
        self.todo_dates = dates;
    }
}

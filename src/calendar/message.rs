//! 日历消息（MVU Message）

use crate::settings::AppSettings;

/// 所有可能的 UI 事件
pub enum CalendarMessage {
    PrevMonth,
    NextMonth,
    GoToday,
    SelectDay {
        year: isize,
        month: usize,
        day: usize,
    },
    WeekStartChanged(usize),
    ApplySettings(AppSettings),
}

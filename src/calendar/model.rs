//! 日历状态（MVU Model）

use crate::model::Ymd;
use crate::model::today_ymd;

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
}

impl CalendarModel {
    pub fn new() -> Self {
        let today = today_ymd();
        Self {
            display: (today.0, today.1),
            selected: today,
            today,
            week_start_day: 0,
        }
    }
}

//! 日历数据模型
//!
//! 负责生成月视图的日期数据，包括上月补位、当月日期、下月补位，
//! 共 42 个格子（6 行 × 7 列）。

use chrono::Datelike;
use tyme4rs::tyme::Tyme;
use tyme4rs::tyme::solar::{SolarDay, SolarMonth};

/// 日期类型别名：(年, 月, 日)
pub type Ymd = (isize, usize, usize);

/// 单个日期格子的数据
#[derive(Clone, Copy)]
pub struct DayCell {
    pub year: isize,
    pub month: usize,
    pub day: usize,
    pub is_today: bool,
    pub is_selected: bool,
    pub is_weekend: bool,
    /// 是否属于当前显示的月份（用于区分上月/下月补位的灰色日期）
    pub is_current_month: bool,
}

/// 月份视图数据
pub struct MonthView {
    pub year: isize,
    pub month: usize,
    /// 固定 42 个格子（6 行 × 7 列）
    pub days: Vec<DayCell>,
}

/// 获取今天的日期
pub fn today_ymd() -> Ymd {
    let today = chrono::Local::now().date_naive();
    (
        today.year() as isize,
        today.month() as usize,
        today.day() as usize,
    )
}

/// 获取星期标题名称，根据起始日旋转顺序
///
/// - start_day=0: ["日", "一", "二", "三", "四", "五", "六"]
/// - start_day=1: ["一", "二", "三", "四", "五", "六", "日"]
pub fn get_weekday_names(start_day: usize) -> Vec<&'static str> {
    let all = ["日", "一", "二", "三", "四", "五", "六"];
    let mut result = Vec::with_capacity(7);
    for i in 0..7 {
        result.push(all[(start_day + i) % 7]);
    }
    result
}

/// 将 SolarDay 转换为 DayCell
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
        // week_index: 0=星期日, 6=星期六
        is_weekend: week_index == 0 || week_index == 6,
        is_current_month: year == current_year && month == current_month,
    }
}

/// 构建月视图数据
///
/// 生成固定 42 个格子的日历数据：
/// 1. 上月补位日期（灰色显示）
/// 2. 当月所有日期
/// 3. 下月补位日期（灰色显示）
///
/// week_start_day 决定第一列是星期几（0=星期日，1=星期一）
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

    // 计算当月 1 号是星期几，再根据起始日偏移得到前导空格数
    let raw_index = SolarDay::from_ymd(year, month, 1).get_week().get_index();
    let leading = (raw_index + 7 - week_start_day) % 7;
    let mut days: Vec<DayCell> = Vec::new();

    // 上月补位：填充当月 1 号之前的空格
    let prev_count = prev.get_day_count();
    for i in 0..leading {
        let d = prev_count - leading + i + 1;
        let solar = SolarDay::from_ymd(prev.get_year(), prev.get_month(), d);
        days.push(to_day_cell(solar, selected, today, month, year));
    }

    // 当月所有日期
    let current_count = current.get_day_count();
    for d in 1..=current_count {
        let solar = SolarDay::from_ymd(year, month, d);
        days.push(to_day_cell(solar, selected, today, month, year));
    }

    // 下月补位：凑满 42 个格子（6 行 × 7 列）
    let trailing = 42usize.saturating_sub(days.len());
    for d in 1..=trailing {
        let solar = SolarDay::from_ymd(next.get_year(), next.get_month(), d);
        days.push(to_day_cell(solar, selected, today, month, year));
    }

    MonthView { year, month, days }
}

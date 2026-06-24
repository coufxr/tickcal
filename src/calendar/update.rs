//! MVU Update 函数 + UI 刷新

use slint::{ModelRc, SharedString, VecModel};

use crate::model::{self, DayCell, get_weekday_names};
use crate::{AppWindow, calendar_day, calendar_month, weekday_row};

use super::message::CalendarMessage;
use super::model::CalendarModel;

/// 纯函数：处理消息，更新状态
pub fn update(model: &mut CalendarModel, message: CalendarMessage) {
    match message {
        CalendarMessage::PrevMonth => {
            shift_month(model, -1);
        }
        CalendarMessage::NextMonth => {
            shift_month(model, 1);
        }
        CalendarMessage::GoToday => {
            model.display = (model.today.0, model.today.1);
            model.selected = model.today;
        }
        CalendarMessage::SelectDay { year, month, day } => {
            model.selected = (year, month.max(1), day.max(1));
        }
        CalendarMessage::WeekStartChanged(day) => {
            model.week_start_day = day;
        }
        CalendarMessage::ApplySettings(settings) => {
            model.week_start_day = settings.week_start_day as usize;
        }
    }
}

/// 偏移当前显示的月份
fn shift_month(model: &mut CalendarModel, offset: isize) {
    use tyme4rs::tyme::Tyme;
    use tyme4rs::tyme::solar::SolarMonth;
    let month = SolarMonth::from_ym(model.display.0, model.display.1).next(offset);
    model.display = (month.get_year(), month.get_month());
}

/// 将 Model 状态推送到 Slint UI
pub fn refresh_ui(ui: &AppWindow, model: &CalendarModel) {
    // 更新月份视图
    let month_view = model::build_month_view(
        model.display.0,
        model.display.1,
        model.selected,
        model.today,
        model.week_start_day,
    );
    ui.set_month_data(calendar_month {
        year: month_view.year as i32,
        month: month_view.month as i32,
        days: ModelRc::new(VecModel::from(
            month_view
                .days
                .into_iter()
                .map(from_day_cell)
                .collect::<Vec<_>>(),
        )),
    });

    // 更新星期标题行
    let weekdays = get_weekday_names(model.week_start_day);
    ui.set_weekdays(weekday_row {
        label: ModelRc::new(VecModel::from(
            weekdays
                .iter()
                .map(|s| SharedString::from(*s))
                .collect::<Vec<_>>(),
        )),
    });
}

/// 将内部 DayCell 转换为 Slint 端的 calendar_day
fn from_day_cell(day: DayCell) -> calendar_day {
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

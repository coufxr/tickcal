//! ViewModel 层（MVVM）
//!
//! 负责日历的状态管理、数据转换和 UI 事件处理。
//! 作为 View（Slint）和 Model（model.rs）之间的桥梁。

use std::cell::RefCell;
use std::rc::Rc;

use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use tyme4rs::tyme::Tyme;
use tyme4rs::tyme::solar::SolarMonth;

use crate::model::{self, DayCell, Ymd, get_weekday_names, today_ymd};
use crate::settings::AppSettings;
use crate::{AppWindow, calendar_day, calendar_month, weekday_row};

/// 日历 ViewModel
///
/// 持有日历的所有状态，提供数据转换方法，并注册 UI 回调。
/// 通过 Rc<RefCell<CalendarViewModel>> 在多个回调间共享可变访问。
pub struct CalendarViewModel {
    /// 当前显示的年月 (year, month)
    display: (isize, usize),
    /// 用户选中的日期 (year, month, day)
    selected: Ymd,
    /// 今天的日期
    today: Ymd,
    /// 每周起始日（0=星期日，1=星期一）
    week_start_day: usize,
}

impl CalendarViewModel {
    /// 从设置创建 ViewModel，应用设置到 UI 并初始化视图
    pub fn from_settings(ui: &AppWindow, settings: &AppSettings) -> Rc<RefCell<Self>> {
        // 应用设置到 UI
        ui.invoke_apply_settings(
            settings.dark_mode,
            settings.accent_index,
            settings.week_start_day,
            settings.cell_size_index,
        );

        // 创建 ViewModel
        let start_day = settings.week_start_day as usize;
        let today = today_ymd();
        let vm = Rc::new(RefCell::new(Self {
            display: (today.0, today.1),
            selected: today,
            today,
            week_start_day: start_day,
        }));

        // 初始化 UI
        vm.borrow().set_weekdays_ui(ui);
        vm.borrow().refresh_ui(ui);

        vm
    }

    // ========== 状态变更（命令） ==========

    /// 切换到上一个月
    pub fn prev_month(&mut self) {
        self.shift_month(-1);
    }

    /// 切换到下一个月
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

    /// 切换每周起始日
    pub fn set_week_start_day(&mut self, day: usize) {
        self.week_start_day = day;
    }

    /// 偏移当前显示的月份（正数向后，负数向前）
    fn shift_month(&mut self, offset: isize) {
        let month = SolarMonth::from_ym(self.display.0, self.display.1).next(offset);
        self.display = (month.get_year(), month.get_month());
    }

    // ========== 数据转换（Model → View 类型） ==========

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

    /// 构建月份视图模型，供 Slint UI 渲染使用
    pub fn build_month_vm(&self) -> calendar_month {
        let month_view = model::build_month_view(
            self.display.0,
            self.display.1,
            self.selected,
            self.today,
            self.week_start_day,
        );
        calendar_month {
            year: month_view.year as i32,
            month: month_view.month as i32,
            days: ModelRc::new(VecModel::from(
                month_view
                    .days
                    .into_iter()
                    .map(Self::from_day_cell)
                    .collect::<Vec<_>>(),
            )),
        }
    }

    /// 设置 UI 中的星期标题行
    pub fn set_weekdays_ui(&self, ui: &AppWindow) {
        let weekdays = get_weekday_names(self.week_start_day);
        ui.set_weekdays(weekday_row {
            label: ModelRc::new(VecModel::from(
                weekdays
                    .iter()
                    .map(|s| SharedString::from(*s))
                    .collect::<Vec<_>>(),
            )),
        });
    }

    /// 刷新 UI 月份数据（从当前状态读取并更新 UI）
    pub fn refresh_ui(&self, ui: &AppWindow) {
        ui.set_month_data(self.build_month_vm());
    }

    // ========== UI 回调注册 ==========

    /// 注册所有日历交互回调到 UI
    ///
    /// - prev_month / next_month：月份导航
    /// - go_today：跳转到今天
    /// - day_clicked：点击选择日期
    /// - week_start_changed：切换每周起始日
    pub fn register_callbacks(ui: &AppWindow, vm: &Rc<RefCell<CalendarViewModel>>) {
        // 切换到上一个月
        let weak = ui.as_weak();
        let vm_ref = Rc::clone(vm);
        ui.on_prev_month(move || {
            if let Some(ui) = weak.upgrade() {
                vm_ref.borrow_mut().prev_month();
                vm_ref.borrow().refresh_ui(&ui);
            }
        });

        // 切换到下一个月
        let weak = ui.as_weak();
        let vm_ref = Rc::clone(vm);
        ui.on_next_month(move || {
            if let Some(ui) = weak.upgrade() {
                vm_ref.borrow_mut().next_month();
                vm_ref.borrow().refresh_ui(&ui);
            }
        });

        // 跳转到今天
        let weak = ui.as_weak();
        let vm_ref = Rc::clone(vm);
        ui.on_go_today(move || {
            if let Some(ui) = weak.upgrade() {
                vm_ref.borrow_mut().go_today();
                vm_ref.borrow().refresh_ui(&ui);
            }
        });

        // 点击某一天
        let weak = ui.as_weak();
        let vm_ref = Rc::clone(vm);
        ui.on_day_clicked(move |year, month, day| {
            if let Some(ui) = weak.upgrade() {
                vm_ref
                    .borrow_mut()
                    .select_day(year as isize, month as usize, day as usize);
                vm_ref.borrow().refresh_ui(&ui);
            }
        });

        // 切换每周起始日（星期日/星期一）
        let weak = ui.as_weak();
        let vm_ref = Rc::clone(vm);
        ui.on_week_start_changed(move |day| {
            if let Some(ui) = weak.upgrade() {
                let start = day as usize;
                vm_ref.borrow_mut().set_week_start_day(start);
                vm_ref.borrow().set_weekdays_ui(&ui);
                vm_ref.borrow().refresh_ui(&ui);
            }
        });
    }
}

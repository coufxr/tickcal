//! Task 模块：回调注册 + UI 刷新

use std::cell::RefCell;
use std::rc::Rc;

use slint::{ComponentHandle, Global, ModelRc, SharedString, VecModel};

use crate::models::{CalendarModel, TaskModel};
use crate::{AppWindow, TaskItem as SlintTaskItem, TaskState};

/// 将 Model 状态推送到 Slint UI 的 TaskState global
pub fn refresh_task_ui(ui: &AppWindow, model: &TaskModel) {
    let global = TaskState::get(ui);
    global.set_selected_date(SharedString::from(&model.selected_date));

    let current = model.current_items();
    log::debug!(
        "refresh_ui: selected_date={}, items_count={}",
        model.selected_date,
        current.len()
    );
    let items: Vec<SlintTaskItem> = current
        .into_iter()
        .map(|item| SlintTaskItem {
            id: item.id as i32,
            text: SharedString::from(&item.text),
            completed: item.completed,
        })
        .collect();
    global.set_task_items(ModelRc::new(VecModel::from(items)));
}

/// 注册所有 task 相关的 Slint 回调（通过 TaskState global）
pub fn register_task_callbacks(
    ui: &AppWindow,
    model: &Rc<RefCell<TaskModel>>,
    calendar_model: &Rc<RefCell<CalendarModel>>,
) {
    let global = TaskState::get(ui);

    let weak = ui.as_weak();
    let m = Rc::clone(model);
    let cm = Rc::clone(calendar_model);
    global.on_add_task(move |text| {
        super::with_ui(&weak, |ui| {
            let result = m.borrow_mut().add_task(text.to_string());
            if result.task_dates_changed {
                let dates = m.borrow().task_date_set();
                cm.borrow_mut().set_task_dates(dates);
                super::calendar::refresh_calendar_ui(ui, &cm.borrow());
            }
            if result.changed {
                refresh_task_ui(ui, &m.borrow());
            }
        });
    });

    let weak = ui.as_weak();
    let m = Rc::clone(model);
    global.on_toggle_task(move |id| {
        super::with_ui(&weak, |ui| {
            m.borrow_mut().toggle_task(id as u32);
            refresh_task_ui(ui, &m.borrow());
        });
    });

    let weak = ui.as_weak();
    let m = Rc::clone(model);
    let cm = Rc::clone(calendar_model);
    global.on_delete_task(move |id| {
        super::with_ui(&weak, |ui| {
            let result = m.borrow_mut().delete_task(id as u32);
            if result.task_dates_changed {
                let dates = m.borrow().task_date_set();
                cm.borrow_mut().set_task_dates(dates);
                super::calendar::refresh_calendar_ui(ui, &cm.borrow());
            }
            if result.changed {
                refresh_task_ui(ui, &m.borrow());
            }
        });
    });
}

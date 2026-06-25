//! Todo 模块：回调注册 + UI 刷新

use std::cell::RefCell;
use std::rc::Rc;

use slint::{ComponentHandle, Global, ModelRc, SharedString, VecModel};

use crate::models::{CalendarModel, TodoModel};
use crate::{AppWindow, TodoItem as SlintTodoItem, TodoState};

/// 将 Model 状态推送到 Slint UI 的 TodoState global
pub fn refresh_todo_ui(ui: &AppWindow, model: &TodoModel) {
    let global = TodoState::get(ui);
    global.set_selected_date(SharedString::from(&model.selected_date));

    let current = model.current_items();
    log::debug!(
        "refresh_ui: selected_date={}, items_count={}",
        model.selected_date,
        current.len()
    );
    let items: Vec<SlintTodoItem> = current
        .into_iter()
        .map(|item| SlintTodoItem {
            id: item.id as i32,
            text: SharedString::from(&item.text),
            completed: item.completed,
        })
        .collect();
    global.set_todo_items(ModelRc::new(VecModel::from(items)));
}

/// 注册所有 todo 相关的 Slint 回调（通过 TodoState global）
pub fn register_todo_callbacks(
    ui: &AppWindow,
    model: &Rc<RefCell<TodoModel>>,
    calendar_model: &Rc<RefCell<CalendarModel>>,
) {
    let global = TodoState::get(ui);

    let weak = ui.as_weak();
    let m = Rc::clone(model);
    let cm = Rc::clone(calendar_model);
    global.on_add_todo(move |text| {
        super::with_ui(&weak, |ui| {
            let result = m.borrow_mut().add_todo(text.to_string());
            if result.todo_dates_changed {
                let dates = m.borrow().todo_date_set();
                cm.borrow_mut().set_todo_dates(dates);
                super::calendar::refresh_calendar_ui(ui, &cm.borrow());
            }
            if result.changed {
                refresh_todo_ui(ui, &m.borrow());
            }
        });
    });

    let weak = ui.as_weak();
    let m = Rc::clone(model);
    global.on_toggle_todo(move |id| {
        super::with_ui(&weak, |ui| {
            m.borrow_mut().toggle_todo(id as u32);
            refresh_todo_ui(ui, &m.borrow());
        });
    });

    let weak = ui.as_weak();
    let m = Rc::clone(model);
    let cm = Rc::clone(calendar_model);
    global.on_delete_todo(move |id| {
        super::with_ui(&weak, |ui| {
            let result = m.borrow_mut().delete_todo(id as u32);
            if result.todo_dates_changed {
                let dates = m.borrow().todo_date_set();
                cm.borrow_mut().set_todo_dates(dates);
                super::calendar::refresh_calendar_ui(ui, &cm.borrow());
            }
            if result.changed {
                refresh_todo_ui(ui, &m.borrow());
            }
        });
    });
}

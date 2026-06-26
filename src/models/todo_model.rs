//! Todo 状态 + 业务逻辑
//!
//! 使用 SQLite 数据库持久化存储。

use std::collections::HashSet;
use std::sync::Arc;

use crate::db::Database;

/// 单个 Todo 项目
#[derive(Clone, Debug)]
pub struct TodoItem {
    pub id: u32,
    pub text: String,
    pub completed: bool,
}

/// 更新结果
pub struct UpdateResult {
    /// 模型是否变化（需要刷新 UI）
    pub changed: bool,
    /// todo 日期集合是否变化（需要通知日历模块）
    pub todo_dates_changed: bool,
}

/// Todo 应用状态
pub struct TodoModel {
    /// 数据库连接
    db: Arc<Database>,
    /// 当前选中日期的 todo 缓存
    current_items: Vec<TodoItem>,
    /// 当前选中的日期
    pub selected_date: String,
}

impl TodoModel {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            current_items: Vec::new(),
            selected_date: String::new(),
        }
    }

    /// 获取当前选中日期的 todo 列表
    pub fn current_items(&self) -> Vec<TodoItem> {
        self.current_items.clone()
    }

    /// 获取所有有待办的日期集合
    pub fn todo_date_set(&self) -> HashSet<String> {
        self.db.get_todo_dates()
    }

    /// 切换选中日期
    pub fn select_date(&mut self, date: String) -> UpdateResult {
        log::debug!("Todo: select_date = {}", date);
        self.selected_date = date.clone();
        self.current_items = self.db.load_by_date(&date);
        UpdateResult {
            changed: true,
            todo_dates_changed: false,
        }
    }

    /// 添加 todo
    pub fn add_todo(&mut self, text: String) -> UpdateResult {
        log::debug!(
            "Todo: add_todo = '{}', selected_date = '{}'",
            text,
            self.selected_date
        );
        if text.is_empty() || self.selected_date.is_empty() {
            log::debug!("Todo: add_todo 跳过（text 或 selected_date 为空）");
            return UpdateResult {
                changed: false,
                todo_dates_changed: false,
            };
        }

        if let Some(id) = self.db.insert_todo(&self.selected_date, &text) {
            self.current_items.push(TodoItem {
                id,
                text,
                completed: false,
            });
            UpdateResult {
                changed: true,
                todo_dates_changed: true,
            }
        } else {
            UpdateResult {
                changed: false,
                todo_dates_changed: false,
            }
        }
    }

    /// 切换完成状态
    pub fn toggle_todo(&mut self, id: u32) -> UpdateResult {
        if self.db.toggle_todo(id) {
            if let Some(item) = self.current_items.iter_mut().find(|i| i.id == id) {
                item.completed = !item.completed;
            }
            UpdateResult {
                changed: true,
                todo_dates_changed: false,
            }
        } else {
            UpdateResult {
                changed: false,
                todo_dates_changed: false,
            }
        }
    }

    /// 删除 todo
    pub fn delete_todo(&mut self, id: u32) -> UpdateResult {
        if self.db.delete_todo(id) {
            let before = self.current_items.len();
            self.current_items.retain(|i| i.id != id);
            let todo_dates_changed = self.current_items.is_empty() && before > 0;
            UpdateResult {
                changed: true,
                todo_dates_changed,
            }
        } else {
            UpdateResult {
                changed: false,
                todo_dates_changed: false,
            }
        }
    }
}

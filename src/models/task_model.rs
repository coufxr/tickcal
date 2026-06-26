//! Task 状态 + 业务逻辑
//!
//! 使用 SQLite 数据库持久化存储。

use std::collections::HashSet;
use std::sync::Arc;

use crate::db::Database;

/// 单个 Task 项目
#[derive(Clone, Debug)]
pub struct TaskItem {
    pub id: u32,
    pub text: String,
    pub completed: bool,
}

/// 更新结果
pub struct UpdateResult {
    /// 模型是否变化（需要刷新 UI）
    pub changed: bool,
    /// task 日期集合是否变化（需要通知日历模块）
    pub task_dates_changed: bool,
}

/// Task 应用状态
pub struct TaskModel {
    /// 数据库连接
    db: Arc<Database>,
    /// 当前选中日期的 task 缓存
    current_items: Vec<TaskItem>,
    /// 当前选中的日期
    pub selected_date: String,
}

impl TaskModel {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            current_items: Vec::new(),
            selected_date: String::new(),
        }
    }

    /// 获取当前选中日期的 task 列表
    pub fn current_items(&self) -> Vec<TaskItem> {
        self.current_items.clone()
    }

    /// 获取所有有待办的日期集合
    pub fn task_date_set(&self) -> HashSet<String> {
        self.db.get_task_dates()
    }

    /// 切换选中日期
    pub fn select_date(&mut self, date: String) -> UpdateResult {
        log::debug!("Task: select_date = {}", date);
        self.selected_date = date.clone();
        self.current_items = self.db.load_by_date(&date);
        UpdateResult {
            changed: true,
            task_dates_changed: false,
        }
    }

    /// 添加 task
    pub fn add_task(&mut self, text: String) -> UpdateResult {
        log::debug!(
            "Task: add_task = '{}', selected_date = '{}'",
            text,
            self.selected_date
        );
        if text.is_empty() || self.selected_date.is_empty() {
            log::debug!("Task: add_task 跳过（text 或 selected_date 为空）");
            return UpdateResult {
                changed: false,
                task_dates_changed: false,
            };
        }

        if let Some(id) = self.db.insert_task(&self.selected_date, &text) {
            self.current_items.push(TaskItem {
                id,
                text,
                completed: false,
            });
            UpdateResult {
                changed: true,
                task_dates_changed: true,
            }
        } else {
            UpdateResult {
                changed: false,
                task_dates_changed: false,
            }
        }
    }

    /// 切换完成状态
    pub fn toggle_task(&mut self, id: u32) -> UpdateResult {
        if self.db.toggle_task(id) {
            if let Some(item) = self.current_items.iter_mut().find(|i| i.id == id) {
                item.completed = !item.completed;
            }
            UpdateResult {
                changed: true,
                task_dates_changed: false,
            }
        } else {
            UpdateResult {
                changed: false,
                task_dates_changed: false,
            }
        }
    }

    /// 删除 task
    pub fn delete_task(&mut self, id: u32) -> UpdateResult {
        if self.db.delete_task(id) {
            let before = self.current_items.len();
            self.current_items.retain(|i| i.id != id);
            let task_dates_changed = self.current_items.is_empty() && before > 0;
            UpdateResult {
                changed: true,
                task_dates_changed,
            }
        } else {
            UpdateResult {
                changed: false,
                task_dates_changed: false,
            }
        }
    }
}

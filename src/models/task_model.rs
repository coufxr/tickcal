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
            self.current_items.retain(|i| i.id != id);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_test_model() -> TaskModel {
        let db = Arc::new(crate::db::Database::new_in_memory());
        TaskModel::new(db)
    }

    #[test]
    fn test_add_task() {
        let mut model = new_test_model();
        model.select_date("2026-01-01".into());

        let result = model.add_task("买菜".into());
        assert!(result.changed);
        assert!(result.task_dates_changed);
        assert_eq!(model.current_items().len(), 1);
        assert_eq!(model.current_items()[0].text, "买菜");
    }

    #[test]
    fn test_add_task_without_date() {
        let mut model = new_test_model();
        let result = model.add_task("买菜".into());
        assert!(!result.changed);
        assert!(!result.task_dates_changed);
    }

    #[test]
    fn test_add_empty_text() {
        let mut model = new_test_model();
        model.select_date("2026-01-01".into());

        let result = model.add_task("".into());
        assert!(!result.changed);
    }

    #[test]
    fn test_toggle_task() {
        let mut model = new_test_model();
        model.select_date("2026-01-01".into());
        model.add_task("任务".into());

        let id = model.current_items()[0].id;
        let result = model.toggle_task(id);
        assert!(result.changed);
        assert!(model.current_items()[0].completed);

        model.toggle_task(id);
        assert!(!model.current_items()[0].completed);
    }

    #[test]
    fn test_toggle_nonexistent_task() {
        let mut model = new_test_model();
        let result = model.toggle_task(999);
        assert!(!result.changed);
    }

    #[test]
    fn test_delete_task() {
        let mut model = new_test_model();
        model.select_date("2026-01-01".into());
        model.add_task("任务".into());

        let id = model.current_items()[0].id;
        let result = model.delete_task(id);
        assert!(result.changed);
        assert!(result.task_dates_changed);
        assert!(model.current_items().is_empty());
    }

    #[test]
    fn test_delete_nonexistent_task() {
        let mut model = new_test_model();
        let result = model.delete_task(999);
        assert!(!result.changed);
    }

    #[test]
    fn test_select_date_loads_items() {
        let mut model = new_test_model();
        model.select_date("2026-01-01".into());
        model.add_task("任务A".into());

        model.select_date("2026-02-15".into());
        assert!(model.current_items().is_empty());
        assert_eq!(model.selected_date, "2026-02-15");

        model.select_date("2026-01-01".into());
        assert_eq!(model.current_items().len(), 1);
    }

    #[test]
    fn test_task_date_set() {
        let mut model = new_test_model();
        model.select_date("2026-01-01".into());
        model.add_task("A".into());
        model.select_date("2026-02-15".into());
        model.add_task("B".into());

        let dates = model.task_date_set();
        assert_eq!(dates.len(), 2);
        assert!(dates.contains("2026-01-01"));
        assert!(dates.contains("2026-02-15"));
    }

    #[test]
    fn test_delete_last_task_clears_date() {
        let mut model = new_test_model();
        model.select_date("2026-01-01".into());
        model.add_task("唯一任务".into());

        let id = model.current_items()[0].id;
        let result = model.delete_task(id);
        assert!(result.task_dates_changed);
        assert!(model.task_date_set().is_empty());
    }

    #[test]
    fn test_delete_one_of_many_keeps_remaining() {
        let mut model = new_test_model();
        model.select_date("2026-01-01".into());
        model.add_task("任务A".into());
        model.add_task("任务B".into());
        model.add_task("任务C".into());

        let id_b = model.current_items()[1].id;
        let result = model.delete_task(id_b);

        assert!(result.changed);
        assert!(result.task_dates_changed);
        assert_eq!(model.current_items().len(), 2);
        assert_eq!(model.current_items()[0].text, "任务A");
        assert_eq!(model.current_items()[1].text, "任务C");
        assert!(model.task_date_set().contains("2026-01-01"));
    }

    #[test]
    fn test_toggle_task_from_other_date() {
        let mut model = new_test_model();
        model.select_date("2026-01-01".into());
        model.add_task("A".into());
        model.select_date("2026-02-15".into());
        model.add_task("B".into());

        let id_a = 1;
        let result = model.toggle_task(id_a);

        assert!(result.changed);
        assert_eq!(model.current_items().len(), 1);
        assert!(!model.current_items()[0].completed);
        model.select_date("2026-01-01".into());
        assert!(model.current_items()[0].completed);
    }
}

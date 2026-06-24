//! Todo 状态 + 业务逻辑

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::services::store;

/// 更新结果
pub struct UpdateResult {
    /// 模型是否变化（需要刷新 UI）
    pub changed: bool,
    /// todo 日期集合是否变化（需要通知日历模块）
    pub todo_dates_changed: bool,
}

/// 单个 Todo 项目
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: u32,
    pub text: String,
    pub completed: bool,
}

/// Todo 应用状态
pub struct TodoModel {
    /// 所有 todo 数据，key 为日期字符串 "YYYY-MM-DD"
    pub items: HashMap<String, Vec<TodoItem>>,
    /// 当前选中的日期
    pub selected_date: String,
    /// 下一个可用的 ID
    pub next_id: u32,
}

impl TodoModel {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            selected_date: String::new(),
            next_id: 1,
        }
    }

    /// 获取当前选中日期的 todo 列表
    pub fn current_items(&self) -> Vec<TodoItem> {
        self.items
            .get(&self.selected_date)
            .cloned()
            .unwrap_or_default()
    }

    /// 获取所有有待办的日期集合
    pub fn todo_date_set(&self) -> HashSet<String> {
        self.items
            .iter()
            .filter(|(_, items)| !items.is_empty())
            .map(|(date, _)| date.clone())
            .collect()
    }

    /// 切换选中日期
    pub fn select_date(&mut self, date: String) -> UpdateResult {
        log::debug!("Todo: select_date = {}", date);
        self.selected_date = date;
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
        let id = self.next_id;
        self.next_id += 1;
        let items = self.items.entry(self.selected_date.clone()).or_default();
        items.push(TodoItem {
            id,
            text,
            completed: false,
        });
        store::save(self);
        UpdateResult {
            changed: true,
            todo_dates_changed: true,
        }
    }

    /// 切换完成状态
    pub fn toggle_todo(&mut self, id: u32) -> UpdateResult {
        if let Some(items) = self.items.get_mut(&self.selected_date)
            && let Some(item) = items.iter_mut().find(|i| i.id == id)
        {
            item.completed = !item.completed;
            store::save(self);
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
        if let Some(items) = self.items.get_mut(&self.selected_date) {
            let before = items.len();
            items.retain(|i| i.id != id);
            if items.len() == before {
                return UpdateResult {
                    changed: false,
                    todo_dates_changed: false,
                };
            }
            let todo_dates_changed = items.is_empty();
            if todo_dates_changed {
                self.items.remove(&self.selected_date);
            }
            store::save(self);
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

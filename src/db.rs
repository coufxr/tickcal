//! SQLite 数据库模块
//!
//! 负责 Task 数据的持久化存储。

use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::{Connection, params};

use crate::models::TaskItem;
use crate::util;

/// 数据库连接包装
pub struct Database {
    conn: Mutex<Connection>,
}

#[cfg(test)]
impl Database {
    pub(crate) fn new_in_memory() -> Self {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(CREATE_TABLE_SQL).unwrap();
        Database {
            conn: Mutex::new(conn),
        }
    }
}

impl Database {
    /// 初始化数据库，创建表结构
    pub fn new() -> Self {
        let path = db_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(&path)
            .unwrap_or_else(|e| panic!("打开数据库失败 {}: {}", path.display(), e));

        conn.execute_batch(CREATE_TABLE_SQL).expect("创建表失败");

        Self {
            conn: Mutex::new(conn),
        }
    }

    /// 加载指定日期的 task 列表
    pub fn load_by_date(&self, date: &str) -> Vec<TaskItem> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, text, completed FROM tasks WHERE date = ?1 ORDER BY id")
            .unwrap();

        stmt.query_map(params![date], |row| {
            Ok(TaskItem {
                id: row.get(0)?,
                text: row.get(1)?,
                completed: row.get(2)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    /// 获取所有有 task 的日期集合
    pub fn get_task_dates(&self) -> std::collections::HashSet<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT DISTINCT date FROM tasks").unwrap();

        stmt.query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    /// 添加 task，返回新 ID
    pub fn insert_task(&self, date: &str, text: &str) -> Option<u32> {
        if text.is_empty() || date.is_empty() {
            return None;
        }

        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO tasks (date, text, completed) VALUES (?1, ?2, 0)",
            params![date, text],
        )
        .ok()?;

        Some(conn.last_insert_rowid() as u32)
    }

    /// 切换 task 完成状态
    pub fn toggle_task(&self, id: u32) -> bool {
        let conn = self.conn.lock().unwrap();
        let rows = conn
            .execute(
                "UPDATE tasks SET completed = NOT completed WHERE id = ?1",
                params![id],
            )
            .unwrap_or(0);

        rows > 0
    }

    /// 删除 task
    pub fn delete_task(&self, id: u32) -> bool {
        let conn = self.conn.lock().unwrap();
        let rows = conn
            .execute("DELETE FROM tasks WHERE id = ?1", params![id])
            .unwrap_or(0);

        rows > 0
    }
}

/// 获取数据库文件路径
fn db_path() -> PathBuf {
    util::config_dir().join("calendar.db")
}

const CREATE_TABLE_SQL: &str = "
    CREATE TABLE IF NOT EXISTS tasks (
        id INTEGER PRIMARY KEY,
        date TEXT NOT NULL,
        text TEXT NOT NULL,
        completed BOOLEAN NOT NULL DEFAULT 0
    );
    CREATE INDEX IF NOT EXISTS idx_tasks_date ON tasks(date);";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_load() {
        let db = Database::new_in_memory();
        let id = db.insert_task("2026-01-01", "买菜").unwrap();
        assert_eq!(id, 1);

        let items = db.load_by_date("2026-01-01");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].text, "买菜");
        assert!(!items[0].completed);
    }

    #[test]
    fn test_insert_empty_text_returns_none() {
        let db = Database::new_in_memory();
        assert!(db.insert_task("2026-01-01", "").is_none());
    }

    #[test]
    fn test_insert_empty_date_returns_none() {
        let db = Database::new_in_memory();
        assert!(db.insert_task("", "买菜").is_none());
    }

    #[test]
    fn test_load_by_date_returns_empty_for_unknown_date() {
        let db = Database::new_in_memory();
        let items = db.load_by_date("2099-01-01");
        assert!(items.is_empty());
    }

    #[test]
    fn test_get_task_dates() {
        let db = Database::new_in_memory();
        db.insert_task("2026-01-01", "任务A").unwrap();
        db.insert_task("2026-01-01", "任务B").unwrap();
        db.insert_task("2026-02-15", "任务C").unwrap();

        let dates = db.get_task_dates();
        assert_eq!(dates.len(), 2);
        assert!(dates.contains("2026-01-01"));
        assert!(dates.contains("2026-02-15"));
    }

    #[test]
    fn test_toggle_task() {
        let db = Database::new_in_memory();
        let id = db.insert_task("2026-01-01", "任务").unwrap();

        assert!(!db.load_by_date("2026-01-01")[0].completed);
        assert!(db.toggle_task(id));
        assert!(db.load_by_date("2026-01-01")[0].completed);
        assert!(db.toggle_task(id));
        assert!(!db.load_by_date("2026-01-01")[0].completed);
    }

    #[test]
    fn test_toggle_nonexistent_task() {
        let db = Database::new_in_memory();
        assert!(!db.toggle_task(999));
    }

    #[test]
    fn test_delete_task() {
        let db = Database::new_in_memory();
        let id = db.insert_task("2026-01-01", "任务").unwrap();
        assert!(db.delete_task(id));
        assert!(db.load_by_date("2026-01-01").is_empty());
    }

    #[test]
    fn test_delete_nonexistent_task() {
        let db = Database::new_in_memory();
        assert!(!db.delete_task(999));
    }

    #[test]
    fn test_ids_are_sequential() {
        let db = Database::new_in_memory();
        let id1 = db.insert_task("2026-01-01", "A").unwrap();
        let id2 = db.insert_task("2026-01-01", "B").unwrap();
        let id3 = db.insert_task("2026-01-02", "C").unwrap();
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }
}

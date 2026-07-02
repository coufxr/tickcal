//! SQLite 数据库模块
//!
//! 负责 Task 数据的持久化存储和应用设置管理。

use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::{Connection, params};

use crate::models::TaskItem;
use crate::platform;
use crate::settings::AppSettings;

/// 数据库连接包装
pub struct Database {
    conn: Mutex<Connection>,
}

#[cfg(test)]
impl Database {
    pub(crate) fn new_in_memory() -> Self {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(CREATE_TABLE_SQL).unwrap();
        conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])
            .unwrap();
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
        conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])
            .expect("创建设置行失败");

        Self {
            conn: Mutex::new(conn),
        }
    }

    /// 从数据库加载设置
    pub fn load_settings(&self) -> AppSettings {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT dark_mode, accent_index, week_start_day, cell_size_index FROM settings WHERE id = 1",
            [],
            |row| {
                Ok(AppSettings {
                    dark_mode: row.get::<_, i32>(0)? != 0,
                    accent_index: row.get(1)?,
                    week_start_day: row.get(2)?,
                    cell_size_index: row.get(3)?,
                })
            },
        )
        .unwrap_or_else(|e| {
            log::error!("加载设置失败: {}", e);
            AppSettings::default()
        })
    }

    /// 将设置保存到数据库
    pub fn save_settings(&self, settings: &AppSettings) {
        let conn = self.conn.lock().unwrap();
        if let Err(e) = conn.execute(
            "UPDATE settings SET dark_mode = ?1, accent_index = ?2, week_start_day = ?3, cell_size_index = ?4 WHERE id = 1",
            params![
                settings.dark_mode as i32,
                settings.accent_index,
                settings.week_start_day,
                settings.cell_size_index,
            ],
        ) {
            log::error!("保存设置失败: {}", e);
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
    platform::data_dir().join(format!("{}.db", platform::app_name()))
}

const CREATE_TABLE_SQL: &str = "
    CREATE TABLE IF NOT EXISTS tasks (
        id INTEGER PRIMARY KEY,
        date TEXT NOT NULL,
        text TEXT NOT NULL,
        completed BOOLEAN NOT NULL DEFAULT 0
    );
    CREATE INDEX IF NOT EXISTS idx_tasks_date ON tasks(date);

    CREATE TABLE IF NOT EXISTS settings (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        dark_mode INTEGER NOT NULL DEFAULT 0,
        accent_index INTEGER NOT NULL DEFAULT 0,
        week_start_day INTEGER NOT NULL DEFAULT 0,
        cell_size_index INTEGER NOT NULL DEFAULT 1
    );";

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

    #[test]
    fn test_default_settings() {
        let db = Database::new_in_memory();
        let s = db.load_settings();
        assert!(!s.dark_mode);
        assert_eq!(s.accent_index, 0);
        assert_eq!(s.week_start_day, 0);
        assert_eq!(s.cell_size_index, 1);
    }

    #[test]
    fn test_save_and_load_settings() {
        let db = Database::new_in_memory();
        let s = AppSettings {
            dark_mode: true,
            accent_index: 5,
            week_start_day: 1,
            cell_size_index: 2,
        };
        db.save_settings(&s);
        let loaded = db.load_settings();
        assert!(loaded.dark_mode);
        assert_eq!(loaded.accent_index, 5);
        assert_eq!(loaded.week_start_day, 1);
        assert_eq!(loaded.cell_size_index, 2);
    }
}

//! SQLite 数据库模块
//!
//! 负责 Todo 数据的持久化存储。

use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::{Connection, params};

use crate::models::TodoItem;
use crate::util;

/// 数据库连接包装
pub struct Database {
    conn: Mutex<Connection>,
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

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS todos (
                id INTEGER PRIMARY KEY,
                date TEXT NOT NULL,
                text TEXT NOT NULL,
                completed BOOLEAN NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_todos_date ON todos(date);",
        )
        .expect("创建表失败");

        Self {
            conn: Mutex::new(conn),
        }
    }

    /// 加载指定日期的 todo 列表
    pub fn load_by_date(&self, date: &str) -> Vec<TodoItem> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, text, completed FROM todos WHERE date = ?1 ORDER BY id")
            .unwrap();

        stmt.query_map(params![date], |row| {
            Ok(TodoItem {
                id: row.get(0)?,
                text: row.get(1)?,
                completed: row.get(2)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    /// 获取所有有 todo 的日期集合
    pub fn get_todo_dates(&self) -> std::collections::HashSet<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT DISTINCT date FROM todos").unwrap();

        stmt.query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }

    /// 添加 todo，返回新 ID
    pub fn insert_todo(&self, date: &str, text: &str) -> Option<u32> {
        if text.is_empty() || date.is_empty() {
            return None;
        }

        let conn = self.conn.lock().unwrap();

        let id: u32 = conn
            .query_row("SELECT COALESCE(MAX(id), 0) + 1 FROM todos", [], |row| {
                row.get(0)
            })
            .unwrap_or(1);

        conn.execute(
            "INSERT INTO todos (id, date, text, completed) VALUES (?1, ?2, ?3, 0)",
            params![id, date, text],
        )
        .ok()?;

        Some(id)
    }

    /// 切换 todo 完成状态
    pub fn toggle_todo(&self, id: u32) -> bool {
        let conn = self.conn.lock().unwrap();
        let rows = conn
            .execute(
                "UPDATE todos SET completed = NOT completed WHERE id = ?1",
                params![id],
            )
            .unwrap_or(0);

        rows > 0
    }

    /// 删除 todo
    pub fn delete_todo(&self, id: u32) -> bool {
        let conn = self.conn.lock().unwrap();
        let rows = conn
            .execute("DELETE FROM todos WHERE id = ?1", params![id])
            .unwrap_or(0);

        rows > 0
    }
}

/// 获取数据库文件路径
fn db_path() -> PathBuf {
    util::config_dir().join("calendar.db")
}

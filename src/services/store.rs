//! 数据持久化（JSON 文件）

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::models::{TodoItem, TodoModel};

/// 获取 todo 存储文件路径
fn store_path() -> PathBuf {
    let dir = if cfg!(debug_assertions) {
        PathBuf::from(".")
    } else {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("calendar")
    };
    dir.join("todos.json")
}

/// 可序列化的存储格式
#[derive(serde::Serialize, serde::Deserialize)]
struct StoreData {
    items: HashMap<String, Vec<TodoItem>>,
    next_id: u32,
}

/// 从磁盘加载 todo 数据
pub fn load(model: &mut TodoModel) {
    let path = store_path();
    if !path.exists() {
        return;
    }
    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<StoreData>(&content) {
            Ok(data) => {
                model.items = data.items;
                model.next_id = data.next_id;
            }
            Err(e) => {
                log::error!("Todo 文件解析失败 {}: {}", path.display(), e);
            }
        },
        Err(e) => {
            log::error!("Todo 文件读取失败 {}: {}", path.display(), e);
        }
    }
}

/// 将 todo 数据保存到磁盘
pub fn save(model: &TodoModel) {
    let path = store_path();
    if let Some(parent) = path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        log::error!("创建 todo 存储目录失败: {}", e);
        return;
    }
    let data = StoreData {
        items: model.items.clone(),
        next_id: model.next_id,
    };
    match serde_json::to_string_pretty(&data) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json) {
                log::error!("保存 todo 到 {} 失败: {}", path.display(), e);
            }
        }
        Err(e) => {
            log::error!("Todo 序列化失败: {}", e);
        }
    }
}

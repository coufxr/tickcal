//! 应用设置持久化模块
//!
//! 负责将用户设置（深色模式、主题色、起始星期、单元格大小）保存为 TOML 文件，
//! 并在启动时加载恢复。

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::util;

/// 获取设置文件路径
fn settings_path() -> PathBuf {
    util::config_dir().join("settings.toml")
}

/// 应用设置结构体，与 TOML 文件字段一一对应
///
/// 注意：字段顺序和命名需与 Slint 端 `apply_settings` 回调参数保持一致
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub dark_mode: bool,
    pub accent_index: i32,
    pub week_start_day: i32,
    pub cell_size_index: i32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            dark_mode: false,
            accent_index: 0,
            week_start_day: 0,
            cell_size_index: 1, // 默认中等大小
        }
    }
}

impl AppSettings {
    /// 校验并修正设置值，确保在合法范围内
    ///
    /// - accent_index: 0..=11，与 light-palette.slint / dark-palette.slint 中 brand_bg_options 数组长度一致
    /// - week_start_day: 0..=6（星期日=0，星期一=1，...，星期六=6）
    /// - cell_size_index: 0..=2（小=0，中=1，大=2）
    fn validate(mut self) -> Self {
        self.accent_index = self.accent_index.clamp(0, 11);
        self.week_start_day = self.week_start_day.clamp(0, 6);
        self.cell_size_index = self.cell_size_index.clamp(0, 2);
        self
    }
}

/// 从磁盘加载设置
///
/// - 文件不存在：返回默认值（首次运行）
/// - 文件损坏/解析失败：记录错误日志，返回默认值（不覆盖原文件）
/// - 加载成功：校验后返回
pub fn load() -> AppSettings {
    let path = settings_path();
    if !path.exists() {
        return AppSettings::default();
    }
    match fs::read_to_string(&path) {
        Ok(content) => match toml::from_str::<AppSettings>(&content) {
            Ok(settings) => settings.validate(),
            Err(e) => {
                log::error!("设置文件解析失败 {}: {}", path.display(), e);
                AppSettings::default()
            }
        },
        Err(e) => {
            log::error!("设置文件读取失败 {}: {}", path.display(), e);
            AppSettings::default()
        }
    }
}

/// 将设置保存到磁盘
///
/// 保存前会先校验值范围，确保写入的都是合法值。
/// 自动创建配置目录（首次保存时目录可能不存在）。
pub fn save(settings: &AppSettings) {
    let settings = settings.clone().validate();
    let path = settings_path();
    if let Some(parent) = path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        log::error!("创建配置目录失败: {}", e);
        return;
    }
    match toml::to_string_pretty(&settings) {
        Ok(toml_str) => {
            if let Err(e) = fs::write(&path, &toml_str) {
                log::error!("保存设置到 {} 失败: {}", path.display(), e);
            }
        }
        Err(e) => {
            log::error!("设置序列化失败: {}", e);
        }
    }
}

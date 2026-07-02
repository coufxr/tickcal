//! 应用设置持久化模块
//!
//! 负责将用户设置（深色模式、主题色、起始星期、单元格大小）存入数据库，
//! 并在启动时加载恢复。

use crate::db::Database;

/// 应用设置结构体
///
/// 注意：字段顺序和命名需与 Slint 端 `apply_settings` 回调参数保持一致
#[derive(Clone, Debug)]
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
            cell_size_index: 1,
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

/// 从数据库加载设置
pub fn load(db: &Database) -> AppSettings {
    db.load_settings().validate()
}

/// 将设置保存到数据库
pub fn save(settings: AppSettings, db: &Database) {
    db.save_settings(&settings.validate());
}

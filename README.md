# Calendar

基于 Slint + Rust 的日历待办应用，遵循 Fluent 2 设计风格。

## 项目结构

```
├── src/
│   ├── main.rs              # 入口
│   ├── app_logic/           # 应用逻辑层
│   │   ├── mod.rs           # init() 聚合入口
│   │   ├── calendar.rs      # 日历回调 + UI 刷新
│   │   └── todo.rs          # Todo 回调 + UI 刷新
│   ├── models/              # 数据模型层
│   │   ├── calendar_model.rs
│   │   └── todo_model.rs
│   ├── services/            # 服务层
│   │   └── store.rs         # JSON 持久化
│   ├── model.rs             # 共享类型 (Ymd, DayCell, MonthView)
│   ├── settings.rs          # 设置读写
│   └── lifespan.rs          # 启动/关闭生命周期
├── ui/
│   ├── app-window.slint     # 根组件
│   ├── globals/
│   │   └── todo-state.slint # TodoState Global
│   ├── components/          # 可复用组件
│   │   ├── calendar-*.slint
│   │   ├── todo-*.slint
│   │   ├── menu-bar.slint
│   │   └── settings-dialog.slint
│   ├── types/               # 数据结构定义
│   ├── theme/               # Fluent 2 设计令牌
│   ├── config/              # 布局配置
│   └── icons/               # SVG 图标
└── Cargo.toml
```

## 架构特点

- **声明式 UI**：`.slint` 文件仅负责布局和数据绑定
- **属性绑定**：响应式数据流，避免手动同步
- **Global 单例**：`Fluent2Palette`、`CalendarSpacingConfig`、`CalendarSettings`、`TodoState`
- **组件组合**：`in`/`out`/`in-out` 属性 + `callback` 事件流
- **SVG 图标**：所有非文字图形使用 SVG + `colorize` 适配主题

## 运行

```bash
cargo run
```

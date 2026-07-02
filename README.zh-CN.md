# Calendar

[![CI](https://github.com/USER/calendar/actions/workflows/ci.yml/badge.svg)](https://github.com/USER/calendar/actions/workflows/ci.yml)
[![GPLv3 License](LICENSE)](LICENSE)

[English](README.md) | **中文**

基于 [Slint](https://slint.dev) + Rust 的日历待办应用，遵循 Fluent 2 设计风格。

> 该项目为个人练手项目，仅供参考和学习交流。

## 功能

- **月视图日历** — 支持月份切换与日期选择
- **待办事项管理** — 按日期组织，支持增删改
- **系统托盘** — 后台常驻，快速访问
- **跨平台** — Windows / Linux / macOS
- **深色模式** — 自适应 Fluent 2 主题

## 快速开始

### 前置依赖

- Rust 工具链（推荐 [rustup.rs](https://rustup.rs)）

**Linux 额外依赖：**

```bash
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev \
  libappindicator3-dev librsvg2-dev
```

### 运行

```bash
cargo run
```

## 构建

### 当前平台

```bash
cargo build --release
```

### 交叉编译

```bash
# Windows (MSVC)
cargo build --release --target x86_64-pc-windows-msvc

# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# macOS Intel / Apple Silicon
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

也可通过 `.cargo/config.toml` 中定义的别名简化：

```bash
cargo release-win
cargo release-linux
cargo release-mac
cargo release-mac-arm
```

## 配置目录

| 平台      | Debug 模式 | Release 模式                                |
|---------|----------|-------------------------------------------|
| Windows | 项目根目录    | `%APPDATA%\calendar\`                     |
| Linux   | 项目根目录    | `~/.config/calendar/`                     |
| macOS   | 项目根目录    | `~/Library/Application Support/calendar/` |

## 项目结构

```
src/
├── main.rs              # 入口
├── platform.rs          # 平台抽象层
├── db.rs                # SQLite 数据库
├── settings.rs          # TOML 设置持久化
├── lifespan.rs          # 启动/关闭生命周期
├── model.rs             # 共享类型
├── app_logic/           # ViewModel 层
│   ├── mod.rs
│   ├── calendar.rs
│   └── task.rs
├── models/              # Model 层
│   ├── calendar_model.rs
│   └── task_model.rs
ui/
├── app-window.slint     # 根组件
├── components/          # 可复用组件
│   ├── common/
│   ├── calendar/
│   └── task/
├── dialogs/
├── types/
├── theme/               # Fluent 2 设计令牌
├── config/              # 布局配置
└── icons/               # SVG 图标
.cargo/config.toml       # 跨平台编译配置
.github/workflows/       # CI/CD
```

## 发布

推送 Git tag 触发自动构建与发布：

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions 将生成平台安装包并上传至 Release。

## 贡献

欢迎提交 Issue 和 Pull Request。

## 许可

[GNU General Public License v3.0](LICENSE)

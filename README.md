# TickCal

[![CI](https://github.com/USER/tickcal/actions/workflows/ci.yml/badge.svg)](https://github.com/USER/tickcal/actions/workflows/ci.yml)
[![GPLv3 License](LICENSE)](LICENSE)

**English** | [中文](README.zh-CN.md)

A Fluent 2 style calendar and todo app built with [Slint](https://slint.dev) + Rust.

> Personal practice project. For reference and learning only.

## Features

- **Monthly calendar view** — Month switching and date selection
- **Todo management** — Grouped by date, CRUD support
- **System tray** — Background resident, quick access
- **Cross-platform** — Windows / Linux / macOS
- **Dark mode** — Adaptive Fluent 2 theme

## Quick Start

### Prerequisites

- Rust toolchain (recommended via [rustup.rs](https://rustup.rs))

**Linux extra dependencies:**

```bash
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev \
  libappindicator3-dev librsvg2-dev
```

### Run

```bash
cargo run
```

## Build

### Current platform

```bash
cargo build --release
```

### Cross-compilation

```bash
# Windows (MSVC)
cargo build --release --target x86_64-pc-windows-msvc

# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# macOS Intel / Apple Silicon
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

Or use aliases defined in `.cargo/config.toml`:

```bash
cargo release-win
cargo release-linux
cargo release-mac
cargo release-mac-arm
```

## Config Directory

| Platform | Debug        | Release                           |
|----------|--------------|-----------------------------------|
| Windows  | Project root | `%APPDATA%\tickcal\`              |
| Linux    | Project root | `~/.config/tickcal/`              |
| macOS    | Project root | `~/Library/Application Support/tickcal/` |

## Project Structure

```
src/
├── main.rs              # Entry
├── platform.rs          # Platform abstraction
├── db.rs                # SQLite database
├── settings.rs          # TOML settings
├── lifespan.rs          # Startup/shutdown lifecycle
├── model.rs             # Shared types
├── app_logic/           # ViewModel layer
│   ├── mod.rs
│   ├── calendar.rs
│   └── task.rs
├── models/              # Model layer
│   ├── calendar_model.rs
│   └── task_model.rs
ui/
├── app-window.slint     # Root component
├── components/          # Reusable components
│   ├── common/
│   ├── calendar/
│   └── task/
├── dialogs/
├── types/
├── theme/               # Fluent 2 design tokens
├── config/              # Layout config
└── icons/               # SVG icons
.cargo/config.toml       # Cross-platform build config
.github/workflows/       # CI/CD
```

## Release

Push a Git tag to trigger automatic build and release:

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions will generate platform packages and upload them to Release.

## Contributing

Issues and Pull Requests are welcome.

## License

[GNU General Public License v3.0](LICENSE)

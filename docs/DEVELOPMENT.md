# BarTap 开发指南

## 环境

- Rust stable，`x86_64-pc-windows-msvc`。
- Visual Studio C++ 构建工具。
- Windows 10 / 11 SDK。
- 打包安装程序时需要 NSIS 3；可通过 `scripts/package.ps1 -NsisCompiler` 指定 `makensis.exe`。

## 构建与测试

```powershell
rustup default stable-x86_64-pc-windows-msvc
cargo build --locked --bin bartap
cargo test --locked --all-targets --features record -- --test-threads=1
```

测试按单线程运行，因为部分测试通过环境变量切换独立数据目录。低内存设备可以设置：

```powershell
$env:CARGO_BUILD_JOBS = "1"
```

开发实例使用独立数据目录，避免覆盖日常设置：

```powershell
$env:BARTAP_DATA_DIR = Join-Path $env:TEMP 'bartap-development'
cargo run --locked --bin bartap
```

## 目录

| 路径 | 内容 |
| --- | --- |
| `src/main.rs` | Windows 外壳、系统状态和后台服务衔接 |
| `src/app.rs`、`src/pages.rs` | 应用状态、悬浮岛与页面绘制 |
| `src/settings_theme.rs` | ThemeManager、Fluent Token、系统主题读取 |
| `src/skin_paint.rs` | 岛体内容的 Light / Dark 投影 |
| `src/extras.rs` | 设置页、通知队列和共享控件 |
| `src/calendar*` | 日历 |
| `src/quota*`、provider 模块 | AI 额度与连接 |
| `src/native.rs`、`src/system.rs` | 原生窗口、输入、拖放和系统事件 |
| `src/ui_tests.rs` | 无窗口交互、布局和主题回归测试 |
| `src/bin/bartap-record` | 合成数据演示与软件渲染器 |
| `vendor/` | 少量必要的上游补丁 |
| `installer/`、`scripts/` | 安装、卸载与打包 |

## 主题架构

BarTap 的界面颜色只有一个来源：`src/settings_theme.rs` 中的 `ThemeTokens`。

- `ThemeManager` 解析 `System / Light / Dark`，读取 Windows `AppsUseLightTheme`。
- `apply()` 把 Fluent `egui::Visuals` 应用到设置页、更新弹窗等 viewport。
- 悬浮岛页面按统一深色基准绘制，再由 `skin_paint` 投影到 Fluent Light/Dark；不要为单页再写一套浅色常量。
- 主题角色包括 `window_bg`、`card_bg`、`secondary_bg`、`text`、`text_secondary`、`border`、`accent`、控件四态和状态色。
- 品牌色（AI 服务标识、微信/支付宝、专辑封面）不参与主题换色。

新增 UI 时必须：

1. 优先使用 `settings_theme::current()` 的 Token。
2. 不新增页面级 Light/Dark 双份颜色。
3. 新 viewport 在首帧应用主题，避免深色背景上出现白色输入框。
4. 为浅色、深色和跟随系统各检查一次可读性。

完整规范见 [DESIGN.md](DESIGN.md)。

## 测试辅助

常用检查：

```powershell
cargo test --locked --features record -- --test-threads=1 settings_theme
cargo test --locked --features record -- --test-threads=1 theme_manager
cargo test --locked --features record -- --test-threads=1 page_visibility
```

需要人工查看的截图测试默认标记为 `#[ignore]`，通过环境变量指定导出目录，例如：

```powershell
$env:BARTAP_THEME_PREVIEW_DIR = Join-Path $env:TEMP 'bartap-settings'
$env:BARTAP_FLUENT_PREVIEW_DIR = Join-Path $env:TEMP 'bartap-islands'
cargo test --locked --features record -- --ignored --test-threads=1 settings_theme_follows_system
cargo test --locked --features record -- --ignored --test-threads=1 fluent_island_gallery
```

真实账号、真实日历和联网额度测试不要放在公共 CI 中运行。

## 命名

BarTap 使用 `bartap` 作为 Cargo 包名、lib 名和可执行文件名，用户数据目录为 `%APPDATA%\BarTap`，安装目录为 `%LOCALAPPDATA%\Programs\BarTap`。

修改名称时至少需要同步 `Cargo.toml`、`app.rc`、`installer/`、`scripts/`、单实例窗口、托盘提示、设置标题和测试中的二进制名。

## 提交与反馈

BarTap 目前是个人维护项目，不设公开贡献流程。Issue 中请提供：

- BarTap 版本、Windows 版本和屏幕缩放。
- 主题模式与背景材质。
- 可复现步骤和脱敏日志。

不要提交真实账号、授权码、Cookie、凭据文件或私人日程。

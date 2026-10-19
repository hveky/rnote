# CLAUDE.md — Rnote

Rnote 是一款用 Rust 编写的、基于 GTK4/libadwaita 的矢量手写笔记应用。跨平台（Linux/Windows/macOS），主入口仓库：https://github.com/flxzt/rnote 。

## 构建 / 测试命令

构建采用 **meson + just**（`justfile` 封装 meson），不是纯 cargo。首次配置：

```bash
just setup-build              # meson setup _mesonbuild（默认 debug）
# 或 meson setup _mesonbuild -Dbuildtype=release
```

常用：

```bash
just build                    # 编译（UI + CLI）
just lint                     # clippy（经 meson: ui-cargo-clippy / cli-cargo-clippy）+ yamllint
just test                     # meson test + cargo-test
just run                      # 运行应用
```

纯 cargo 也可用于快速迭代单个 crate：`cargo build -p rnote-engine`、`cargo nextest run -p rnote-engine`（CI 用 nextest）。注意 rnote-ui 依赖 meson 生成的 `config.rs`/gresource，需 meson 构建目录才能编译。

Windows/MSYS2 构建：`just msys2-setup` 等 recipe（见 BUILDING.md）。

## Workspace 结构（4 crates）

```
crates/rnote-compose   形状/路径/样式的构建块（不依赖 GTK 渲染）
crates/rnote-engine    核心引擎：Engine 结构体是主入口；store、pens、导入导出、渲染
crates/rnote-cli       rnote-cli 命令行工具：create/export/import/test/thumbnail 子命令
crates/rnote-ui        GTK4/libadwaita 界面（基于 gtk4-rs），composite templates (.ui XML + blp)
```

依赖方向：`rnote-ui` / `rnote-cli` → `rnote-engine` → `rnote-compose`。

## 关键模块地图

### rnote-compose
- `shapes/` — Shape 类型：line、arrow、rectangle、ellipse、cubbez/quadbez、polygon、polyline（`shapeable.rs` 定义 trait）
- `style/` — 样式系统：`smooth`、`rough`（rough-js 风格手绘）、`textured`（纹理笔刷）；`composer.rs`
- `penpath/` — 笔输入路径；`penevent.rs` — PenEvent / EventResult 输入模型
- `builders/` — 从笔路径构建形状

### rnote-engine（核心，入口 `engine::Engine`）
- `engine/` — Engine 本体：rendering、import、export、snapshot、animation、strokecontent、visual_debug
- `store/` — StrokeStore（component 风格：stroke_comp、render_comp、selection_comp、trash_comp、chrono_comp、keytree）
- `pens/` — 笔逻辑：brush、shaper、typewriter、eraser、selector、tools；`penholder.rs` 状态机；`pensconfig/`
- `strokes/` — StrokeContent：brushstroke、shapestroke、textstroke、vectorimage、bitmapimage
- `document/` — Document：布局、背景、格式（纸张尺寸）
- `fileformats/` — `.rnote`（自有格式，目录内子模块）与 xoppformat（Xournal++ 互操作）
- `camera.rs`、`snap.rs`、`tasks.rs`（后台任务）、`widgetflags.rs`（engine→UI 信号标志）

### rnote-ui
- `appwindow/` — AppWindow（imp.rs 为 GTK composite template 子类），actions.rs 集中定义 gio actions
- `canvas/` — RnoteCanvas widget（canvaslayout.rs、input.rs、imexport.rs）
- `penssidebar/` — 每支笔一个设置页（brushpage、shaperpage、typewriterpage、eraserpage、selectorpage、toolspage）
- `dialogs/`、`workspacebrowser/`、`settingspanel/`、`colorpicker/`、`strokewidthpicker/`
- UI 模板：`.ui` XML 由 blueprint (`.blp`) 编译；资源经 gresource 打包（`data/resources.gresource.xml`）
- settings 经 gschema（`data/app.gschema.xml.in`）

## 约定

- Rust edition 2024，MSRV 1.89；workspace 依赖集中在根 `Cargo.toml`
- 代码风格：`rustfmt.toml`；模块文档注释齐全（`#![warn(missing_docs)]` 于 compose crate）
- GTK 部件采用 libadwaita（`adw` crate）+ gtk4-rs，template 子类模式：`mod.rs`（公共）+ `imp.rs`（TemplateChild 等）
- 引擎与 UI 解耦：engine 通过 `WidgetFlags` 与 UI 通信，不直接依赖 GTK widget（仅 `ui` feature 下有少量绑定）
- 文件兼容性回归：`just test-file-compatibility`（用 `misc/file-tests/*.rnote`）

## 提交 / PR

遵循 conventional commits（feat/fix/chore），见 git log。主分支为 `main`。

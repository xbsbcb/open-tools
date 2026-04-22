# Step 1 构建总结 — Tauri 骨架

**完成时间**: 2026-04-22  
**Git 提交**: `853472d` (骨架) + `40ced92` (构建修复)  
**最终构建**: `Finished dev profile [unoptimized + debuginfo] target(s) in 10.72s` ✅

---

## 已完成的组件

### Rust 主进程 (`src-tauri/src/`)

| 文件 | 功能 | 状态 |
|------|------|------|
| `lib.rs` | Tauri Builder，注册全部插件和命令 | ✅ |
| `tray.rs` | 系统托盘，Show/Quit 菜单，左键切换窗口 | ✅ |
| `shortcut.rs` | 全局快捷键 Alt+Space 唤起/隐藏主窗口 | ✅ |
| `sidecar.rs` | Deno 进程启动，stdout JSON-lines 转 Tauri 事件，`ping_sidecar` 命令 | ✅ |
| `db.rs` | SQLite migration v1：`plugins` 表 + `settings` 表 | ✅ |
| `commands/search.rs` | `search_apps`：扫描 `~/.local/share/applications` + `/usr/share/applications`，解析 `.desktop`，返回最多 20 条 | ✅ |
| `commands/open.rs` | `open_path`：解析 `Exec=` 字段，剥离 `%F/%u` 等占位符，fallback `xdg-open` | ✅ |

### Vue 渲染进程 (`src/`)

| 文件 | 功能 | 状态 |
|------|------|------|
| `main.ts` | 挂载 Vue + Pinia + Ant Design Vue | ✅ |
| `App.vue` | 根组件，Escape 键隐藏窗口，居中布局 | ✅ |
| `components/SearchBox.vue` | 640×52px 输入框，150ms 防抖，毛玻璃效果，`invoke('ping_sidecar')` IPC 验证 | ✅ |
| `components/ResultList.vue` | 结果列表，最多 8 条，↑↓/Enter 键盘导航，`invoke('open_path')` 启动应用 | ✅ |
| `store/app.ts` | Pinia store，`search(q)` → `invoke('search_apps', { query: q })` | ✅ |

### 基础设施

| 文件 | 功能 | 状态 |
|------|------|------|
| `src-tauri/tauri.conf.json` | 窗口 480×320，无装饰，透明，隐藏启动，`externalBin: ["binaries/deno"]` | ✅ |
| `src-tauri/capabilities/default.json` | 全部 Tauri 权限声明 | ✅ |
| `src-tauri/Cargo.toml` | `tauri 2` (tray-icon feature) + 4 个插件 crate | ✅ |
| `plugin-host/main.ts` | Deno sidecar 占位，stdin JSON-lines echo | ✅ |
| `scripts/download-deno.sh` | 自动检测平台 triple，下载 deno v1.44.4 | ✅ |
| `scripts/build-sidecar.sh` | `deno compile` → `src-tauri/binaries/deno-<triple>` | ✅ |
| `src-tauri/icons/*.png` | 32×32 / 128×128 / 128×128@2x，RGBA (color_type=6) | ✅ |

---

## 构建期间修复的错误

### 错误 1：Tauri 权限名称变更
- **现象**: `Error: Permission shell:allow-stdin not found`
- **原因**: Tauri 2.x 将该权限重命名
- **修复**: `capabilities/default.json` → `shell:allow-stdin-write`

### 错误 2：托盘图标功能门控
- **现象**: `error[E0433]: failed to resolve: use of undeclared crate or module 'tray'`
- **原因**: `tauri::tray` 模块需要 Cargo feature 才解锁
- **修复**: `Cargo.toml` → `tauri = { version = "2", features = ["tray-icon"] }`

### 错误 3：API 重命名
- **现象**: `error[E0277]` on `menu_on_left_click`
- **原因**: Tauri 2.x API 重命名
- **修复**: `tray.rs` → `show_menu_on_left_click(false)`

### 错误 4：错误类型不兼容
- **现象**: `error[E0277]: ?` couldn't convert the error to `tauri::Error`
- **原因**: `GlobalShortcutExt::on_shortcut` 返回插件自身的 `Result`，非 `tauri::Result`
- **修复**: `shortcut.rs` → `.map_err(|e| tauri::Error::Anyhow(e.into()))`

### 错误 5：Icon 文件不存在
- **现象**: `generate_context!()` panic — icon file not found
- **原因**: `tauri.conf.json` 中引用了 `icon.icns` / `icon.ico` 但未创建
- **修复**: 移除这两个引用，只保留三个 PNG

### 错误 6：Icon 格式错误（RGB vs RGBA）
- **现象**: `generate_context!()` panic — icon must be RGBA
- **原因**: 首次生成的 PNG 为 RGB（color_type=2），Tauri 要求 RGBA（color_type=6）
- **修复**: 用 Python `struct.pack` 重新生成，每像素 4 字节（R G B A）

---

## IPC 通路验证

```
[搜索框输入] → store.search(q) → invoke('search_apps', { query })
                                    ↓ (Rust)
                                    扫描 .desktop → Vec<SearchResult>
                                    ↓
                              ResultList 渲染图标 + 名称

[任意输入]  → invoke('ping_sidecar', { message })
                                    ↓ (Rust)
                                    "pong: {message}"

[Alt+Space] → toggle_main_window → 窗口 show/hide + focus
[Escape]    → app.hide()
[托盘左键]  → toggle_main_window
[托盘 Quit] → app.exit(0)
```

---

## 下一步 (Step 2)

- [ ] `search_apps` 补充 macOS (`/Applications/`) 和 Windows Start Menu 实现
- [ ] 从 `Icon=` 字段解析系统主题图标路径（hicolor theme）
- [ ] 文件搜索集成（`~` 目录 + 常用路径）
- [ ] 搜索结果分组（应用 / 文件 / 计算器 / …）
- [ ] 窗口位置跟随当前屏幕（多显示器支持）

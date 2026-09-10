# AGENTS.md

本文件面向 AI 编码代理，介绍本项目的架构、构建方式与开发约定。

## 项目概览

**lizl-work-manager（lizl 待办管理）**：一款基于 **Tauri 2 + Vue 3 + Rust + SQLite** 的桌面待办事项管理应用（Windows 桌面端），支持项目管理、任务优先级、富文本笔记（Vditor Markdown 编辑器）、桌面小窗模式、窗口置顶等功能。数据完全本地化（SQLite，WAL 模式），无云端依赖。

主要功能详见 `README.md`（中文），含应用截图 `docs/screenshot.png`。

## 技术栈

| 层 | 技术 |
|---|---|
| 前端 | Vue 3（`<script setup>` SFC）+ TypeScript + Vite + Pinia + Tailwind CSS 4 + Vditor + lucide-vue-next |
| 桌面端 | Tauri 2（含 `tauri-plugin-shell`） |
| 后端 | Rust（独立 crate `todo-core`，2021 edition） |
| 数据库 | SQLite（rusqlite 0.31 bundled，WAL 模式，外键开启，busy_timeout 5000ms） |
| 包管理 | pnpm（pnpm-workspace，`apps/*`）+ Cargo workspace |

## 仓库结构

```
├── Cargo.toml               # Cargo workspace 根（members: crates/todo-core, src-tauri）
├── package.json             # pnpm 根，脚本: dev/build/tauri（均指向 tauri CLI）
├── pnpm-workspace.yaml      # packages: apps/*
├── apps/desktop/            # Vue 3 前端（package name: "desktop"）
│   └── src/
│       ├── App.vue, main.ts, style.css
│       ├── components/      # Sidebar, TaskList, TaskItem, QuickInput, MiniMode, TitleBar, GlobalOverlays
│       ├── stores/          # Pinia：taskStore.ts, projectStore.ts, uiStore.ts
│       ├── lib/             # api.ts（Tauri invoke 封装）、window.ts（窗口控制：置顶/小窗模式）
│       └── types/task.ts    # 与 Rust 模型对应的 TypeScript 类型
├── crates/todo-core/        # Rust 核心库（无 Tauri 依赖，可独立测试）
│   └── src/
│       ├── lib.rs           # 公共导出
│       ├── error.rs         # TodoError（thiserror）：DatabaseError/ValidationError/NotFound/Conflict
│       ├── models/          # task.rs, project.rs
│       └── storage/
│           ├── db.rs        # Database 结构体，封装所有 SQL 操作
│           └── schema.rs    # 版本化迁移（schema_migrations 表，当前 version 3）
├── src-tauri/               # Tauri 壳（package name: "lizl-work-manager"）
│   ├── src/lib.rs           # run()：初始化 Database、注册命令；AppState { db: Mutex<Database> }
│   ├── src/commands/        # task_cmd.rs, project_cmd.rs（#[tauri::command] 薄封装）
│   ├── capabilities/default.json  # 主窗口权限（窗口控制、shell 等）
│   └── tauri.conf.json      # 无边框窗口（decorations: false），1000x700，identifier com.lizl.workmanager
├── data/                    # SQLite 数据文件（tasks.db 生产 / tasks.dev.db 开发，git 忽略）
└── docs/screenshot.png
```

## 构建与运行命令

```bash
# 安装依赖
pnpm install

# 开发模式（同时启动 Vite dev server :5173 和 Tauri 窗口）
pnpm tauri dev          # 或 pnpm dev

# 仅前端构建（含 vue-tsc 类型检查）
pnpm --dir apps/desktop build

# 打包发布
pnpm tauri build

# Rust 侧检查
cargo check             # workspace 根目录
cargo build
```

注意：
- 修改 `tauri.conf.json` 或 `capabilities/` 权限文件后，必须**完全重启** `pnpm tauri dev` 才会生效（热更新不会重载）。
- `apps/desktop` 的 `build` 脚本会先跑 `vue-tsc -b`，因此构建即类型检查。项目没有配置 ESLint/Prettier。

## 运行时架构

- **进程模型**：Tauri 2 单进程模型 —— WebView 渲染前端，Rust 侧通过 `invoke` 命令处理业务。
- **状态管理**：`AppState { db: Mutex<Database> }` 通过 `app.manage()` 注入，所有命令用 `state.db.lock().unwrap()` 串行访问 SQLite（单连接 + Mutex，不是连接池）。
- **数据库路径**：
  - 开发（`debug_assertions`）：仓库根 `data/tasks.dev.db`
  - 发布：`app_data_dir()/data/tasks.db`
- **数据库迁移**：`storage/schema.rs` 用 `schema_migrations` 表做线性版本迁移（当前到 version 3）。新增 schema 变更时追加一个 `if version < N { ... }` 块，不要修改已有块。
- **软删除**：任务删除是软删除（`deleted_at` 时间戳），支持撤销恢复。
- **默认项目**：首次启动自动创建收件箱（inbox，`is_default = 1`，数据库用部分唯一索引保证只有一个默认项目）。删除项目时任务自动移回收件箱。

## 前后端接口约定

- 前端所有后端调用集中在 `apps/desktop/src/lib/api.ts`，通过 `@tauri-apps/api/core` 的 `invoke` 调用；命令参数名用 camelCase（如 `taskId`、`includeArchived`），对应 Rust 参数的 snake_case（Tauri 自动转换）。
- TypeScript 类型（`apps/desktop/src/types/task.ts`）需与 Rust 模型（`crates/todo-core/src/models/`）手动保持同步 —— 改一侧必须改另一侧。
- `TaskStatus` 在 Rust 中 `serde(rename_all = "snake_case")`，序列化值为 `'todo' | 'in_progress' | 'completed'`；但 `update_task_status` 命令接收 `String` 并在 Rust 侧 `FromStr` 解析。
- 错误传递：Rust 侧 `TodoError` 在 `src-tauri/src/commands/mod.rs` 转为 `CommandError { code, message }`（code 为 `VALIDATION_ERROR` / `NOT_FOUND` / `CONFLICT` / `STORAGE_ERROR`），以结构化对象 reject 到前端 Promise。
- `priority` 是 0–3 的整数（0=无, 1=低, 2=中, 3=高），数据库有 CHECK 约束。

## 代码风格

- **前端**：Vue 3 `<script setup lang="ts">` SFC；Pinia store 用 `xxxStore.ts` 命名放 `src/stores/`；可复用逻辑放 `src/lib/`；样式用 Tailwind CSS 4（通过 `@tailwindcss/vite` 插件，入口 `src/style.css` 含 Vditor 覆盖样式）。
- **Rust**：标准 rustfmt 默认风格；错误统一用 `thiserror` 定义的 `TodoError`，crate 内用 `crate::error::Result<T>`；序列化字段用 snake_case。
- **文档与提交信息使用中文**（README、git 提交历史均为中文）；代码内注释中英混用。
- `src-tauri/src/lib.rs` 中新增命令后需在 `invoke_handler` 的 `generate_handler!` 列表中注册。

## 测试

- 项目目前**没有任何测试**（无 Rust `#[test]`、无前端测试框架、无 CI 配置）。如新增测试：Rust 侧用 `cargo test`（`todo-core` 的 `Database` 可独立于 Tauri 实例化，适合单测）；前端尚未配置测试框架，不要擅自引入。
- 验证改动的主要方式：`cargo check`（Rust）+ `pnpm --dir apps/desktop build`（前端类型检查与构建）+ `pnpm tauri dev` 手动验证。

## 安全注意事项

- 数据库文件（`data/*.db*`）包含用户真实数据，已 git 忽略；不要提交、不要读取生产库 `data/tasks.db` 用于调试（开发用 `tasks.dev.db`）。
- `tauri.conf.json` 中 `csp: null`（未启用 CSP）——这是已知现状，改动安全策略需谨慎评估对 Vditor 等依赖的影响。
- 新窗口能力必须在 `src-tauri/capabilities/default.json` 中显式声明权限，否则前端调用会被拒绝。
- 任务 `attachments` 字段存粘贴图片数据（base64 于 description/attachments 中），注意数据体积。
- 不要运行具有破坏性的 git 命令（push、reset --hard 等），除非用户明确要求。

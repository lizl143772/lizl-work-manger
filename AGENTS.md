# AGENTS.md

本文件面向 AI 编码代理，介绍本项目的架构、构建方式与开发约定。阅读本文件前不需要了解项目的任何背景。

## 项目概览

**lizl-work-manager（lizl 待办管理）**：一款基于 **Tauri 2 + Vue 3 + Rust + SQLite** 的 Windows 桌面待办事项管理应用，数据完全本地化（SQLite，WAL 模式），无云端依赖、离线可用。

主要功能（详见 `README.md` 与截图 `docs/screenshot.png`）：

- 任务管理：待办聚合列表、状态页签（全部/未开始/进行中）、优先级循环切换（无/低/中/高）、右键菜单、Vditor 所见即所得 Markdown 笔记、Ctrl+V 粘贴图片附件、时间规划与自动耗时计算、删除后 5 秒内可撤销
- 多视图：任务列表、**日历视图**（按天聚合创建/完成活跃度）、**回收站**（软删除任务管理，支持按保留期自动清理）、**统计视图**、**设置视图**、已完成列表
- 项目管理：8 种图标 × 7 种颜色，右键编辑/删除（任务自动移回收件箱），侧栏实时未完成任务数
- 窗口框架：无边框窗口 + 自定义标题栏、窗口置顶（状态持久化）、380×560 桌面小窗模式（尺寸可在设置页自定义）

## 技术栈

| 层 | 技术 |
|---|---|
| 前端 | Vue 3（`<script setup lang="ts">` SFC）+ TypeScript + Vite + Pinia + Tailwind CSS 4 + Vditor + marked + lucide-vue-next |
| 桌面端 | Tauri 2（含 `tauri-plugin-shell`） |
| 后端 | Rust（独立 crate `todo-core`，2021 edition） |
| 数据库 | SQLite（rusqlite 0.31 bundled，WAL 模式，外键开启，busy_timeout 5000ms） |
| 包管理 | pnpm（pnpm-workspace，`apps/*`）+ Cargo workspace（`crates/todo-core`、`src-tauri`） |

## 仓库结构

```
├── Cargo.toml               # Cargo workspace 根（members: crates/todo-core, src-tauri）
├── package.json             # pnpm 根，脚本: dev/build/tauri（均指向 tauri CLI）
├── pnpm-workspace.yaml      # packages: apps/*（含一条占位性质的 allowBuilds 配置）
├── apps/desktop/            # Vue 3 前端（package name: "desktop"）
│   └── src/
│       ├── App.vue, main.ts, style.css
│       ├── components/      # Sidebar, TaskList, TaskItem, QuickInput, MiniMode, TitleBar,
│       │                    # GlobalOverlays, CalendarView, TrashView, StatsView, SettingsView,
│       │                    # DateRangePicker（HelloWorld.vue 为脚手架遗留）
│       ├── stores/          # Pinia：taskStore.ts, projectStore.ts, uiStore.ts, settingsStore.ts
│       ├── lib/             # api.ts（Tauri invoke 封装）、window.ts（置顶/小窗模式）、duration.ts（耗时格式化）
│       └── types/task.ts    # 与 Rust 模型对应的 TS 类型 + 日期范围/排序等前端常量
├── crates/todo-core/        # Rust 核心库（无 Tauri 依赖，可独立实例化 Database）
│   └── src/
│       ├── lib.rs           # 公共导出
│       ├── error.rs         # TodoError（thiserror）：DatabaseError/ValidationError/NotFound/Conflict
│       ├── models/          # task.rs（Task/TaskQuery/DailyActivity/TaskStats 等）, project.rs
│       └── storage/
│           ├── db.rs        # Database 结构体，封装所有 SQL 操作
│           └── schema.rs    # 版本化迁移（schema_migrations 表，当前 version 7）
├── src-tauri/               # Tauri 壳（package name: "lizl-work-manager"）
│   ├── src/lib.rs           # run()：初始化 Database、注册命令；AppState { db: Mutex<Database>, db_path }
│   ├── src/commands/        # task_cmd.rs, project_cmd.rs, app_cmd.rs（#[tauri::command] 薄封装）
│   ├── capabilities/default.json  # 主窗口权限（窗口控制、shell 等）
│   └── tauri.conf.json      # 无边框窗口（decorations: false），1000x700（最小 800x600），
│                            # identifier com.lizl.workmanager
├── data/                    # SQLite 数据文件（tasks.db 生产 / tasks.dev.db 开发，git 忽略）
└── docs/screenshot.png
```

## 构建与运行命令

```bash
# 安装依赖
pnpm install

# 开发模式（自动启动 Vite dev server :5173 和 Tauri 窗口）
pnpm tauri dev          # 或 pnpm dev

# 仅前端构建（先跑 vue-tsc -b 类型检查，再 vite build）
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
- `tauri.conf.json` 的 `beforeDevCommand`/`beforeBuildCommand` 已配置为自动执行前端 dev/build，无需手动另起 Vite。

## 运行时架构

- **进程模型**：Tauri 2 单进程模型 —— WebView 渲染前端，Rust 侧通过 `invoke` 命令处理业务。
- **状态管理**：`AppState { db: Mutex<Database>, db_path: PathBuf }` 通过 `app.manage()` 注入，所有命令用 `state.db.lock().unwrap()` 串行访问 SQLite（单连接 + Mutex，不是连接池）。
- **数据库路径**：
  - 开发（`debug_assertions`）：`../data/tasks.dev.db`（相对 src-tauri 工作目录，即仓库根 `data/tasks.dev.db`）
  - 发布：`app_data_dir()/data/tasks.db`
- **数据库迁移**：`storage/schema.rs` 用 `schema_migrations` 表做线性版本迁移（当前到 **version 7**：v2 attachments、v3 time_spent、v4 完成时间索引、v5 回收站索引、v6 started_at、v7 历史数据回填）。新增 schema 变更时追加一个 `if version < N { ... }` 块，不要修改已有块。
- **软删除与回收站**：任务删除是软删除（`deleted_at` 时间戳）。回收站支持恢复（`restore_task`）、单条/全部彻底清除（`purge_task` / `purge_deleted_tasks`）、按保留天数清理过期任务（`purge_expired_deleted_tasks`，应用启动时按设置自动执行，0 = 永久保留）。
- **耗时口径**：`started_at` 在任务首次进入「进行中」时自动打点；完成时由「起点（优先 started_at，其次 due_date）→ completed_at」推导 `time_spent`（分钟）。前端展示耗时优先用时间戳现算（秒级），算不出来才退回分钟数（见 `lib/duration.ts`）。
- **时间戳格式**：库中时间戳为 RFC3339 UTC，但格式不统一（Rust 侧写 `+00:00` 纳秒精度，前端回写 `Z` 毫秒精度），解析用 `parse_from_rfc3339`，比较时注意边界处理（`types/task.ts` 的 `localDayBounds` 有详细说明）。
- **默认项目**：首次启动自动创建收件箱（inbox，`is_default = 1`，数据库用部分唯一索引保证只有一个默认项目；活动项目名也有大小写不敏感的唯一索引）。删除项目时任务自动移回收件箱。
- **应用设置**：前端偏好（时间范围、侧栏 Tab、日历口径、置顶、小窗尺寸、回收站保留天数等）统一存 localStorage 单一键 `workmanager.settings`，`settingsStore.ts` 负责对历史遗留的分散键做一次性迁移。

## 前后端接口约定

- 前端所有后端调用集中在 `apps/desktop/src/lib/api.ts`，通过 `@tauri-apps/api/core` 的 `invoke` 调用；命令参数名用 camelCase（如 `taskId`、`includeArchived`、`retentionDays`），对应 Rust 参数的 snake_case（Tauri 自动转换）。
- TypeScript 类型（`apps/desktop/src/types/task.ts`）需与 Rust 模型（`crates/todo-core/src/models/`）**手动保持同步** —— 改一侧必须改另一侧。
- `TaskStatus` 在 Rust 中 `serde(rename_all = "snake_case")`，序列化值为 `'todo' | 'in_progress' | 'completed'`；但 `update_task_status` 命令接收 `String` 并在 Rust 侧 `FromStr` 解析。
- 错误传递：Rust 侧 `TodoError` 在 `src-tauri/src/commands/mod.rs` 转为 `CommandError { code, message }`（code 为 `VALIDATION_ERROR` / `NOT_FOUND` / `CONFLICT` / `STORAGE_ERROR`），以结构化对象 reject 到前端 Promise。
- `priority` 是 0–3 的整数（0=无, 1=低, 2=中, 3=高），数据库有 CHECK 约束；`title` 有 1–500 字符的 CHECK 约束。
- `list_tasks` 接收 `TaskQuery`（项目、状态、创建/完成时间区间、关键字、排序白名单字段、分页），返回 `TaskPage`；统计/日历类接口：`get_task_stats`（当前存量聚合）、`get_daily_activity`（按本地时区自然日聚合，每天最多附带 10 条任务摘要）。时间相关指标由前端基于 `get_daily_activity` 计算，后端不重复实现区间统计。
- `update_task_details` 的可空字段（description/attachments/due_date/completed_at/started_at/time_spent）是三层语义：键不传 = 不修改，显式传 `null` = 清空，传值 = 写入。Rust 侧用 `Option<Option<T>>` + 自定义反序列化器区分（见 `models/task.rs` 的 `de_nullable`），新增可空字段时沿用同一写法。
- 回收站列表查询不取 `attachments` 列（base64 图片可能几百 KB），见 `db.rs` 的 `TASK_COLUMNS_WITHOUT_ATTACHMENTS`；`list_deleted_tasks` 的 `page_size` 传 0 表示不分页全量返回（前端回收站即如此调用）。

## 代码风格

- **前端**：Vue 3 `<script setup lang="ts">` SFC；Pinia store 用 `xxxStore.ts` 命名放 `src/stores/`；可复用逻辑放 `src/lib/`；样式用 Tailwind CSS 4（通过 `@tailwindcss/vite` 插件，入口 `src/style.css` 含 Vditor 覆盖样式）；图标用 lucide-vue-next。
- **Rust**：标准 rustfmt 默认风格；错误统一用 `thiserror` 定义的 `TodoError`，crate 内用 `crate::error::Result<T>`；序列化字段用 snake_case。
- **文档与提交信息使用中文**（README、git 提交历史均为中文）；代码内注释中英混用。
- `src-tauri/src/lib.rs` 中新增命令后需在 `invoke_handler` 的 `generate_handler!` 列表中注册。
- `db.rs` 中任务行的 SELECT 列清单统一用常量 `TASK_COLUMNS`，新增字段只改该常量并对齐 `map_task` 的索引。

## 测试

- Rust 侧已有测试：`crates/todo-core/src/storage/db.rs` 末尾的 `#[cfg(test)]` 模块（`cargo test -p todo-core` 运行，`Database` 可独立于 Tauri 实例化），覆盖状态机耗时口径、列顺序契约、关键字过滤、null 清空语义等。改动存储层逻辑时应同步更新/补充这里的测试。
- 前端尚未配置测试框架，不要擅自引入；无 CI 配置。
- 验证改动的主要方式：`cargo test`（Rust）+ `pnpm --dir apps/desktop build`（前端类型检查与构建）+ `pnpm tauri dev` 手动验证。

## 安全注意事项

- 数据库文件（`data/*.db*`）包含用户真实数据，已 git 忽略；不要提交、不要读取生产库 `data/tasks.db` 用于调试（开发用 `tasks.dev.db`）。
- `tauri.conf.json` 中 `csp: null`（未启用 CSP）——这是已知现状，改动安全策略需谨慎评估对 Vditor 等依赖的影响。
- 新窗口能力必须在 `src-tauri/capabilities/default.json` 中显式声明权限，否则前端调用会被拒绝（当前已声明 set-always-on-top、set-size、start-dragging、minimize/maximize/close 等窗口权限与 `shell:default`）。
- 任务 `attachments` 字段存粘贴图片的 base64 数据，注意数据体积（列表查询应使用不含附件的列清单）。
- 不要运行具有破坏性的 git 命令（push、reset --hard 等），除非用户明确要求。

use rusqlite::{Connection, OpenFlags, params, Row};
use std::path::Path;
use chrono::{Duration, Utc};
use uuid::Uuid;
use std::str::FromStr;

use crate::error::{TodoError, Result};
use crate::models::project::{Project, CreateProjectInput, UpdateProjectInput, TaskCountsSummary};
use crate::models::task::{
    Task, TaskStatus, CreateTaskInput, TaskQuery, TaskPage, DailyActivity, ActivityItem,
    BreakdownBucket, TaskStats,
};
use crate::storage::schema;

/// 日历每天最多附带多少条任务摘要（供格子内缩略展示 + 悬停提示）。
/// 格子实际能铺几条取决于窗口高度，前端自行截断，这里给足余量。
const CALENDAR_ITEMS_PER_DAY: usize = 10;

/// 任务行的标准列清单。`map_task` 依赖这个顺序，新增字段时**只改这里**，
/// 并把 `map_task` 的索引对应上，避免散落各处的 SELECT 漏列。
const TASK_COLUMNS: &str = "id, project_id, title, description, status, priority, due_date, completed_at, started_at, deleted_at, attachments, time_spent, sort_order, created_at, updated_at";

/// 回收站用：附件是粘贴图片的 base64，可能几百 KB，列表不展示图片就不取它
const TASK_COLUMNS_WITHOUT_ATTACHMENTS: &str = "id, project_id, title, description, status, priority, due_date, completed_at, started_at, deleted_at, NULL AS attachments, time_spent, sort_order, created_at, updated_at";

/// 由「起点」与完成时刻推导耗时（分钟）。
///
/// 起点优先传 `started_at`（实际开始），没有才传 `due_date`（任务时间/预定）。
/// 口径与前端 `TaskItem.vue` 的 `autoTimeSpent` 保持一致：差值四舍五入到分钟，
/// 并夹到非负（提前完成算 0）。起点缺失或格式无法解析时返回 `None`，
/// 由调用方决定是否沿用原值。
///
/// 注意库里的时间戳格式并不统一（Rust 写的是 `+00:00` 纳秒精度，
/// 前端回写的是 `Z` 毫秒精度），`parse_from_rfc3339` 对两者都能解析。
fn compute_time_spent(origin: Option<&str>, completed_at: chrono::DateTime<Utc>) -> Option<i32> {
    let start = chrono::DateTime::parse_from_rfc3339(origin?).ok()?;
    let seconds = (completed_at - start.with_timezone(&Utc)).num_seconds();
    Some(((seconds as f64) / 60.0).round().max(0.0) as i32)
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_URI,
        )?;
        
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "busy_timeout", 5000)?;

        schema::migrate(&mut conn)?;
        
        let mut db = Self { conn };
        db.init_default_projects()?;
        
        Ok(db)
    }

    fn init_default_projects(&mut self) -> Result<()> {
        let count: i32 = self.conn.query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))?;

        if count == 0 {
            let now = Utc::now().to_rfc3339();
            
            let inbox_id = Uuid::new_v4().to_string();
            self.conn.execute(
                "INSERT INTO projects (id, name, color, icon, is_default, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![inbox_id, "收件箱", "#3b82f6", "inbox", 1, now, now],
            )?;

            let work_id = Uuid::new_v4().to_string();
            self.conn.execute(
                "INSERT INTO projects (id, name, color, icon, is_default, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![work_id, "工作", "#ef4444", "briefcase", 0, now, now],
            )?;

            let life_id = Uuid::new_v4().to_string();
            self.conn.execute(
                "INSERT INTO projects (id, name, color, icon, is_default, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![life_id, "生活", "#10b981", "user", 0, now, now],
            )?;
        }
        Ok(())
    }

    pub fn get_default_project_id(&self) -> Result<String> {
        self.conn.query_row("SELECT id FROM projects WHERE is_default = 1", [], |row| row.get(0)).map_err(|e| e.into())
    }

    fn map_project(row: &Row) -> rusqlite::Result<Project> {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            icon: row.get(3)?,
            is_default: row.get::<_, i32>(4)? == 1,
            is_archived: row.get::<_, i32>(5)? == 1,
            sort_order: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    }

    pub fn list_projects(&self, include_archived: bool) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, color, icon, is_default, is_archived, sort_order, created_at, updated_at
             FROM projects WHERE (?1 OR is_archived = 0) ORDER BY is_default DESC, sort_order ASC, created_at ASC"
        )?;

        let iter = stmt.query_map(params![include_archived], Self::map_project)?;
        let mut projects = Vec::new();
        for proj in iter { projects.push(proj?); }
        Ok(projects)
    }
    
    pub fn create_project(&self, input: CreateProjectInput) -> Result<Project> {
        let name = input.name.trim().to_string();
        if name.is_empty() || name.len() > 50 {
            return Err(TodoError::ValidationError("Project name must be 1-50 characters".into()));
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT INTO projects (id, name, color, icon, is_default, is_archived, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 0, 0, 0, ?5, ?6)",
            params![
                id, 
                name, 
                input.color.unwrap_or_else(|| "#3b82f6".into()), 
                input.icon.unwrap_or_else(|| "folder".into()),
                now, 
                now
            ],
        ).map_err(|e| match e {
            rusqlite::Error::SqliteFailure(err, _) if err.code == rusqlite::ffi::ErrorCode::ConstraintViolation => {
                TodoError::Conflict("Project name already exists".into())
            }
            _ => e.into()
        })?;

        self.get_project(&id)
    }

    pub fn get_project(&self, id: &str) -> Result<Project> {
        self.conn.query_row(
            "SELECT id, name, color, icon, is_default, is_archived, sort_order, created_at, updated_at FROM projects WHERE id = ?1",
            params![id],
            Self::map_project,
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => TodoError::NotFound(format!("Project {} not found", id)),
            _ => e.into()
        })
    }

    pub fn update_project(&self, id: &str, input: UpdateProjectInput) -> Result<Project> {
        let proj = self.get_project(id)?;
        if proj.is_default && input.name.is_some() {
            return Err(TodoError::ValidationError("Cannot rename inbox".into()));
        }

        let name = input.name.unwrap_or(proj.name).trim().to_string();
        if name.is_empty() || name.len() > 50 {
            return Err(TodoError::ValidationError("Project name must be 1-50 characters".into()));
        }
        
        let color = input.color.unwrap_or(proj.color);
        let icon = input.icon.unwrap_or(proj.icon);
        let sort_order = input.sort_order.unwrap_or(proj.sort_order);
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "UPDATE projects SET name = ?1, color = ?2, icon = ?3, sort_order = ?4, updated_at = ?5 WHERE id = ?6",
            params![name, color, icon, sort_order, now, id],
        )?;

        self.get_project(id)
    }

    pub fn archive_project(&self, id: &str) -> Result<Project> {
        let proj = self.get_project(id)?;
        if proj.is_default {
            return Err(TodoError::ValidationError("Cannot archive inbox".into()));
        }
        
        let now = Utc::now().to_rfc3339();
        self.conn.execute("UPDATE projects SET is_archived = 1, updated_at = ?1 WHERE id = ?2", params![now, id])?;
        self.get_project(id)
    }

    pub fn delete_project(&mut self, id: &str) -> Result<()> {
        let proj = self.get_project(id)?;
        if proj.is_default {
            return Err(TodoError::ValidationError("Cannot delete inbox".into()));
        }
        let default_id = self.get_default_project_id()?;
        let tx = self.conn.transaction()?;
        let now = Utc::now().to_rfc3339();
        tx.execute("UPDATE tasks SET project_id = ?1, updated_at = ?2 WHERE project_id = ?3", params![default_id, now, id])?;
        tx.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        tx.commit()?;
        Ok(())
    }

    fn map_task(row: &Row) -> rusqlite::Result<Task> {
        let status_str: String = row.get(4)?;
        Ok(Task {
            id: row.get(0)?,
            project_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            status: TaskStatus::from_str(&status_str).unwrap_or(TaskStatus::Todo),
            priority: row.get(5)?,
            due_date: row.get(6)?,
            completed_at: row.get(7)?,
            started_at: row.get(8)?,
            deleted_at: row.get(9)?,
            attachments: row.get(10)?,
            time_spent: row.get(11)?,
            sort_order: row.get(12)?,
            created_at: row.get(13)?,
            updated_at: row.get(14)?,
        })
    }

    pub fn get_task(&self, id: &str) -> Result<Task> {
        self.conn.query_row(
            &format!("SELECT {} FROM tasks WHERE id = ?1", TASK_COLUMNS),
            params![id],
            Self::map_task,
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => TodoError::NotFound(format!("Task {} not found", id)),
            _ => e.into()
        })
    }

    pub fn create_task(&self, input: CreateTaskInput) -> Result<Task> {
        let title = input.title.trim().to_string();
        if title.is_empty() || title.len() > 500 {
            return Err(TodoError::ValidationError("Title must be 1-500 characters".into()));
        }
        
        let project_id = input.project_id.unwrap_or_else(|| self.get_default_project_id().unwrap());
        let _proj = self.get_project(&project_id)?;
        
        let priority = input.priority.unwrap_or(0);
        if priority > 3 {
            return Err(TodoError::ValidationError("Priority must be 0-3".into()));
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        
        self.conn.execute(
            "INSERT INTO tasks (id, project_id, title, description, status, priority, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'todo', ?5, ?6, ?7)",
            params![id, project_id, title, input.description, priority, now, now],
        )?;

        self.get_task(&id)
    }

    pub fn update_task_status(&self, id: &str, status: TaskStatus) -> Result<Task> {
        let task = self.get_task(id)?;
        let status_str = status.to_string();
        let now_dt = Utc::now();
        let now = now_dt.to_rfc3339();

        let completed_at = if status == TaskStatus::Completed { Some(now.clone()) } else { None };

        // 实际开始时间：
        // - 首次进入「进行中」时打点（已有值就保留，避免来回切换把起点冲掉）
        // - 退回「待办」说明这次作废，清掉，下次开始会重新打点
        // - 标记完成时保持不变；若从未经过「进行中」则为空，耗时会退回用任务时间
        let started_at = match status {
            TaskStatus::InProgress => Some(task.started_at.clone().unwrap_or_else(|| now.clone())),
            TaskStatus::Todo => None,
            TaskStatus::Completed => task.started_at.clone(),
        };

        // 耗时 = 完成时刻 − 起点。起点优先用真实开始时间，没有才退回任务时间（预定）；
        // 两者都没有（或解析失败）就算不出来，保留原值不覆盖。
        let time_spent = if status == TaskStatus::Completed {
            let origin = started_at.as_deref().or(task.due_date.as_deref());
            compute_time_spent(origin, now_dt).or(task.time_spent)
        } else {
            // 未完成时耗时没有意义，清掉，否则会一直显示上一次的残留值
            None
        };

        self.conn.execute(
            "UPDATE tasks SET status = ?1, completed_at = ?2, started_at = ?3, time_spent = ?4, updated_at = ?5 WHERE id = ?6",
            params![status_str, completed_at, started_at, time_spent, now, id],
        )?;
        self.get_task(id)
    }

    pub fn update_task_priority(&self, id: &str, priority: u8) -> Result<Task> {
        if priority > 3 {
            return Err(TodoError::ValidationError("Priority must be 0-3".into()));
        }
        let now = Utc::now().to_rfc3339();
        self.conn.execute("UPDATE tasks SET priority = ?1, updated_at = ?2 WHERE id = ?3", params![priority, now, id])?;
        self.get_task(id)
    }

    pub fn update_task_details(&self, id: &str, input: crate::models::task::TaskUpdateInput) -> Result<Task> {
        let task = self.get_task(id)?;
        
        let now = Utc::now().to_rfc3339();
        
        let title = input.title.unwrap_or(task.title);
        let title = title.trim();
        if title.is_empty() || title.len() > 500 {
            return Err(TodoError::ValidationError("Title must be 1-500 characters".into()));
        }
        
        let description = if input.description.is_some() { input.description } else { task.description };
        let attachments = if input.attachments.is_some() { input.attachments } else { task.attachments };
        let due_date = if input.due_date.is_some() { input.due_date } else { task.due_date };
        let completed_at = if input.completed_at.is_some() { input.completed_at } else { task.completed_at };
        let started_at = if input.started_at.is_some() { input.started_at } else { task.started_at };
        let time_spent = if input.time_spent.is_some() { input.time_spent } else { task.time_spent };

        println!("SQL Params: attachments is some? {}, len: {}", attachments.is_some(), attachments.as_ref().map(|s| s.len()).unwrap_or(0));

        self.conn.execute(
            "UPDATE tasks SET title = ?1, description = ?2, attachments = ?3, due_date = ?4, completed_at = ?5, started_at = ?6, time_spent = ?7, updated_at = ?8 WHERE id = ?9",
            params![title, description, attachments, due_date, completed_at, started_at, time_spent, now, id]
        )?;
        self.get_task(id)
    }

    pub fn move_task(&self, id: &str, project_id: &str) -> Result<Task> {
        let proj = self.get_project(project_id)?;
        if proj.is_archived {
            return Err(TodoError::ValidationError("Cannot move task to archived project".into()));
        }
        let now = Utc::now().to_rfc3339();
        self.conn.execute("UPDATE tasks SET project_id = ?1, updated_at = ?2 WHERE id = ?3", params![project_id, now, id])?;
        self.get_task(id)
    }

    pub fn delete_task(&self, id: &str) -> Result<Task> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute("UPDATE tasks SET deleted_at = ?1, updated_at = ?2 WHERE id = ?3", params![now, now, id])?;
        self.get_task(id)
    }

    pub fn restore_task(&self, id: &str) -> Result<Task> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute("UPDATE tasks SET deleted_at = NULL, updated_at = ?1 WHERE id = ?2", params![now, id])?;
        self.get_task(id)
    }
    
    pub fn list_tasks(&self, query: TaskQuery) -> Result<TaskPage> {
        let mut sql = format!("SELECT {} FROM tasks WHERE deleted_at IS NULL", TASK_COLUMNS);
        let mut count_sql = "SELECT COUNT(*) FROM tasks WHERE deleted_at IS NULL".to_string();
        
        if let Some(pid) = query.project_id {
            let p = format!("'{}'", pid);
            sql.push_str(&format!(" AND project_id = {}", p));
            count_sql.push_str(&format!(" AND project_id = {}", p));
        }

        if let Some(statuses) = query.statuses {
            if !statuses.is_empty() {
                let status_list = statuses.iter().map(|s| format!("'{}'", s.to_string())).collect::<Vec<_>>().join(",");
                sql.push_str(&format!(" AND status IN ({})", status_list));
                count_sql.push_str(&format!(" AND status IN ({})", status_list));
            }
        }

        // 创建时间范围过滤（RFC3339 字符串可直接做字典序比较）
        if let Some(from) = query.created_from {
            if !from.is_empty() {
                let cond = format!(" AND created_at >= '{}'", from.replace('\'', "''"));
                sql.push_str(&cond);
                count_sql.push_str(&cond);
            }
        }

        if let Some(to) = query.created_to {
            if !to.is_empty() {
                let cond = format!(" AND created_at <= '{}'", to.replace('\'', "''"));
                sql.push_str(&cond);
                count_sql.push_str(&cond);
            }
        }

        // 完成时间范围过滤。completed_at 可空，NULL 参与比较结果为 NULL，
        // 因此按完成时间筛选时会自然排除掉尚未完成的任务。
        if let Some(from) = query.completed_from {
            if !from.is_empty() {
                let cond = format!(" AND completed_at >= '{}'", from.replace('\'', "''"));
                sql.push_str(&cond);
                count_sql.push_str(&cond);
            }
        }

        if let Some(to) = query.completed_to {
            if !to.is_empty() {
                let cond = format!(" AND completed_at <= '{}'", to.replace('\'', "''"));
                sql.push_str(&cond);
                count_sql.push_str(&cond);
            }
        }

        // 标题关键字过滤（回收站列表用的是另一套方法，注意保持一致）
        if let Some(kw) = query.keyword {
            let kw = kw.trim().to_string();
            if !kw.is_empty() {
                let like = format!("%{}%", kw.replace('\'', "''"));
                let cond = format!(" AND title LIKE '{}'", like);
                sql.push_str(&cond);
                count_sql.push_str(&cond);
            }
        }

        // 排序：指定了时间字段就按它排，否则用默认排序（未完成优先 + 优先级 + 录入时间）。
        // 列名走白名单，绝不拼接原始输入。
        let sort_col = match query.sort_by.as_deref() {
            Some("completed_at") => Some("completed_at"),
            Some("started_at") => Some("started_at"),
            Some("created_at") => Some("created_at"),
            Some("due_date") => Some("due_date"),
            _ => None,
        };

        match sort_col {
            Some(col) => {
                let dir = if query.sort_desc.unwrap_or(true) { "DESC" } else { "ASC" };
                // 该字段为 NULL 的一律沉底，避免没填时间的任务占满列表顶部；
                // 末尾再用 created_at 兜底，保证同值时顺序稳定。
                sql.push_str(&format!(
                    " ORDER BY ({} IS NULL) ASC, {} {}, created_at DESC",
                    col, col, dir
                ));
            }
            None => {
                sql.push_str(" ORDER BY CASE WHEN status = 'completed' THEN 1 ELSE 0 END ASC, priority DESC, sort_order ASC, created_at DESC");
            }
        }

        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(100).min(200);
        let offset = (page - 1) * page_size;

        sql.push_str(&format!(" LIMIT {} OFFSET {}", page_size, offset));

        let total: u64 = self.conn.query_row(&count_sql, [], |row| row.get(0))?;

        let mut stmt = self.conn.prepare(&sql)?;
        let iter = stmt.query_map([], Self::map_task)?;
        
        let mut items = Vec::new();
        for task in iter { items.push(task?); }
        
        Ok(TaskPage { items, total, page, page_size })
    }

    pub fn get_task_counts(&self) -> Result<TaskCountsSummary> {
        let inbox_id = self.get_default_project_id()?;
        
        let mut stmt = self.conn.prepare(
            "SELECT project_id, COUNT(*) FROM tasks WHERE deleted_at IS NULL AND status != 'completed' GROUP BY project_id"
        )?;
        
        let iter = stmt.query_map([], |row| {
            let pid: String = row.get(0)?;
            let count: u64 = row.get(1)?;
            Ok((pid, count))
        })?;
        
        let mut projects = std::collections::HashMap::new();
        let mut inbox = 0;

        for row in iter {
            let (pid, count) = row?;
            inbox += count;
            if pid != inbox_id {
                projects.insert(pid, count);
            }
        }

        let completed: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE deleted_at IS NULL AND status = 'completed'",
            [],
            |row| row.get(0),
        )?;

        Ok(TaskCountsSummary {
            inbox,
            projects,
            completed,
        })
    }

    /// 日历视图数据：按**本地时区**的自然日聚合。
    ///
    /// 一次算出「当天完成数 / 当天创建数 / 当天完成任务的耗时合计」，
    /// 并附带每天的若干条任务摘要供日历格内缩略展示 —— 因此前端切换统计口径
    /// 或渲染格子内容都无需再次请求。
    ///
    /// `from` / `to` 为本地日期字符串（`YYYY-MM-DD`，闭区间）。
    /// 日期换算交给 SQLite 的 `localtime` 修饰符，避免在应用层重复处理时区——
    /// `completed_at` / `created_at` 存的是 UTC（`+00:00`），直接按字符串截取日期会跨零点分错天。
    pub fn get_daily_activity(&self, from: &str, to: &str) -> Result<Vec<DailyActivity>> {
        let mut stmt = self.conn.prepare(
            "SELECT d, kind, title, project_id, minutes FROM (
                 SELECT date(completed_at, 'localtime') AS d,
                        'completed' AS kind,
                        title,
                        project_id,
                        completed_at AS ts,
                        COALESCE(time_spent, 0) AS minutes
                 FROM tasks
                 WHERE deleted_at IS NULL
                   AND completed_at IS NOT NULL
                   AND date(completed_at, 'localtime') BETWEEN ?1 AND ?2
                 UNION ALL
                 SELECT date(created_at, 'localtime') AS d,
                        'created' AS kind,
                        title,
                        project_id,
                        created_at AS ts,
                        0 AS minutes
                 FROM tasks
                 WHERE deleted_at IS NULL
                   AND date(created_at, 'localtime') BETWEEN ?1 AND ?2
             )
             ORDER BY d ASC, ts ASC",
        )?;

        let rows = stmt.query_map(params![from, to], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })?;

        // BTreeMap 保证返回结果按日期升序
        let mut map: std::collections::BTreeMap<String, DailyActivity> = std::collections::BTreeMap::new();

        for row in rows {
            let (date, kind, title, project_id, minutes) = row?;
            let entry = map.entry(date.clone()).or_insert_with(|| DailyActivity {
                date,
                completed_count: 0,
                created_count: 0,
                completed_minutes: 0,
                completed_items: Vec::new(),
                created_items: Vec::new(),
            });

            if kind == "completed" {
                entry.completed_count += 1;
                entry.completed_minutes += minutes;
                if entry.completed_items.len() < CALENDAR_ITEMS_PER_DAY {
                    entry.completed_items.push(ActivityItem { title, project_id });
                }
            } else {
                entry.created_count += 1;
                if entry.created_items.len() < CALENDAR_ITEMS_PER_DAY {
                    entry.created_items.push(ActivityItem { title, project_id });
                }
            }
        }

        Ok(map.into_values().collect())
    }

    // ---------- 回收站 ----------

    /// 回收站列表：只含已软删除的任务，按删除时间倒序，支持标题关键字。
    ///
    /// 刻意把 `attachments` 取成 NULL —— 附件是粘贴图片的 base64，单条可能几百 KB，
    /// 回收站列表不展示图片，没必要把它拖出来。
    pub fn list_deleted_tasks(
        &self,
        keyword: Option<String>,
        page: u32,
        page_size: u32,
    ) -> Result<TaskPage> {
        let mut sql = format!("SELECT {} FROM tasks WHERE deleted_at IS NOT NULL", TASK_COLUMNS_WITHOUT_ATTACHMENTS);
        let mut count_sql = "SELECT COUNT(*) FROM tasks WHERE deleted_at IS NOT NULL".to_string();

        if let Some(kw) = keyword {
            let kw = kw.trim();
            if !kw.is_empty() {
                let like = format!("%{}%", kw.replace('\'', "''"));
                let cond = format!(" AND title LIKE '{}'", like);
                sql.push_str(&cond);
                count_sql.push_str(&cond);
            }
        }

        sql.push_str(" ORDER BY deleted_at DESC");

        let page = page.max(1);
        let page_size = page_size.clamp(1, 200);
        let offset = (page - 1) * page_size;
        sql.push_str(&format!(" LIMIT {} OFFSET {}", page_size, offset));

        let total: u64 = self.conn.query_row(&count_sql, [], |row| row.get(0))?;

        let mut stmt = self.conn.prepare(&sql)?;
        let iter = stmt.query_map([], Self::map_task)?;
        let mut items = Vec::new();
        for task in iter {
            items.push(task?);
        }

        Ok(TaskPage {
            items,
            total,
            page,
            page_size,
        })
    }

    /// 彻底删除单条任务。
    ///
    /// 条件里带 `deleted_at IS NOT NULL`：只允许清理回收站里的任务，
    /// 防止绕过软删除直接把在用数据抹掉。
    pub fn purge_task(&self, id: &str) -> Result<()> {
        let affected = self.conn.execute(
            "DELETE FROM tasks WHERE id = ?1 AND deleted_at IS NOT NULL",
            params![id],
        )?;
        if affected == 0 {
            return Err(TodoError::NotFound(format!(
                "Task {} not found in trash",
                id
            )));
        }
        Ok(())
    }

    /// 清空回收站，返回删除条数
    pub fn purge_deleted_tasks(&self) -> Result<u64> {
        let affected = self
            .conn
            .execute("DELETE FROM tasks WHERE deleted_at IS NOT NULL", [])?;
        Ok(affected as u64)
    }

    /// 清理超过保留期的回收站任务，返回删除条数。
    /// `retention_days <= 0` 表示永久保留，不清理。
    pub fn purge_expired_deleted_tasks(&self, retention_days: i64) -> Result<u64> {
        if retention_days <= 0 {
            return Ok(0);
        }
        let cutoff = (Utc::now() - Duration::days(retention_days)).to_rfc3339();
        let affected = self.conn.execute(
            "DELETE FROM tasks WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
            params![cutoff],
        )?;
        Ok(affected as u64)
    }

    // ---------- 统计 ----------

    /// 统计页概览：存量计数 + 未完成任务的项目 / 优先级分布
    pub fn get_task_stats(&self) -> Result<TaskStats> {
        let mut todo = 0u64;
        let mut in_progress = 0u64;
        let mut completed = 0u64;

        let mut stmt = self.conn.prepare(
            "SELECT status, COUNT(*) FROM tasks WHERE deleted_at IS NULL GROUP BY status",
        )?;
        let iter = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
        })?;
        for row in iter {
            let (status, count) = row?;
            match status.as_str() {
                "todo" => todo = count,
                "in_progress" => in_progress = count,
                "completed" => completed = count,
                _ => {}
            }
        }

        // 未完成任务按项目分布：用 LEFT JOIN 一次把项目名与颜色带出来
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.name, p.color, COUNT(t.id) AS cnt
             FROM projects p
             LEFT JOIN tasks t
               ON t.project_id = p.id AND t.deleted_at IS NULL AND t.status != 'completed'
             WHERE p.is_archived = 0
             GROUP BY p.id, p.name, p.color
             HAVING cnt > 0
             ORDER BY cnt DESC, p.sort_order ASC",
        )?;
        let iter = stmt.query_map([], |row| {
            Ok(BreakdownBucket {
                key: row.get(0)?,
                label: row.get(1)?,
                color: Some(row.get(2)?),
                count: row.get(3)?,
            })
        })?;
        let mut by_project = Vec::new();
        for row in iter {
            by_project.push(row?);
        }

        // 未完成任务按优先级分布
        let mut stmt = self.conn.prepare(
            "SELECT priority, COUNT(*) FROM tasks
             WHERE deleted_at IS NULL AND status != 'completed'
             GROUP BY priority ORDER BY priority DESC",
        )?;
        let iter = stmt.query_map([], |row| {
            Ok((row.get::<_, u8>(0)?, row.get::<_, u64>(1)?))
        })?;
        let mut by_priority = Vec::new();
        for row in iter {
            let (priority, count) = row?;
            let label = match priority {
                3 => "高",
                2 => "中",
                1 => "低",
                _ => "无",
            };
            by_priority.push(BreakdownBucket {
                key: priority.to_string(),
                label: label.to_string(),
                color: None,
                count,
            });
        }

        Ok(TaskStats {
            todo,
            in_progress,
            completed,
            by_project,
            by_priority,
        })
    }

    /// 当前数据库的 schema 版本（设置页展示用）
    pub fn schema_version(&self) -> Result<i32> {
        let v: i32 = self.conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::task::TaskUpdateInput;
    use chrono::{Duration, Utc};

    fn at(iso: &str) -> chrono::DateTime<Utc> {
        chrono::DateTime::parse_from_rfc3339(iso).unwrap().with_timezone(&Utc)
    }

    /// 建一个临时库（会跑真实迁移与默认项目初始化），返回库与文件路径
    fn temp_db(tag: &str) -> (Database, std::path::PathBuf) {
        let mut path = std::env::temp_dir();
        path.push(format!("lizl_test_{}_{}.db", tag, Uuid::new_v4()));
        let db = Database::new(&path).expect("failed to create temp db");
        (db, path)
    }

    /// WAL 模式会额外产生 -wal / -shm，一并清掉
    fn cleanup(path: &std::path::Path) {
        for suffix in ["", "-wal", "-shm"] {
            let mut p = path.as_os_str().to_os_string();
            p.push(suffix);
            let _ = std::fs::remove_file(std::path::PathBuf::from(p));
        }
    }

    // ---------- 耗时计算 ----------

    #[test]
    fn computes_minutes_between_due_and_completion() {
        let done = at("2026-09-11T02:00:00+00:00");
        assert_eq!(
            compute_time_spent(Some("2026-09-11T00:30:00+00:00"), done),
            Some(90)
        );
    }

    #[test]
    fn rounds_to_nearest_minute() {
        let done = at("2026-09-11T02:00:00+00:00");
        // 10 秒 → 0 分钟；31 秒 → 1 分钟
        assert_eq!(compute_time_spent(Some("2026-09-11T01:59:50+00:00"), done), Some(0));
        assert_eq!(compute_time_spent(Some("2026-09-11T01:59:29+00:00"), done), Some(1));
    }

    #[test]
    fn clamps_negative_to_zero() {
        let done = at("2026-09-11T02:00:00+00:00");
        // 预定时间在完成之后，提前完成不应产生负耗时
        assert_eq!(compute_time_spent(Some("2026-09-11T05:00:00+00:00"), done), Some(0));
    }

    #[test]
    fn returns_none_without_usable_due_date() {
        let done = at("2026-09-11T02:00:00+00:00");
        assert_eq!(compute_time_spent(None, done), None);
        assert_eq!(compute_time_spent(Some(""), done), None);
        assert_eq!(compute_time_spent(Some("不是时间"), done), None);
    }

    /// 库里存在两种时间戳写法，都必须能解析
    #[test]
    fn parses_both_timestamp_formats_found_in_db() {
        let done = at("2026-09-11T02:00:00+00:00");
        // Rust 侧 Utc::now().to_rfc3339()：纳秒精度 + +00:00
        assert_eq!(
            compute_time_spent(Some("2026-09-11T01:00:00.612607200+00:00"), done),
            Some(60)
        );
        // 前端 new Date().toISOString() 回写：毫秒精度 + Z
        assert_eq!(
            compute_time_spent(Some("2026-09-11T01:00:00.000Z"), done),
            Some(60)
        );
    }

    #[test]
    fn handles_long_span() {
        let done = Utc::now();
        let due = done - Duration::days(1) - Duration::minutes(30);
        assert_eq!(
            compute_time_spent(Some(&due.to_rfc3339()), done),
            Some(24 * 60 + 30)
        );
    }

    // ---------- 状态切换时的耗时落库 ----------

    /// 回归测试：走界面上的圆圈（未开始 → 进行中 → 已完成）时，
    /// 耗时应按「实际开始时间 → 完成时间」计算，不依赖用户手填的任务时间。
    #[test]
    fn completing_after_in_progress_uses_started_at() {
        let (db, path) = temp_db("started_at");
        let task = db
            .create_task(CreateTaskInput {
                title: "回归：按开始时间算耗时".into(),
                project_id: None,
                description: None,
                priority: None,
            })
            .expect("create task");

        // 点一下 → 进行中，应打上开始时间
        let started = db
            .update_task_status(&task.id, TaskStatus::InProgress)
            .expect("start task");
        assert!(started.started_at.is_some(), "进入进行中时应记录开始时间");

        // 把开始时间挪到 90 分钟前，模拟真的做了一段时间
        // （测试模块与 db.rs 同模块，可以直接访问 conn）
        let back = (Utc::now() - Duration::minutes(90)).to_rfc3339();
        db.conn
            .execute(
                "UPDATE tasks SET started_at = ?1 WHERE id = ?2",
                params![back, &task.id],
            )
            .expect("backdate started_at");

        // 再点一下 → 已完成
        let done = db
            .update_task_status(&task.id, TaskStatus::Completed)
            .expect("complete task");
        assert!(done.completed_at.is_some(), "完成时间应被写入");
        let spent = done.time_spent.expect("应从开始时间算出耗时");
        assert!((89..=91).contains(&spent), "期望约 90 分钟，实际 {spent}");

        // 退回待办：开始时间、完成时间与耗时都应清空
        let reset = db
            .update_task_status(&task.id, TaskStatus::Todo)
            .expect("reset task");
        assert_eq!(reset.started_at, None, "退回待办应清掉开始时间");
        assert_eq!(reset.completed_at, None);
        assert_eq!(reset.time_spent, None);

        drop(db);
        cleanup(&path);
    }

    /// 从未经过「进行中」时，起点退回「任务时间（预定）」
    #[test]
    fn completing_without_started_at_falls_back_to_due_date() {
        let (db, path) = temp_db("status_time");
        let task = db
            .create_task(CreateTaskInput {
                title: "回归：状态按钮算耗时".into(),
                project_id: None,
                description: None,
                priority: None,
            })
            .expect("create task");

        let due = Utc::now() - Duration::minutes(90);
        db.update_task_details(
            &task.id,
            TaskUpdateInput {
                title: None,
                description: None,
                attachments: None,
                due_date: Some(due.to_rfc3339()),
                completed_at: None,
                started_at: None,
                time_spent: None,
            },
        )
        .expect("set due date");

        let done = db
            .update_task_status(&task.id, TaskStatus::Completed)
            .expect("complete task");

        assert!(done.completed_at.is_some(), "完成时间应被写入");
        assert!(done.started_at.is_none(), "没经过进行中就不该有开始时间");
        let spent = done.time_spent.expect("应退回用任务时间算出耗时");
        assert!((89..=91).contains(&spent), "期望约 90 分钟，实际 {spent}");

        // 取消完成后，耗时与完成时间都应被清掉，避免未完成任务显示残留值
        let reopened = db
            .update_task_status(&task.id, TaskStatus::Todo)
            .expect("reopen task");
        assert_eq!(reopened.completed_at, None);
        assert_eq!(reopened.time_spent, None);

        drop(db);
        cleanup(&path);
    }

    /// 手动修正开始时间应能落库
    #[test]
    fn started_at_can_be_corrected_manually() {
        let (db, path) = temp_db("started_manual");
        let task = db
            .create_task(CreateTaskInput {
                title: "手动修正开始时间".into(),
                project_id: None,
                description: None,
                priority: None,
            })
            .expect("create task");

        let manual = (Utc::now() - Duration::minutes(45)).to_rfc3339();
        let updated = db
            .update_task_details(
                &task.id,
                TaskUpdateInput {
                    title: None,
                    description: None,
                    attachments: None,
                    due_date: None,
                    completed_at: None,
                    started_at: Some(manual.clone()),
                    time_spent: None,
                },
            )
            .expect("correct started_at");
        assert_eq!(updated.started_at.as_deref(), Some(manual.as_str()));

        drop(db);
        cleanup(&path);
    }

    /// 没有起点时无从计算，应保留原值而不是清零
    #[test]
    fn completing_without_due_date_keeps_existing_time_spent() {
        let (db, path) = temp_db("status_no_due");
        let task = db
            .create_task(CreateTaskInput {
                title: "无任务时间".into(),
                project_id: None,
                description: None,
                priority: None,
            })
            .expect("create task");

        db.update_task_details(
            &task.id,
            TaskUpdateInput {
                title: None,
                description: None,
                attachments: None,
                due_date: None,
                completed_at: None,
                started_at: None,
                time_spent: Some(42),
            },
        )
        .expect("set time spent");

        let done = db
            .update_task_status(&task.id, TaskStatus::Completed)
            .expect("complete task");

        assert_eq!(done.time_spent, Some(42), "无起点时应保留原耗时");

        drop(db);
        cleanup(&path);
    }

    /// 列顺序契约。
    ///
    /// `map_task` 是按索引取列的，新增字段时最容易在这里静默错位
    /// （某个字段读到隔壁列的值，编译不会报错）。这里让所有字段都取非默认值，
    /// 再把三条查询路径（get / list / 回收站列表）都往返一遍。
    #[test]
    fn task_columns_stay_aligned_with_map_task() {
        let (db, path) = temp_db("columns");
        let task = db
            .create_task(CreateTaskInput {
                title: "占位".into(),
                project_id: None,
                description: None,
                priority: None,
            })
            .expect("create task");

        db.update_task_details(
            &task.id,
            TaskUpdateInput {
                title: Some("列顺序校验".into()),
                description: Some("正文".into()),
                attachments: None,
                due_date: Some("2026-09-11T01:00:00+00:00".into()),
                completed_at: Some("2026-09-11T02:00:00+00:00".into()),
                started_at: Some("2026-09-11T01:30:00+00:00".into()),
                time_spent: Some(30),
            },
        )
        .expect("fill all fields");

        let via_get = db.get_task(&task.id).expect("get_task");

        let page = db
            .list_tasks(TaskQuery {
                page: None,
                page_size: None,
                ..Default::default()
            })
            .expect("list_tasks");
        let via_list = page
            .items
            .iter()
            .find(|t| t.id == task.id)
            .expect("task should appear in list_tasks");

        db.delete_task(&task.id).expect("soft delete");
        let trash = db.list_deleted_tasks(None, 1, 10).expect("list_deleted_tasks");
        let via_trash = trash
            .items
            .iter()
            .find(|t| t.id == task.id)
            .expect("task should appear in trash");

        for t in [&via_get, via_list, via_trash] {
            assert_eq!(t.title, "列顺序校验");
            assert_eq!(t.description.as_deref(), Some("正文"));
            assert_eq!(t.due_date.as_deref(), Some("2026-09-11T01:00:00+00:00"));
            assert_eq!(t.completed_at.as_deref(), Some("2026-09-11T02:00:00+00:00"));
            assert_eq!(t.started_at.as_deref(), Some("2026-09-11T01:30:00+00:00"));
            assert_eq!(t.time_spent, Some(30));
            assert!(!t.created_at.is_empty(), "created_at 不应被读到隔壁列");
            assert!(t.updated_at.starts_with("2026-"), "updated_at 不应被读到隔壁列");
            assert!(t.project_id.len() > 10, "project_id 不应被读到隔壁列");
        }

        // 回收站列表刻意不取附件（base64 图片太大）
        assert!(via_trash.attachments.is_none());
        assert!(via_trash.deleted_at.is_some());

        drop(db);
        cleanup(&path);
    }

    /// 关键字搜索 + 按时间字段排序（「已完成」视图用的那套）
    #[test]
    fn keyword_filter_and_time_sorting() {
        let (db, path) = temp_db("search_sort");

        // 三条已完成任务，完成时间依次更近
        for (title, minutes_ago) in [("写周报", 90i64), ("改接口超时", 60), ("写周报终稿", 30)] {
            let task = db
                .create_task(CreateTaskInput {
                    title: title.into(),
                    project_id: None,
                    description: None,
                    priority: None,
                })
                .expect("create task");
            // 先走状态机置为已完成，再回头把时间改成期望值
            // （update_task_status 会用「此刻」覆盖 completed_at）
            db.update_task_status(&task.id, TaskStatus::Completed)
                .expect("complete");
            let done = Utc::now() - Duration::minutes(minutes_ago);
            db.update_task_details(
                &task.id,
                TaskUpdateInput {
                    started_at: Some((done - Duration::minutes(30)).to_rfc3339()),
                    completed_at: Some(done.to_rfc3339()),
                    time_spent: Some(30),
                    ..Default::default()
                },
            )
            .expect("backdate");
        }

        // 再来一条未完成的，用来验证按时间排序时 NULL 沉底
        db.create_task(CreateTaskInput {
            title: "还没做".into(),
            project_id: None,
            description: None,
            priority: None,
        })
        .expect("create todo");

        let query = |q: TaskQuery| db.list_tasks(q).expect("list_tasks").items;

        // 关键字：只命中含「周报」的两条
        let hit = query(TaskQuery {
            keyword: Some("周报".into()),
            ..Default::default()
        });
        assert_eq!(hit.len(), 2, "关键字应只命中 2 条");

        // 空关键字 / 纯空格不应过滤掉任何东西
        let blank = query(TaskQuery {
            keyword: Some("   ".into()),
            ..Default::default()
        });
        assert_eq!(blank.len(), 4, "纯空格关键字等于不过滤");

        // 默认按完成时间降序：最近完成的在最前
        let by_completed = query(TaskQuery {
            sort_by: Some("completed_at".into()),
            ..Default::default()
        });
        assert_eq!(by_completed.len(), 4);
        assert_eq!(by_completed[0].title, "写周报终稿");
        assert_eq!(by_completed[1].title, "改接口超时");
        assert_eq!(by_completed[2].title, "写周报");
        assert_eq!(by_completed[3].title, "还没做", "没有完成时间的应沉底");

        // 升序则反过来
        let asc = query(TaskQuery {
            sort_by: Some("completed_at".into()),
            sort_desc: Some(false),
            ..Default::default()
        });
        assert_eq!(asc[0].title, "写周报");
        assert_eq!(asc[3].title, "还没做", "升序时 NULL 同样沉底");

        // 按开始时间排序也应可用
        let by_started = query(TaskQuery {
            sort_by: Some("started_at".into()),
            ..Default::default()
        });
        assert_eq!(by_started[0].title, "写周报终稿");

        // 非法排序字段必须被忽略（走白名单），既不能报错也不能注入
        let fallback = query(TaskQuery {
            sort_by: Some("title; DROP TABLE tasks".into()),
            ..Default::default()
        });
        assert_eq!(fallback.len(), 4, "非法排序字段应回落到默认排序");
        assert_eq!(
            db.list_tasks(TaskQuery::default()).expect("db still alive").total,
            4,
            "tasks 表应完好无损"
        );

        drop(db);
        cleanup(&path);
    }
}

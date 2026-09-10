use rusqlite::{Connection, OpenFlags, params, Row};
use std::path::Path;
use chrono::Utc;
use uuid::Uuid;
use std::str::FromStr;

use crate::error::{TodoError, Result};
use crate::models::project::{Project, CreateProjectInput, UpdateProjectInput, TaskCountsSummary};
use crate::models::task::{Task, TaskStatus, CreateTaskInput, TaskQuery, TaskPage, DailyActivity, ActivityItem};
use crate::storage::schema;

/// 日历每天最多附带多少条任务摘要（供格子内缩略展示 + 悬停提示）
const CALENDAR_ITEMS_PER_DAY: usize = 4;

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
            deleted_at: row.get(8)?,
            attachments: row.get(9)?,
            time_spent: row.get(10)?,
            sort_order: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    }

    pub fn get_task(&self, id: &str) -> Result<Task> {
        self.conn.query_row(
            "SELECT id, project_id, title, description, status, priority, due_date, completed_at, deleted_at, attachments, time_spent, sort_order, created_at, updated_at FROM tasks WHERE id = ?1",
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
        let _task = self.get_task(id)?;
        let status_str = status.to_string();
        let now = Utc::now().to_rfc3339();
        
        let completed_at = if status == TaskStatus::Completed { Some(now.clone()) } else { None };

        self.conn.execute(
            "UPDATE tasks SET status = ?1, completed_at = ?2, updated_at = ?3 WHERE id = ?4",
            params![status_str, completed_at, now, id],
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
        let time_spent = if input.time_spent.is_some() { input.time_spent } else { task.time_spent };

        println!("SQL Params: attachments is some? {}, len: {}", attachments.is_some(), attachments.as_ref().map(|s| s.len()).unwrap_or(0));

        self.conn.execute(
            "UPDATE tasks SET title = ?1, description = ?2, attachments = ?3, due_date = ?4, completed_at = ?5, time_spent = ?6, updated_at = ?7 WHERE id = ?8", 
            params![title, description, attachments, due_date, completed_at, time_spent, now, id]
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
        let mut sql = "SELECT id, project_id, title, description, status, priority, due_date, completed_at, deleted_at, attachments, time_spent, sort_order, created_at, updated_at FROM tasks WHERE deleted_at IS NULL".to_string();
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

        sql.push_str(" ORDER BY CASE WHEN status = 'completed' THEN 1 ELSE 0 END ASC, priority DESC, sort_order ASC, created_at DESC");

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
}

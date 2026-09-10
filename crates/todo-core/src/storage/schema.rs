use rusqlite::Connection;
use crate::error::Result;

pub fn migrate(conn: &mut Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY NOT NULL
        );"
    )?;

    let version: i32 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    if version < 1 {
        let tx = conn.transaction()?;
        tx.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                color TEXT NOT NULL DEFAULT '#3b82f6',
                icon TEXT NOT NULL DEFAULT 'folder',
                is_default INTEGER NOT NULL DEFAULT 0,
                is_archived INTEGER NOT NULL DEFAULT 0,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                CHECK (is_default IN (0, 1)),
                CHECK (is_archived IN (0, 1))
            );

            CREATE UNIQUE INDEX IF NOT EXISTS idx_projects_single_default
            ON projects(is_default) WHERE is_default = 1;

            CREATE UNIQUE INDEX IF NOT EXISTS idx_projects_active_name
            ON projects(name COLLATE NOCASE) WHERE is_archived = 0;

            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY NOT NULL,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL DEFAULT 'todo',
                priority INTEGER NOT NULL DEFAULT 0,
                due_date TEXT,
                completed_at TEXT,
                deleted_at TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                CHECK (status IN ('todo', 'in_progress', 'completed')),
                CHECK (priority BETWEEN 0 AND 3),
                CHECK (length(trim(title)) BETWEEN 1 AND 500)
            );

            CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON tasks(project_id);
            CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
            CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at);
            CREATE INDEX IF NOT EXISTS idx_tasks_active_project_status
            ON tasks(project_id, status, priority, created_at) WHERE deleted_at IS NULL;

            INSERT INTO schema_migrations (version) VALUES (1);
            "
        )?;
        tx.commit()?;
    }

    if version < 2 {
        let tx = conn.transaction()?;
        tx.execute_batch(
            "
            ALTER TABLE tasks ADD COLUMN attachments TEXT;
            INSERT INTO schema_migrations (version) VALUES (2);
            "
        )?;
        tx.commit()?;
    }

    if version < 3 {
        let tx = conn.transaction()?;
        tx.execute_batch(
            "
            ALTER TABLE tasks ADD COLUMN time_spent INTEGER;
            INSERT INTO schema_migrations (version) VALUES (3);
            "
        )?;
        tx.commit()?;
    }

    if version < 4 {
        // 日历视图按完成时间做范围查询与按天聚合，需要独立索引
        let tx = conn.transaction()?;
        tx.execute_batch(
            "
            CREATE INDEX IF NOT EXISTS idx_tasks_completed_at
            ON tasks(completed_at) WHERE deleted_at IS NULL;
            INSERT INTO schema_migrations (version) VALUES (4);
            "
        )?;
        tx.commit()?;
    }

    Ok(())
}

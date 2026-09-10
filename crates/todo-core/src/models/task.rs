use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Completed,
}

impl ToString for TaskStatus {
    fn to_string(&self) -> String {
        match self {
            TaskStatus::Todo => "todo".to_string(),
            TaskStatus::InProgress => "in_progress".to_string(),
            TaskStatus::Completed => "completed".to_string(),
        }
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = crate::error::TodoError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "todo" => Ok(TaskStatus::Todo),
            "in_progress" => Ok(TaskStatus::InProgress),
            "completed" => Ok(TaskStatus::Completed),
            _ => Err(crate::error::TodoError::ValidationError(format!("Invalid task status: {}", s))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: u8,
    pub due_date: Option<String>,
    pub completed_at: Option<String>,
    pub deleted_at: Option<String>,
    pub attachments: Option<String>,
    pub time_spent: Option<i32>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(serde::Deserialize)]
pub struct TaskUpdateInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub attachments: Option<String>,
    pub due_date: Option<String>,
    pub completed_at: Option<String>,
    pub time_spent: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskInput {
    pub title: String,
    pub project_id: Option<String>,
    pub description: Option<String>,
    pub priority: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskQuery {
    pub project_id: Option<String>,
    pub statuses: Option<Vec<TaskStatus>>,
    /// 创建时间下界（RFC3339，含）
    pub created_from: Option<String>,
    /// 创建时间上界（RFC3339，含）
    pub created_to: Option<String>,
    /// 完成时间下界（RFC3339，含）
    pub completed_from: Option<String>,
    /// 完成时间上界（RFC3339，含）
    pub completed_to: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

/// 日历格内缩略展示用的一条任务摘要（不含正文与附件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    pub title: String,
    pub project_id: String,
}

/// 日历用：某一天的活跃度汇总（按本地时区的自然日聚合）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyActivity {
    /// 本地日期，格式 YYYY-MM-DD
    pub date: String,
    /// 当天完成的任务数
    pub completed_count: u64,
    /// 当天创建的任务数
    pub created_count: u64,
    /// 当天完成任务的耗时合计（分钟）
    pub completed_minutes: i64,
    /// 当天完成的任务（按完成时间正序，最多若干条）
    pub completed_items: Vec<ActivityItem>,
    /// 当天创建的任务（按创建时间正序，最多若干条）
    pub created_items: Vec<ActivityItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPage {
    pub items: Vec<Task>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

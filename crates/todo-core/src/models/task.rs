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
    /// 实际开始时间：任务首次进入「进行中」时自动打点，用于计算真实耗时
    pub started_at: Option<String>,
    pub deleted_at: Option<String>,
    pub attachments: Option<String>,
    pub time_spent: Option<i32>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// 区分「键没传」与「显式传了 null」：serde 默认会把两者都收成 `None`，
/// 导致前端清空字段（传 null）被当成「不修改」。这个反序列化器只在键存在时
/// 被调用（键缺失走 `default` 得到外层 `None`），所以返回的总是 `Some(...)`，
/// 内层再按 JSON 值是 null 还是具体值得到 `None`（清空）/ `Some(v)`（写入）。
fn de_nullable<'de, D, T>(deserializer: D) -> std::result::Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::<T>::deserialize(deserializer)?))
}

#[derive(serde::Deserialize, Default)]
pub struct TaskUpdateInput {
    pub title: Option<String>,
    #[serde(default, deserialize_with = "de_nullable")]
    pub description: Option<Option<String>>,
    #[serde(default, deserialize_with = "de_nullable")]
    pub attachments: Option<Option<String>>,
    #[serde(default, deserialize_with = "de_nullable")]
    pub due_date: Option<Option<String>>,
    #[serde(default, deserialize_with = "de_nullable")]
    pub completed_at: Option<Option<String>>,
    /// 允许手动修正实际开始时间
    #[serde(default, deserialize_with = "de_nullable")]
    pub started_at: Option<Option<String>>,
    #[serde(default, deserialize_with = "de_nullable")]
    pub time_spent: Option<Option<i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskInput {
    pub title: String,
    pub project_id: Option<String>,
    pub description: Option<String>,
    pub priority: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    /// 标题关键字（模糊匹配）
    pub keyword: Option<String>,
    /// 排序字段，白名单：completed_at / started_at / created_at / due_date。
    /// 留空则用默认排序（未完成优先 + 优先级 + 录入时间）。
    pub sort_by: Option<String>,
    /// 排序方向，默认降序
    pub sort_desc: Option<bool>,
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

/// 统计页用的一个分组桶（按项目 / 按优先级）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakdownBucket {
    /// 分组键：项目 id 或优先级数值
    pub key: String,
    /// 展示名：项目名、优先级中文名等
    pub label: String,
    /// 项目色；优先级分组为 None，由前端按档位上色
    pub color: Option<String>,
    pub count: u64,
}

/// 统计页概览数据。
///
/// 只出「当前存量」的聚合；时间相关的指标（近 N 天完成数 / 耗时）由前端基于
/// `get_daily_activity` 计算，避免重复实现一套区间统计。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStats {
    pub todo: u64,
    pub in_progress: u64,
    pub completed: u64,
    /// 未完成任务按项目分布，数量降序
    pub by_project: Vec<BreakdownBucket>,
    /// 未完成任务按优先级分布，优先级降序（高 → 无）
    pub by_priority: Vec<BreakdownBucket>,
}

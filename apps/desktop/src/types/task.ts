export type TaskStatus = 'todo' | 'in_progress' | 'completed';

export interface Task {
  id: string;
  project_id: string;
  title: string;
  description: string | null;
  status: TaskStatus;
  priority: number; // 0, 1, 2, 3
  due_date: string | null;
  completed_at: string | null;
  /** 实际开始时间：任务首次进入「进行中」时自动打点，用于计算真实耗时 */
  started_at: string | null;
  deleted_at: string | null;
  attachments: string | null;
  time_spent: number | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface TaskUpdateInput {
  title?: string | null;
  description?: string | null;
  attachments?: string | null;
  due_date?: string | null;
  completed_at?: string | null;
  /** 允许手动修正实际开始时间 */
  started_at?: string | null;
  time_spent?: number | null;
}

export interface Project {
  id: string;
  name: string;
  color: string;
  icon: string;
  is_default: boolean;
  is_archived: boolean;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface TaskPage {
  items: Task[];
  total: number;
  page: number;
  page_size: number;
}

export interface TaskCountsSummary {
  inbox: number;
  projects: Record<string, number>;
  completed: number;
}

export interface CreateTaskInput {
  title: string;
  project_id?: string;
  description?: string;
  priority?: number;
}

export interface TaskQuery {
  project_id?: string;
  statuses?: TaskStatus[];
  /** 创建时间下界（ISO 8601，含） */
  created_from?: string;
  /** 创建时间上界（ISO 8601，含） */
  created_to?: string;
  /** 完成时间下界（ISO 8601，含） */
  completed_from?: string;
  /** 完成时间上界（ISO 8601，含） */
  completed_to?: string;
  /** 标题关键字（模糊匹配） */
  keyword?: string;
  /** 排序字段（后端白名单：completed_at / started_at / created_at / due_date） */
  sort_by?: TaskSortKey;
  /** 排序方向，默认降序 */
  sort_desc?: boolean;
  page?: number;
  page_size?: number;
}

/** 「已完成」列表可选的排序字段 */
export type TaskSortKey = 'completed_at' | 'started_at' | 'created_at';

export const TASK_SORT_OPTIONS: { key: TaskSortKey; label: string }[] = [
  { key: 'completed_at', label: '完成时间' },
  { key: 'started_at', label: '开始时间' },
  { key: 'created_at', label: '录入时间' },
];

export const DEFAULT_TASK_SORT: TaskSortKey = 'completed_at';

/** 日历格内缩略展示用的一条任务摘要（不含正文与附件） */
export interface ActivityItem {
  title: string;
  project_id: string;
}

/** 日历用：某一天的活跃度汇总（后端按本地时区的自然日聚合） */
export interface DailyActivity {
  /** 本地日期，格式 YYYY-MM-DD */
  date: string;
  /** 当天完成的任务数 */
  completed_count: number;
  /** 当天创建的任务数 */
  created_count: number;
  /** 当天完成任务的耗时合计（分钟） */
  completed_minutes: number;
  /** 当天完成的任务（按完成时间正序，后端截断到若干条） */
  completed_items: ActivityItem[];
  /** 当天创建的任务（按创建时间正序，后端截断到若干条） */
  created_items: ActivityItem[];
}

/**
 * 把本地日期（YYYY-MM-DD）换算成一对可直接做字典序比较的 RFC3339 边界。
 *
 * 库中时间戳有两种写法：Rust 侧 chrono 的 `to_rfc3339()`（`+00:00` 结尾，
 * 纳秒非零时带最多 9 位小数），前端回写的 `toISOString()`（`Z` 结尾，恒带
 * 3 位毫秒）。字符串比较要保证两种写法都落在区间内：
 * - 下界用不带小数的秒级格式 —— 小数点（0x2E）的编码大于加号（0x2B），
 *   因此当天零点整点完成的任务不会被漏掉；
 * - 上界用「秒 + `.999Z`」—— 它是当天最后一秒所有写法的字典序上界：
 *   毫秒写法 `…59.999Z` 与它相等，纳秒写法 `…59.999999999+00:00` 因
 *   `'9' < 'Z'` 排在它前面，秒级写法因 `'+' < '.'` 也排在它前面。
 */
export function localDayBounds(dateKey: string): { from: string; to: string } {
  const [y, m, d] = dateKey.split('-').map(Number);
  const start = new Date(y, m - 1, d, 0, 0, 0);
  const end = new Date(y, m - 1, d, 23, 59, 59);
  return {
    from: start.toISOString().slice(0, 19) + '+00:00',
    to: end.toISOString().slice(0, 19) + '.999Z',
  };
}

/** Date → 本地日期键（YYYY-MM-DD），用于与后端 `DailyActivity.date` 对齐 */
export function toDateKey(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

/** 时间范围筛选项 */
export type DateRangeKey = 'week' | 'month' | 'quarter' | 'half_year' | 'year' | 'all';

export interface DateRangeOption {
  key: DateRangeKey;
  label: string;
  /** 往前推的天数（与 months 二选一） */
  days?: number;
  /** 往前推的月数（与 days 二选一） */
  months?: number;
}

export const DATE_RANGE_OPTIONS: DateRangeOption[] = [
  { key: 'week', label: '近 7 天', days: 7 },
  { key: 'month', label: '近 30 天', days: 30 },
  { key: 'quarter', label: '近 3 个月', months: 3 },
  { key: 'half_year', label: '近半年', months: 6 },
  { key: 'year', label: '近 1 年', months: 12 },
  { key: 'all', label: '全部时间' },
];

export const DEFAULT_DATE_RANGE: DateRangeKey = 'half_year';

/** 后端 created_at 为 RFC3339（+00:00 偏移），统一格式以便字符串比较 */
function toRfc3339(d: Date): string {
  return d.toISOString().replace('Z', '+00:00');
}

/** 将时间范围选项换算为查询下界的 ISO 字符串；'all' 返回 undefined 表示不限 */
export function dateRangeToFrom(key: DateRangeKey): string | undefined {
  const opt = DATE_RANGE_OPTIONS.find(o => o.key === key);
  if (!opt) return undefined;
  if (opt.months) {
    const d = new Date();
    d.setMonth(d.getMonth() - opt.months);
    return toRfc3339(d);
  }
  if (opt.days) {
    const d = new Date();
    d.setDate(d.getDate() - opt.days);
    return toRfc3339(d);
  }
  return undefined;
}

export interface CreateProjectInput {
  name: string;
  color?: string;
  icon?: string;
}

export interface UpdateProjectInput {
  name?: string;
  color?: string;
  icon?: string;
  sort_order?: number;
}

/** 统计页的一个分组桶（按项目 / 按优先级） */
export interface BreakdownBucket {
  /** 分组键：项目 id 或优先级数值 */
  key: string;
  /** 展示名：项目名、优先级中文名 */
  label: string;
  /** 项目色；优先级分组为 null，由前端按档位上色 */
  color: string | null;
  count: number;
}

/** 统计页概览：只含「当前存量」的聚合 */
export interface TaskStats {
  todo: number;
  in_progress: number;
  completed: number;
  /** 未完成任务按项目分布，数量降序 */
  by_project: BreakdownBucket[];
  /** 未完成任务按优先级分布，优先级降序 */
  by_priority: BreakdownBucket[];
}

/** 应用信息（设置页展示） */
export interface AppInfo {
  db_path: string;
  version: string;
  schema_version: number;
}

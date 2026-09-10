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
  page?: number;
  page_size?: number;
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

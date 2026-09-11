/**
 * 时长展示与计算。
 *
 * 背景：数据库里的 `time_spent` 是**整数分钟**，精度不够显示到秒。
 * 但 `started_at` / `due_date` / `completed_at` 都是带纳秒的原始时间戳，
 * 所以耗时展示一律**优先用时间戳现算**，算不出来（时间戳缺失）才退回分钟数。
 */

interface TimeBearing {
  started_at?: string | null;
  due_date?: string | null;
  completed_at?: string | null;
  time_spent?: number | null;
}

/**
 * 秒数 → 「X天X小时X分X秒」，跳过为 0 的单位。
 * 例：59 → `59秒`；119 → `1分59秒`；3600 → `1小时`；90061 → `1天1小时1分1秒`
 */
export function formatDuration(totalSeconds: number): string {
  const s = Math.round(totalSeconds);
  if (!Number.isFinite(s) || s <= 0) return '不到 1 秒';

  const days = Math.floor(s / 86400);
  const hours = Math.floor((s % 86400) / 3600);
  const minutes = Math.floor((s % 3600) / 60);
  const seconds = s % 60;

  const parts: string[] = [];
  if (days > 0) parts.push(`${days}天`);
  if (hours > 0) parts.push(`${hours}小时`);
  if (minutes > 0) parts.push(`${minutes}分`);
  if (seconds > 0) parts.push(`${seconds}秒`);
  return parts.join('');
}

/** 分钟数 → 时长文案（聚合统计用，秒位恒为 0）；0 或负数返回空串 */
export function formatDurationFromMinutes(minutes: number | null | undefined): string {
  if (minutes === null || minutes === undefined || minutes <= 0) return '';
  return formatDuration(minutes * 60);
}

/** 两个 ISO 时间相差的秒数；任一缺失或无法解析返回 null，后者早于前者夹到 0 */
export function elapsedSeconds(fromIso?: string | null, toIso?: string | null): number | null {
  if (!fromIso || !toIso) return null;
  const from = new Date(fromIso).getTime();
  const to = new Date(toIso).getTime();
  if (!Number.isFinite(from) || !Number.isFinite(to)) return null;
  return Math.max(0, Math.round((to - from) / 1000));
}

/**
 * 任务耗时的起点与终点。
 * 起点优先用「实际开始时间」，没有才退回「任务时间（预定）」——
 * 与后端 `compute_time_spent` 同一套口径。
 */
export function taskDurationSeconds(task: TimeBearing): number | null {
  return elapsedSeconds(task.started_at || task.due_date, task.completed_at);
}

/** 任务耗时的展示文案；完全算不出来时返回空串 */
export function taskDurationText(task: TimeBearing): string {
  const secs = taskDurationSeconds(task);
  if (secs !== null) return formatDuration(secs);
  return formatDurationFromMinutes(task.time_spent);
}

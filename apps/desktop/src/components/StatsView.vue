<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { api } from '../lib/api';
import { useUiStore } from '../stores/uiStore';
import { useSettingsStore } from '../stores/settingsStore';
import { BarChart3, Flame } from 'lucide-vue-next';
import type { DailyActivity, TaskStats } from '../types/task';
import { toDateKey } from '../types/task';
import { formatDurationFromMinutes } from '../lib/duration';

const uiStore = useUiStore();
const settingsStore = useSettingsStore();

// 初始口径跟随设置页的「日历默认统计口径」（日历与趋势图共用），页内仍可临时切换
type Metric = 'completed' | 'created';
const metric = ref<Metric>(settingsStore.settings.calendarMetric);

const stats = ref<TaskStats | null>(null);
const activity = ref<DailyActivity[]>([]);
const loading = ref(true);

/** 近 30 天窗口 */
const days = 30;
const windowFrom = computed(() => {
  const d = new Date();
  d.setDate(d.getDate() - (days - 1));
  return toDateKey(d);
});
const windowTo = computed(() => toDateKey(new Date()));

const hasData = computed(
  () => !!stats.value && (stats.value.todo + stats.value.in_progress + stats.value.completed) > 0
);

async function load() {
  loading.value = true;
  try {
    const [s, a] = await Promise.all([
      api.getTaskStats(),
      api.getDailyActivity(windowFrom.value, windowTo.value),
    ]);
    stats.value = s;
    activity.value = a;
  } catch (e: any) {
    uiStore.showMessage(e.message || '加载统计失败', 'error');
  } finally {
    loading.value = false;
  }
}

onMounted(load);

// ---------- 概览 ----------

const countOf = (a: DailyActivity) =>
  metric.value === 'completed' ? a.completed_count : a.created_count;

/**
 * 补齐成完整的 N 天序列。
 * `getDailyActivity` 只返回「有记录」的日期（对日历是按 key 查，正好合适），
 * 但趋势图必须逐格渲染，直接遍历会缺天、刻度也会错位。
 */
const trend = computed(() => {
  const map = new Map(activity.value.map(a => [a.date, a]));
  const start = new Date();
  start.setDate(start.getDate() - (days - 1));
  const out: { date: string; count: number; completed: number; minutes: number }[] = [];
  for (let i = 0; i < days; i++) {
    const d = new Date(start.getFullYear(), start.getMonth(), start.getDate() + i);
    const key = toDateKey(d);
    const a = map.get(key);
    out.push({
      date: key,
      count: a ? countOf(a) : 0,
      completed: a ? a.completed_count : 0,
      minutes: a ? a.completed_minutes : 0,
    });
  }
  return out;
});

const windowCompleted = computed(() =>
  activity.value.reduce((sum, a) => sum + a.completed_count, 0)
);
const windowMinutes = computed(() =>
  activity.value.reduce((sum, a) => sum + a.completed_minutes, 0)
);

/** 聚合统计的耗时展示（只有分钟精度，秒位恒为空）；没数据时给占位符 */
const formatAggregate = (minutes: number) => formatDurationFromMinutes(minutes) || '—';

const avgMinutes = computed(() =>
  windowCompleted.value > 0 ? Math.round(windowMinutes.value / windowCompleted.value) : 0
);

const pendingTotal = computed(() =>
  stats.value ? stats.value.todo + stats.value.in_progress : 0
);

/** 近 30 天里最长的一段连续「有完成记录」的天数（不随统计口径变化） */
const longestStreak = computed(() => {
  let best = 0;
  let run = 0;
  for (const d of trend.value) {
    if (d.completed > 0) {
      run += 1;
      best = Math.max(best, run);
    } else {
      run = 0;
    }
  }
  return best;
});

// ---------- 趋势 ----------

const trendMax = computed(() => Math.max(1, ...trend.value.map(d => d.count)));

/** 只取 4 个刻度，避免 30 个标签挤在一起 */
const trendTicks = computed(() =>
  [0, Math.floor(days / 3), Math.floor((days * 2) / 3), days - 1].map(i => {
    const [, m, d] = trend.value[i].date.split('-');
    return `${Number(m)}/${Number(d)}`;
  })
);

// ---------- 分布 ----------

const PRIORITY_COLOR: Record<string, string> = {
  '3': '#F43F5E',
  '2': '#F59E0B',
  '1': '#3B82F6',
  '0': '#CBD5E1',
};

const projectMax = computed(() =>
  Math.max(1, ...(stats.value?.by_project ?? []).map(b => b.count))
);
const priorityTotal = computed(() =>
  (stats.value?.by_priority ?? []).reduce((s, b) => s + b.count, 0)
);
</script>

<template>
  <div class="flex-1 min-h-0 overflow-y-auto custom-scrollbar px-6 pt-5 pb-6">
    <div v-if="loading" class="flex justify-center py-24">
      <div class="w-6 h-6 border-2 border-slate-200 border-t-blue-500 rounded-full animate-spin"></div>
    </div>

    <div v-else-if="!hasData" class="flex flex-col items-center justify-center py-24 text-slate-400">
      <div class="w-16 h-16 bg-slate-100 rounded-full flex items-center justify-center mb-5">
        <BarChart3 class="w-8 h-8 text-slate-300" />
      </div>
      <p class="text-[14px] font-medium text-slate-500">还没有可统计的数据</p>
      <p class="text-[12px] text-slate-400 mt-1">添加一些任务并完成后，这里就会有统计</p>
    </div>

    <template v-else>
      <!-- 指标卡 -->
      <div class="grid grid-cols-4 gap-3">
        <div class="bg-slate-100 rounded-xl px-4 py-3">
          <p class="text-[11px] text-slate-500">未完成</p>
          <p class="text-[22px] leading-tight text-slate-800 tabular-nums mt-1">{{ pendingTotal }}</p>
          <p class="text-[11px] text-slate-400 mt-0.5">
            进行中 {{ stats?.in_progress ?? 0 }} · 待办 {{ stats?.todo ?? 0 }}
          </p>
        </div>
        <div class="bg-slate-100 rounded-xl px-4 py-3">
          <p class="text-[11px] text-slate-500">近 {{ days }} 天完成</p>
          <p class="text-[22px] leading-tight text-slate-800 tabular-nums mt-1">{{ windowCompleted }}</p>
          <p class="text-[11px] text-slate-400 mt-0.5">累计已完成 {{ stats?.completed ?? 0 }}</p>
        </div>
        <div class="bg-slate-100 rounded-xl px-4 py-3">
          <p class="text-[11px] text-slate-500">近 {{ days }} 天耗时</p>
          <p class="text-[22px] leading-tight text-slate-800 tabular-nums mt-1">{{ formatAggregate(windowMinutes) }}</p>
          <p class="text-[11px] text-slate-400 mt-0.5">开始时间优先，缺省用任务时间</p>
        </div>
        <div class="bg-slate-100 rounded-xl px-4 py-3">
          <p class="text-[11px] text-slate-500">平均每项耗时</p>
          <p class="text-[22px] leading-tight text-slate-800 tabular-nums mt-1">{{ formatAggregate(avgMinutes) }}</p>
          <p class="text-[11px] text-slate-400 mt-0.5">近 {{ days }} 天完成的任务</p>
        </div>
      </div>

      <!-- 趋势 -->
      <div class="mt-4 bg-white rounded-2xl border border-slate-100 shadow-sm px-4 py-4">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-3">
            <h3 class="text-[13px] font-medium text-slate-700">近 {{ days }} 天{{ metric === 'completed' ? '完成' : '创建' }}趋势</h3>
            <span
              v-if="longestStreak > 0"
              class="flex items-center gap-1 text-[11px] text-amber-600 bg-amber-50 px-2 py-0.5 rounded-md"
              title="近 30 天内最长的一段连续有完成记录的天数"
            >
              <Flame class="w-3 h-3" />
              <span>最长连续 {{ longestStreak }} 天</span>
            </span>
          </div>
          <div class="flex items-center bg-slate-100 rounded-lg p-0.5">
            <button
              v-for="opt in [{ id: 'completed' as Metric, label: '完成' }, { id: 'created' as Metric, label: '创建' }]"
              :key="opt.id"
              @click="metric = opt.id"
              :class="['px-2.5 py-1 rounded-md text-[12px] font-medium transition-all',
                       metric === opt.id ? 'bg-white text-slate-800 shadow-sm' : 'text-slate-500 hover:text-slate-700']"
            >
              {{ opt.label }}
            </button>
          </div>
        </div>

        <div class="flex items-end gap-[3px] h-36">
          <div
            v-for="d in trend"
            :key="d.date"
            class="flex-1 rounded-t transition-colors"
            :class="d.count > 0 ? 'bg-blue-500 hover:bg-blue-600' : 'bg-slate-100'"
            :style="{ height: d.count > 0 ? `${Math.max(4, (d.count / trendMax) * 100)}%` : '3px' }"
            :title="`${d.date}：${d.count} 项`"
          ></div>
        </div>

        <div class="flex justify-between mt-2 text-[11px] text-slate-400 tabular-nums">
          <span v-for="(t, i) in trendTicks" :key="i">{{ t }}</span>
        </div>
      </div>

      <!-- 分布 -->
      <div class="mt-4 grid grid-cols-2 gap-4">
        <div class="bg-white rounded-2xl border border-slate-100 shadow-sm px-4 py-4">
          <h3 class="text-[13px] font-medium text-slate-700 mb-4">未完成任务按项目分布</h3>

          <div v-if="(stats?.by_project ?? []).length === 0" class="py-6 text-center text-[12px] text-slate-400">
            暂无未完成任务
          </div>

          <div v-else class="space-y-3">
            <div v-for="b in stats?.by_project ?? []" :key="b.key" class="flex items-center">
              <span class="w-20 shrink-0 text-[12px] text-slate-600 truncate" :title="b.label">{{ b.label }}</span>
              <div class="flex-1 h-2.5 bg-slate-100 rounded-full overflow-hidden mr-3">
                <div
                  class="h-full rounded-full"
                  :style="{ width: `${(b.count / projectMax) * 100}%`, backgroundColor: b.color || '#cbd5e1' }"
                ></div>
              </div>
              <span class="w-8 text-right text-[12px] text-slate-500 tabular-nums">{{ b.count }}</span>
            </div>
          </div>
        </div>

        <div class="bg-white rounded-2xl border border-slate-100 shadow-sm px-4 py-4">
          <h3 class="text-[13px] font-medium text-slate-700 mb-4">未完成任务按优先级分布</h3>

          <div v-if="priorityTotal === 0" class="py-6 text-center text-[12px] text-slate-400">
            暂无未完成任务
          </div>

          <template v-else>
            <div class="flex h-4 rounded-full overflow-hidden gap-[2px]">
              <div
                v-for="b in stats?.by_priority ?? []"
                :key="b.key"
                class="h-full"
                :style="{ width: `${(b.count / priorityTotal) * 100}%`, backgroundColor: PRIORITY_COLOR[b.key] || '#CBD5E1' }"
                :title="`${b.label}：${b.count}`"
              ></div>
            </div>

            <div class="grid grid-cols-2 gap-x-6 gap-y-3 mt-5">
              <div v-for="b in stats?.by_priority ?? []" :key="b.key" class="flex items-center">
                <span
                  class="w-2 h-2 rounded-full mr-2 shrink-0"
                  :style="{ backgroundColor: PRIORITY_COLOR[b.key] || '#CBD5E1' }"
                ></span>
                <span class="flex-1 text-[12px] text-slate-600">{{ b.label }}优先级</span>
                <span class="text-[12px] text-slate-500 tabular-nums">{{ b.count }}</span>
              </div>
            </div>
          </template>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: transparent;
  border-radius: 4px;
}
.custom-scrollbar:hover::-webkit-scrollbar-thumb {
  background-color: #cbd5e1;
}
</style>

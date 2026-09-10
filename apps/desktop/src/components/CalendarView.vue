<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { api } from '../lib/api';
import { useTaskStore } from '../stores/taskStore';
import { useProjectStore } from '../stores/projectStore';
import { useUiStore } from '../stores/uiStore';
import { ChevronLeft, ChevronRight, Inbox, PanelRightClose, PanelRightOpen } from 'lucide-vue-next';
import type { DailyActivity, Task, TaskStatus } from '../types/task';
import { localDayBounds, toDateKey } from '../types/task';

const taskStore = useTaskStore();
const projectStore = useProjectStore();
const uiStore = useUiStore();

/** 统计口径：当天完成 / 当天创建 */
type Metric = 'completed' | 'created';
const metric = ref<Metric>('completed');
const metricOptions: { id: Metric; label: string }[] = [
  { id: 'completed', label: '完成' },
  { id: 'created', label: '创建' },
];

const weekdays = ['一', '二', '三', '四', '五', '六', '日'];

/** 当前显示的月份（定位到当月 1 号） */
const cursor = ref(new Date(new Date().getFullYear(), new Date().getMonth(), 1));
const selectedDate = ref(toDateKey(new Date()));

const monthTitle = computed(
  () => `${cursor.value.getFullYear()} 年 ${cursor.value.getMonth() + 1} 月`
);

/**
 * 月历网格：固定 6 行 × 7 列，周一为每周起始。
 * 固定行数是为了翻月时高度不跳动。
 */
const grid = computed(() => {
  const y = cursor.value.getFullYear();
  const m = cursor.value.getMonth();
  // JS 的 getDay() 里 0 = 周日，换算成「周一为 0」
  const lead = (new Date(y, m, 1).getDay() + 6) % 7;
  const cells: { key: string; day: number; inMonth: boolean }[] = [];
  for (let i = 0; i < 42; i++) {
    const d = new Date(y, m, i - lead + 1);
    cells.push({ key: toDateKey(d), day: d.getDate(), inMonth: d.getMonth() === m });
  }
  return cells;
});

// ---------- 数据 ----------

/** date(YYYY-MM-DD) → 当日汇总 */
const activity = ref<Record<string, DailyActivity>>({});
const dayTasks = ref<Task[]>([]);
const loadingDay = ref(false);

/** 网格覆盖范围（含前后补位的邻月日期），一次取回整屏数据 */
async function loadActivity() {
  const cells = grid.value;
  try {
    const list = await api.getDailyActivity(cells[0].key, cells[cells.length - 1].key);
    const map: Record<string, DailyActivity> = {};
    for (const a of list) map[a.date] = a;
    activity.value = map;
  } catch {
    activity.value = {};
  }
}

/** 选中日期的明细。复用 list_tasks，按当前口径换成对应的时间区间 */
async function loadDay() {
  loadingDay.value = true;
  const { from, to } = localDayBounds(selectedDate.value);
  const query =
    metric.value === 'completed'
      ? {
          statuses: ['completed' as TaskStatus],
          completed_from: from,
          completed_to: to,
          page: 1,
          page_size: 200,
        }
      : { created_from: from, created_to: to, page: 1, page_size: 200 };

  try {
    const res = await api.listTasks(query);
    dayTasks.value = res.items;
  } catch {
    dayTasks.value = [];
  } finally {
    loadingDay.value = false;
  }
}

onMounted(() => {
  loadActivity();
  loadDay();
});

watch(cursor, loadActivity);
watch([selectedDate, metric], loadDay);

// ---------- 热力分档 ----------

const metricCount = (key: string) => {
  const a = activity.value[key];
  if (!a) return 0;
  return metric.value === 'completed' ? a.completed_count : a.created_count;
};

/** 以当月峰值为基准分档，避免偶尔一天特别多就把其他天全压成浅色 */
const monthPeak = computed(() => {
  let peak = 0;
  for (const c of grid.value) {
    if (c.inMonth) peak = Math.max(peak, metricCount(c.key));
  }
  return peak;
});

const levelOf = (count: number) => {
  if (count <= 0) return 0;
  const peak = monthPeak.value;
  if (peak <= 1) return 1;
  const r = count / peak;
  if (r <= 0.25) return 1;
  if (r <= 0.5) return 2;
  if (r <= 0.75) return 3;
  return 4;
};

const LEVEL_BADGE = [
  'bg-transparent text-transparent',
  'bg-blue-100 text-blue-700',
  'bg-blue-200 text-blue-800',
  'bg-blue-400 text-white',
  'bg-blue-600 text-white',
];
const LEVEL_FILL = ['bg-blue-100', 'bg-blue-200', 'bg-blue-400', 'bg-blue-600'];

/** 格子里最多铺几条任务摘要（完整条数由角标体现，悬停可看全部摘要） */
const CELL_ITEM_LIMIT = 2;

const todayKey = toDateKey(new Date());

/** 当前口径下，某天要铺在格子里的任务摘要 */
const itemsOf = (key: string) => {
  const a = activity.value[key];
  if (!a) return [];
  const items = metric.value === 'completed' ? a.completed_items : a.created_items;
  return items.slice(0, CELL_ITEM_LIMIT);
};

const colorOf = (projectId: string) =>
  projectStore.projects.find(p => p.id === projectId)?.color || '#cbd5e1';

/** 悬停提示：把当天拿到的全部摘要列出来 */
const cellTooltip = (key: string) => {
  const a = activity.value[key];
  if (!a) return '';
  const items = metric.value === 'completed' ? a.completed_items : a.created_items;
  const total = metric.value === 'completed' ? a.completed_count : a.created_count;
  if (total === 0) return '';
  const head = items.map(i => i.title).join('\n');
  return total > items.length ? `${head}\n…共 ${total} 项` : head;
};

// ---------- 展示 ----------

const selectedLabel = computed(() => {
  const [y, m, d] = selectedDate.value.split('-').map(Number);
  const wd = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'][
    new Date(y, m - 1, d).getDay()
  ];
  return `${m} 月 ${d} 日 · ${wd}`;
});

const formatMinutes = (min: number) => {
  if (!min) return '';
  if (min < 60) return `${min} 分钟`;
  const h = Math.floor(min / 60);
  const m = min % 60;
  return m ? `${h} 小时 ${m} 分钟` : `${h} 小时`;
};

const daySummary = computed(() => {
  if (metric.value === 'created') return `创建 ${dayTasks.value.length} 项`;
  const minutes = dayTasks.value.reduce((sum, t) => sum + (t.time_spent || 0), 0);
  const base = `完成 ${dayTasks.value.length} 项`;
  return minutes > 0 ? `${base} · 累计 ${formatMinutes(minutes)}` : base;
});

const projectOf = (task: Task) => projectStore.projects.find(p => p.id === task.project_id);

// ---------- 交互 ----------

/**
 * 右侧明细面板可折叠。
 * 日历格子宽度很有限（7 列挤在几百像素里），收起面板能明显多显示几个字，
 * 所以把选择权交给用户，状态记在 localStorage。
 */
const DETAIL_PANEL_KEY = 'workmanager.calendarDetailOpen';
const detailOpen = ref(localStorage.getItem(DETAIL_PANEL_KEY) !== '0');
const toggleDetail = () => {
  detailOpen.value = !detailOpen.value;
  localStorage.setItem(DETAIL_PANEL_KEY, detailOpen.value ? '1' : '0');
};

const selectDate = (cell: { key: string; inMonth: boolean }) => {
  selectedDate.value = cell.key;
  if (!cell.inMonth) {
    const [y, m] = cell.key.split('-').map(Number);
    cursor.value = new Date(y, m - 1, 1);
  }
};

const shiftMonth = (delta: number) => {
  cursor.value = new Date(cursor.value.getFullYear(), cursor.value.getMonth() + delta, 1);
  selectedDate.value = toDateKey(cursor.value);
};

const goToday = () => {
  const now = new Date();
  cursor.value = new Date(now.getFullYear(), now.getMonth(), 1);
  selectedDate.value = toDateKey(now);
};

/** 点明细项跳回原任务。已完成的任务去「已完成」视图，否则去所属项目 */
const openTask = async (task: Task) => {
  projectStore.currentViewId = task.status === 'completed' ? 'completed' : task.project_id;
  await taskStore.loadTasks();
  if (taskStore.tasks.some(t => t.id === task.id)) {
    taskStore.expandedTaskId = task.id;
  } else {
    uiStore.showMessage('该任务已被当前时间范围过滤，请调整时间范围后查看', 'info');
  }
};
</script>

<template>
  <div class="flex-1 flex flex-col min-h-0">
    <!-- Toolbar -->
    <div class="flex items-center justify-between gap-4 px-6 pt-5 pb-4">
      <div class="flex items-center space-x-3">
        <h2 class="text-[15px] font-semibold text-slate-700 tabular-nums">{{ monthTitle }}</h2>
        <div class="flex items-center bg-slate-100 rounded-lg p-0.5">
          <button
            @click="shiftMonth(-1)"
            class="p-1 rounded-md text-slate-500 hover:text-slate-800 hover:bg-white transition-colors"
            title="上个月"
          >
            <ChevronLeft class="w-4 h-4" />
          </button>
          <button
            @click="goToday"
            class="px-2.5 py-0.5 rounded-md text-xs font-medium text-slate-600 hover:text-slate-900 hover:bg-white transition-colors"
          >
            今天
          </button>
          <button
            @click="shiftMonth(1)"
            class="p-1 rounded-md text-slate-500 hover:text-slate-800 hover:bg-white transition-colors"
            title="下个月"
          >
            <ChevronRight class="w-4 h-4" />
          </button>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <!-- 统计口径 -->
        <div class="flex items-center bg-slate-100 rounded-xl p-1 w-fit">
          <button
            v-for="opt in metricOptions"
            :key="opt.id"
            @click="metric = opt.id"
            :class="['px-3.5 py-1.5 rounded-lg text-[13px] font-medium transition-all duration-200',
                     metric === opt.id ? 'bg-white text-slate-800 shadow-sm' : 'text-slate-500 hover:text-slate-700']"
          >
            {{ opt.label }}
          </button>
        </div>

        <button
          @click="toggleDetail"
          :title="detailOpen ? '收起明细面板（日历可显示更多字）' : '展开明细面板'"
          class="p-2 rounded-lg text-slate-400 hover:text-slate-700 hover:bg-slate-100 transition-colors"
        >
          <PanelRightClose v-if="detailOpen" class="w-4 h-4" />
          <PanelRightOpen v-else class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Calendar + Detail -->
    <div class="flex-1 flex min-h-0 gap-3 px-6 pb-6">
      <!-- Month grid -->
      <div class="flex-1 min-w-0 bg-white rounded-2xl border border-slate-100 shadow-sm p-3 flex flex-col min-h-0">
        <div class="grid grid-cols-7 gap-1 mb-1">
          <div v-for="w in weekdays" :key="w" class="text-center text-[11px] text-slate-400 py-1">{{ w }}</div>
        </div>

        <div class="grid grid-cols-7 grid-rows-6 gap-1 flex-1 min-h-0">
          <button
            v-for="cell in grid"
            :key="cell.key"
            @click="selectDate(cell)"
            :title="cellTooltip(cell.key)"
            :class="['flex flex-col rounded-lg border p-1 text-left overflow-hidden transition-all',
                     cell.inMonth ? 'bg-white border-slate-100' : 'bg-slate-50/70 border-transparent opacity-45',
                     selectedDate === cell.key
                       ? 'ring-2 ring-blue-500 border-blue-200'
                       : 'hover:border-blue-300 hover:bg-blue-50/40']"
          >
            <div class="flex items-center justify-between gap-1 shrink-0">
              <span
                class="text-[11px] font-medium tabular-nums leading-none"
                :class="cell.key === todayKey ? 'text-blue-600' : (cell.inMonth ? 'text-slate-600' : 'text-slate-400')"
              >{{ cell.day }}</span>
              <span
                v-if="metricCount(cell.key) > 0"
                :class="['text-[10px] font-semibold tabular-nums leading-none px-1 py-0.5 rounded',
                         LEVEL_BADGE[levelOf(metricCount(cell.key))]]"
              >{{ metricCount(cell.key) }}</span>
            </div>

            <div class="flex-1 min-h-0 mt-1 space-y-[2px] overflow-hidden">
              <div
                v-for="(it, i) in itemsOf(cell.key)"
                :key="i"
                class="flex items-center text-[10px] leading-[13px] text-slate-600 overflow-hidden"
              >
                <span
                  class="w-[3px] h-[3px] rounded-full shrink-0 mr-1"
                  :style="{ backgroundColor: colorOf(it.project_id) }"
                ></span>
                <span class="truncate">{{ it.title }}</span>
              </div>
            </div>
          </button>
        </div>

        <!-- Legend -->
        <div class="flex items-center justify-end gap-1.5 pt-2.5 text-[11px] text-slate-400">
          <span>{{ metric === 'completed' ? '每日完成数' : '每日创建数' }}</span>
          <span v-for="(fill, i) in LEVEL_FILL" :key="i" class="w-3 h-3 rounded-[3px]" :class="fill"></span>
          <span>少 → 多</span>
        </div>
      </div>

      <!-- Day detail -->
      <div
        v-if="detailOpen"
        class="w-64 shrink-0 bg-white rounded-2xl border border-slate-100 shadow-sm flex flex-col overflow-hidden"
      >
        <div class="px-5 py-4 border-b border-slate-50">
          <p class="text-[14px] font-semibold text-slate-800">{{ selectedLabel }}</p>
          <p class="text-[11px] text-slate-400 mt-1">{{ daySummary }}</p>
        </div>

        <div class="flex-1 overflow-y-auto px-3 py-3 custom-scrollbar">
          <div v-if="loadingDay" class="flex justify-center py-10">
            <div class="w-5 h-5 border-2 border-slate-200 border-t-blue-500 rounded-full animate-spin"></div>
          </div>

          <div v-else-if="dayTasks.length === 0" class="flex flex-col items-center justify-center py-12 text-slate-400">
            <Inbox class="w-7 h-7 text-slate-200 mb-2" />
            <p class="text-xs">这一天没有记录</p>
          </div>

          <template v-else>
            <button
              v-for="t in dayTasks"
              :key="t.id"
              @click="openTask(t)"
              class="w-full flex items-start px-2.5 py-2 rounded-lg hover:bg-slate-50 transition-colors text-left"
              title="跳转到该任务"
            >
              <span
                class="w-1.5 h-1.5 rounded-full mt-1.5 mr-2.5 shrink-0"
                :style="{ backgroundColor: projectOf(t)?.color || '#cbd5e1' }"
              ></span>
              <span class="flex-1 min-w-0">
                <span
                  class="block text-[13px] truncate"
                  :class="t.status === 'completed' ? 'text-slate-400 line-through' : 'text-slate-700'"
                >{{ t.title }}</span>
                <span class="block text-[11px] text-slate-400 mt-0.5">
                  {{ projectOf(t)?.name || '未分类' }}<template v-if="t.time_spent"> · {{ formatMinutes(t.time_spent) }}</template>
                </span>
              </span>
            </button>
          </template>
        </div>
      </div>
    </div>
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

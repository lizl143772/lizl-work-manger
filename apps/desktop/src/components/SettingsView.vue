<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { api } from '../lib/api';
import { useUiStore } from '../stores/uiStore';
import { useSettingsStore } from '../stores/settingsStore';
import { useTaskStore } from '../stores/taskStore';
import { DATE_RANGE_OPTIONS, type DateRangeKey, type AppInfo } from '../types/task';
import { Copy, RotateCcw, Trash2 } from 'lucide-vue-next';

const uiStore = useUiStore();
const settingsStore = useSettingsStore();
const taskStore = useTaskStore();

const appInfo = ref<AppInfo | null>(null);

onMounted(async () => {
  try {
    appInfo.value = await api.getAppInfo();
  } catch {
    // 拿不到也不影响其它设置项
  }
});

/** 把输入框里的值夹到合法区间；非法输入回落到默认值 */
function clampSize(raw: string, min: number, max: number, fallback: number) {
  const n = Math.round(Number(raw));
  if (!Number.isFinite(n)) return fallback;
  return Math.min(max, Math.max(min, n));
}

function onDateRangeChange(e: Event) {
  const key = (e.target as HTMLSelectElement).value as DateRangeKey;
  taskStore.setDateRange(key);
}

/** 置顶要走 uiStore，才能同时生效到窗口并保持两处状态一致 */
async function setAlwaysOnTop(value: boolean) {
  if (uiStore.isPinned !== value) await uiStore.togglePin();
}

async function copyDbPath() {
  const path = appInfo.value?.db_path;
  if (!path) return;
  try {
    await navigator.clipboard.writeText(path);
    uiStore.showMessage('数据库路径已复制', 'success');
  } catch {
    uiStore.showMessage('复制失败，请手动选中复制', 'error');
  }
}

async function emptyTrash() {
  const ok = await uiStore.confirm(
    '清空回收站？',
    '回收站里的全部任务会被永久删除，此操作无法撤销。'
  );
  if (!ok) return;
  try {
    const n = await api.purgeDeletedTasks();
    uiStore.showMessage(n > 0 ? `已彻底删除 ${n} 条任务` : '回收站已经是空的', 'info');
  } catch (e: any) {
    uiStore.showMessage(e.message || '清空失败', 'error');
  }
}

async function resetSettings() {
  const ok = await uiStore.confirm('恢复默认设置？', '所有偏好将回到初始状态，任务数据不受影响。');
  if (!ok) return;
  settingsStore.reset();
  // 置顶的运行时状态在 uiStore 里，不走 togglePin 同步的话，
  // 存储值已回 false、窗口却仍保持置顶，重启前设置页开关会一直显示错误状态
  if (uiStore.isPinned) await uiStore.togglePin();
  uiStore.showMessage('已恢复默认设置', 'success');
}
</script>

<template>
  <div class="flex-1 min-h-0 overflow-y-auto custom-scrollbar px-6 pt-5 pb-6">
    <div class="max-w-3xl space-y-4">

      <!-- 外观与行为 -->
      <section class="bg-white rounded-2xl border border-slate-100 shadow-sm px-5 py-2">
        <h3 class="text-[13px] font-medium text-slate-700 pt-3 pb-1">外观与行为</h3>

        <div class="flex items-center justify-between py-3 border-t border-slate-50">
          <div>
            <p class="text-[13px] text-slate-700">默认时间范围</p>
            <p class="text-[11px] text-slate-400 mt-0.5">任务列表默认展示哪个时间区间</p>
          </div>
          <select
            :value="settingsStore.settings.dateRange"
            @change="onDateRangeChange"
            class="bg-slate-100 rounded-lg px-3 py-1.5 text-[13px] text-slate-700 outline-none focus:ring-2 focus:ring-blue-200 cursor-pointer"
          >
            <option v-for="opt in DATE_RANGE_OPTIONS" :key="opt.key" :value="opt.key">{{ opt.label }}</option>
          </select>
        </div>

        <div class="flex items-center justify-between py-3 border-t border-slate-50">
          <div>
            <p class="text-[13px] text-slate-700">侧边栏默认页</p>
            <p class="text-[11px] text-slate-400 mt-0.5">每次打开应用时侧边栏停在哪个 Tab</p>
          </div>
          <div class="flex items-center bg-slate-100 rounded-lg p-0.5">
            <button
              v-for="opt in [{ id: 'common' as const, label: '常用' }, { id: 'menu' as const, label: '菜单' }]"
              :key="opt.id"
              @click="settingsStore.patch({ sidebarTab: opt.id })"
              :class="['px-3 py-1 rounded-md text-[12px] font-medium transition-all',
                       settingsStore.settings.sidebarTab === opt.id ? 'bg-white text-slate-800 shadow-sm' : 'text-slate-500 hover:text-slate-700']"
            >
              {{ opt.label }}
            </button>
          </div>
        </div>

        <div class="flex items-center justify-between py-3 border-t border-slate-50">
          <div>
            <p class="text-[13px] text-slate-700">日历默认统计口径</p>
            <p class="text-[11px] text-slate-400 mt-0.5">日历与趋势图默认按哪个口径统计</p>
          </div>
          <div class="flex items-center bg-slate-100 rounded-lg p-0.5">
            <button
              v-for="opt in [{ id: 'completed' as const, label: '完成' }, { id: 'created' as const, label: '创建' }]"
              :key="opt.id"
              @click="settingsStore.patch({ calendarMetric: opt.id })"
              :class="['px-3 py-1 rounded-md text-[12px] font-medium transition-all',
                       settingsStore.settings.calendarMetric === opt.id ? 'bg-white text-slate-800 shadow-sm' : 'text-slate-500 hover:text-slate-700']"
            >
              {{ opt.label }}
            </button>
          </div>
        </div>

        <div class="flex items-center justify-between py-3 border-t border-slate-50">
          <div>
            <p class="text-[13px] text-slate-700">日历明细面板默认展开</p>
            <p class="text-[11px] text-slate-400 mt-0.5">收起后日历能铺满宽度，格子里显示的字更多</p>
          </div>
          <button
            @click="settingsStore.patch({ calendarDetailOpen: !settingsStore.settings.calendarDetailOpen })"
            :class="['relative w-9 h-5 rounded-full transition-colors shrink-0',
                     settingsStore.settings.calendarDetailOpen ? 'bg-blue-600' : 'bg-slate-300']"
            :title="settingsStore.settings.calendarDetailOpen ? '已展开' : '已收起'"
          >
            <span
              :class="['absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-all',
                       settingsStore.settings.calendarDetailOpen ? 'left-[18px]' : 'left-0.5']"
            ></span>
          </button>
        </div>

        <div class="flex items-center justify-between py-3 border-t border-slate-50">
          <div>
            <p class="text-[13px] text-slate-700">每周起始日</p>
            <p class="text-[11px] text-slate-400 mt-0.5">影响日历表头的排列</p>
          </div>
          <div class="flex items-center bg-slate-100 rounded-lg p-0.5">
            <button
              v-for="opt in [{ id: 1 as const, label: '周一' }, { id: 0 as const, label: '周日' }]"
              :key="opt.id"
              @click="settingsStore.patch({ weekStartsOn: opt.id })"
              :class="['px-3 py-1 rounded-md text-[12px] font-medium transition-all',
                       settingsStore.settings.weekStartsOn === opt.id ? 'bg-white text-slate-800 shadow-sm' : 'text-slate-500 hover:text-slate-700']"
            >
              {{ opt.label }}
            </button>
          </div>
        </div>
      </section>

      <!-- 窗口 -->
      <section class="bg-white rounded-2xl border border-slate-100 shadow-sm px-5 py-2">
        <h3 class="text-[13px] font-medium text-slate-700 pt-3 pb-1">窗口</h3>

        <div class="flex items-center justify-between py-3 border-t border-slate-50">
          <div>
            <p class="text-[13px] text-slate-700">保持置顶</p>
            <p class="text-[11px] text-slate-400 mt-0.5">窗口始终显示在最前面，启动时自动生效</p>
          </div>
          <button
            @click="setAlwaysOnTop(!uiStore.isPinned)"
            :class="['relative w-9 h-5 rounded-full transition-colors shrink-0',
                     uiStore.isPinned ? 'bg-blue-600' : 'bg-slate-300']"
          >
            <span
              :class="['absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-all',
                       uiStore.isPinned ? 'left-[18px]' : 'left-0.5']"
            ></span>
          </button>
        </div>

        <div class="flex items-center justify-between py-3 border-t border-slate-50">
          <div>
            <p class="text-[13px] text-slate-700">小窗尺寸</p>
            <p class="text-[11px] text-slate-400 mt-0.5">切换到小窗模式时使用的窗口大小（最小 320 × 400）</p>
          </div>
          <div class="flex items-center gap-2 shrink-0">
            <input
              type="number"
              min="320"
              max="1200"
              :value="settingsStore.settings.miniWidth"
              @change="settingsStore.patch({ miniWidth: clampSize(($event.target as HTMLInputElement).value, 320, 1200, 380) })"
              class="w-20 bg-slate-100 rounded-lg px-2.5 py-1.5 text-[13px] text-slate-700 outline-none focus:ring-2 focus:ring-blue-200 tabular-nums"
            />
            <span class="text-[12px] text-slate-400">×</span>
            <input
              type="number"
              min="400"
              max="1200"
              :value="settingsStore.settings.miniHeight"
              @change="settingsStore.patch({ miniHeight: clampSize(($event.target as HTMLInputElement).value, 400, 1200, 560) })"
              class="w-20 bg-slate-100 rounded-lg px-2.5 py-1.5 text-[13px] text-slate-700 outline-none focus:ring-2 focus:ring-blue-200 tabular-nums"
            />
          </div>
        </div>
      </section>

      <!-- 数据 -->
      <section class="bg-white rounded-2xl border border-slate-100 shadow-sm px-5 py-2">
        <h3 class="text-[13px] font-medium text-slate-700 pt-3 pb-1">数据</h3>

        <div class="py-3 border-t border-slate-50">
          <p class="text-[13px] text-slate-700">数据库位置</p>
          <div class="flex items-center gap-2 mt-2">
            <code class="flex-1 min-w-0 truncate bg-slate-50 border border-slate-100 rounded-lg px-3 py-2 text-[12px] text-slate-600">
              {{ appInfo?.db_path || '读取中…' }}
            </code>
            <button
              @click="copyDbPath"
              :disabled="!appInfo"
              class="flex items-center space-x-1 px-2.5 py-2 rounded-lg text-[12px] text-slate-600 bg-slate-100 hover:bg-slate-200 transition-colors shrink-0 disabled:opacity-40"
            >
              <Copy class="w-3 h-3" />
              <span>复制</span>
            </button>
          </div>
          <p class="text-[11px] text-slate-400 mt-1.5">所有数据都存本地这个 SQLite 文件里，复制它即可做备份</p>
        </div>

        <div class="flex items-center justify-between py-3 border-t border-slate-50">
          <div>
            <p class="text-[13px] text-slate-700">回收站保留天数</p>
            <p class="text-[11px] text-slate-400 mt-0.5">超期自动清理；填 0 表示永久保留、不自动清理</p>
          </div>
          <input
            type="number"
            min="0"
            max="365"
            :value="settingsStore.settings.trashRetentionDays"
            @change="settingsStore.patch({ trashRetentionDays: clampSize(($event.target as HTMLInputElement).value, 0, 365, 30) })"
            class="w-20 shrink-0 bg-slate-100 rounded-lg px-2.5 py-1.5 text-[13px] text-slate-700 outline-none focus:ring-2 focus:ring-blue-200 tabular-nums"
          />
        </div>

        <div class="flex items-center justify-between py-3 border-t border-slate-50">
          <div>
            <p class="text-[13px] text-slate-700">清空回收站</p>
            <p class="text-[11px] text-slate-400 mt-0.5">永久删除回收站里的全部任务，无法撤销</p>
          </div>
          <button
            @click="emptyTrash"
            class="flex items-center space-x-1 px-3 py-1.5 rounded-lg text-[12px] text-rose-600 bg-rose-50 hover:bg-rose-100 transition-colors shrink-0"
          >
            <Trash2 class="w-3 h-3" />
            <span>清空</span>
          </button>
        </div>

        <div class="flex items-center justify-between py-3 border-t border-slate-50">
          <div>
            <p class="text-[13px] text-slate-700">恢复默认设置</p>
            <p class="text-[11px] text-slate-400 mt-0.5">只重置偏好，不影响任何任务数据</p>
          </div>
          <button
            @click="resetSettings"
            class="flex items-center space-x-1 px-3 py-1.5 rounded-lg text-[12px] text-slate-600 bg-slate-100 hover:bg-slate-200 transition-colors shrink-0"
          >
            <RotateCcw class="w-3 h-3" />
            <span>恢复默认</span>
          </button>
        </div>
      </section>

      <!-- 关于 -->
      <section class="bg-white rounded-2xl border border-slate-100 shadow-sm px-5 py-2">
        <h3 class="text-[13px] font-medium text-slate-700 pt-3 pb-1">关于</h3>
        <div class="flex items-center justify-between py-3 border-t border-slate-50">
          <p class="text-[13px] text-slate-700">版本</p>
          <p class="text-[13px] text-slate-500 tabular-nums">{{ appInfo?.version || '—' }}</p>
        </div>
        <div class="flex items-center justify-between py-3 border-t border-slate-50">
          <p class="text-[13px] text-slate-700">数据库 schema 版本</p>
          <p class="text-[13px] text-slate-500 tabular-nums">v{{ appInfo?.schema_version ?? '—' }}</p>
        </div>
        <div class="flex items-center justify-between py-3 border-t border-slate-50">
          <p class="text-[13px] text-slate-700">技术栈</p>
          <p class="text-[13px] text-slate-500">Tauri 2 · Vue 3 · Rust · SQLite</p>
        </div>
        <p class="text-[11px] text-slate-400 py-3 border-t border-slate-50">
          数据完全存于本地，无云端同步。建议定期复制数据库文件做备份。
        </p>
      </section>

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

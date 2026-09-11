<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { api } from '../lib/api';
import { useProjectStore } from '../stores/projectStore';
import { useUiStore } from '../stores/uiStore';
import { useSettingsStore } from '../stores/settingsStore';
import { Search, Trash2, RotateCcw, Inbox, Check } from 'lucide-vue-next';
import type { Task } from '../types/task';

const projectStore = useProjectStore();
const uiStore = useUiStore();
const settingsStore = useSettingsStore();

const tasks = ref<Task[]>([]);
const total = ref(0);
const keyword = ref('');
const loading = ref(false);
const selected = ref<Set<string>>(new Set());

const STATUS_LABEL: Record<string, string> = {
  todo: '待办',
  in_progress: '进行中',
  completed: '已完成',
};

async function load() {
  loading.value = true;
  try {
    const res = await api.listDeletedTasks(keyword.value.trim() || null, 1, 200);
    tasks.value = res.items;
    total.value = res.total;
    // 丢弃已经不在列表里的选中项，避免批量操作打到不存在的记录
    const ids = new Set(res.items.map(t => t.id));
    selected.value = new Set([...selected.value].filter(id => ids.has(id)));
  } catch (e: any) {
    uiStore.showMessage(e.message || '加载回收站失败', 'error');
  } finally {
    loading.value = false;
  }
}

onMounted(load);

// 搜索防抖，避免每敲一个字就查一次
let searchTimer: number | null = null;
watch(keyword, () => {
  if (searchTimer) clearTimeout(searchTimer);
  searchTimer = window.setTimeout(load, 250);
});

const projectOf = (task: Task) => projectStore.projects.find(p => p.id === task.project_id);

/** 「3 天前删除」这种相对时间，比绝对时间更符合回收站的使用场景 */
function relativeTime(iso: string | null) {
  if (!iso) return '';
  const diff = Date.now() - new Date(iso).getTime();
  const minutes = Math.floor(diff / 60000);
  if (minutes < 1) return '刚刚';
  if (minutes < 60) return `${minutes} 分钟前`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours} 小时前`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days} 天前`;
  const months = Math.floor(days / 30);
  if (months < 12) return `${months} 个月前`;
  return `${Math.floor(months / 12)} 年前`;
}

const retentionDays = computed(() => settingsStore.settings.trashRetentionDays);
const retentionHint = computed(() =>
  retentionDays.value > 0
    ? `已删除的任务保留 ${retentionDays.value} 天后自动清理（保留天数可在设置里调整）`
    : '已删除的任务将永久保留，不会自动清理（可在设置里调整）'
);

// ---------- 选择 ----------

const allSelected = computed(
  () => tasks.value.length > 0 && selected.value.size === tasks.value.length
);

function toggleSelect(id: string) {
  const next = new Set(selected.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  selected.value = next;
}

function toggleSelectAll() {
  selected.value = allSelected.value ? new Set() : new Set(tasks.value.map(t => t.id));
}

// ---------- 操作 ----------

async function restore(task: Task) {
  try {
    await api.restoreTask(task.id);
    uiStore.showMessage(`已恢复「${task.title}」`, 'success');
    await load();
    await projectStore.loadCounts();
  } catch (e: any) {
    uiStore.showMessage(e.message || '恢复失败', 'error');
  }
}

async function restoreSelected() {
  const ids = [...selected.value];
  if (ids.length === 0) return;
  const ok = await uiStore.confirm(
    `恢复选中的 ${ids.length} 条任务？`,
    '任务会回到原来所属的项目。'
  );
  if (!ok) return;

  let done = 0;
  for (const id of ids) {
    try {
      await api.restoreTask(id);
      done += 1;
    } catch {
      // 单条失败不中断整批
    }
  }
  selected.value = new Set();
  uiStore.showMessage(
    done === ids.length ? `已恢复 ${done} 条任务` : `已恢复 ${done} 条，${ids.length - done} 条失败`,
    done === ids.length ? 'success' : 'error'
  );
  await load();
  await projectStore.loadCounts();
}

async function purge(task: Task) {
  const ok = await uiStore.confirm(
    '彻底删除该任务？',
    `「${task.title}」将被永久删除，无法再恢复。`
  );
  if (!ok) return;
  try {
    await api.purgeTask(task.id);
    uiStore.showMessage('已彻底删除', 'info');
    await load();
  } catch (e: any) {
    uiStore.showMessage(e.message || '删除失败', 'error');
  }
}

async function purgeSelected() {
  const ids = [...selected.value];
  if (ids.length === 0) return;
  const ok = await uiStore.confirm(
    `彻底删除选中的 ${ids.length} 条任务？`,
    '这些任务将被永久删除，无法再恢复。'
  );
  if (!ok) return;

  let done = 0;
  for (const id of ids) {
    try {
      await api.purgeTask(id);
      done += 1;
    } catch {
      // 单条失败不中断整批
    }
  }
  selected.value = new Set();
  uiStore.showMessage(`已彻底删除 ${done} 条任务`, 'info');
  await load();
}

async function purgeAll() {
  if (total.value === 0) return;
  const ok = await uiStore.confirm(
    '清空回收站？',
    `将永久删除其中全部 ${total.value} 条任务，此操作无法撤销。`
  );
  if (!ok) return;
  try {
    const n = await api.purgeDeletedTasks();
    selected.value = new Set();
    uiStore.showMessage(`已彻底删除 ${n} 条任务`, 'info');
    await load();
  } catch (e: any) {
    uiStore.showMessage(e.message || '清空失败', 'error');
  }
}
</script>

<template>
  <div class="flex-1 flex flex-col min-h-0 px-6 pt-5 pb-6">
    <!-- Toolbar -->
    <div class="flex items-center gap-3 pb-4">
      <div class="relative flex items-center">
        <Search class="w-3.5 h-3.5 text-slate-400 absolute left-2.5 pointer-events-none" />
        <input
          v-model="keyword"
          type="text"
          placeholder="搜索已删除的任务…"
          class="w-64 bg-slate-100 rounded-lg pl-8 pr-3 py-2 text-[13px] text-slate-700 placeholder:text-slate-400 outline-none focus:ring-2 focus:ring-blue-200"
        />
      </div>

      <span class="text-[12px] text-slate-400">共 {{ total }} 条</span>

      <div class="flex-1"></div>

      <button
        @click="purgeAll"
        :disabled="total === 0"
        class="px-3.5 py-2 rounded-lg text-[13px] font-medium text-rose-600 bg-rose-50 hover:bg-rose-100 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
      >
        清空回收站
      </button>
    </div>

    <!-- Batch bar -->
    <div
      v-if="selected.size > 0"
      class="flex items-center gap-3 px-4 py-2 mb-3 rounded-xl bg-blue-50 border border-blue-100"
    >
      <span class="text-[13px] text-blue-700 font-medium">已选中 {{ selected.size }} 条</span>
      <div class="flex-1"></div>
      <button
        @click="restoreSelected"
        class="px-3 py-1.5 rounded-lg text-[13px] text-blue-700 bg-white hover:bg-blue-100 border border-blue-200 transition-colors"
      >
        批量恢复
      </button>
      <button
        @click="purgeSelected"
        class="px-3 py-1.5 rounded-lg text-[13px] text-rose-600 bg-white hover:bg-rose-50 border border-rose-200 transition-colors"
      >
        批量彻底删除
      </button>
    </div>

    <!-- List -->
    <div class="flex-1 min-h-0 overflow-y-auto custom-scrollbar">
      <div v-if="loading" class="flex justify-center py-16">
        <div class="w-6 h-6 border-2 border-slate-200 border-t-blue-500 rounded-full animate-spin"></div>
      </div>

      <div
        v-else-if="tasks.length === 0"
        class="flex flex-col items-center justify-center py-24 text-slate-400"
      >
        <div class="w-16 h-16 bg-slate-100 rounded-full flex items-center justify-center mb-5">
          <Inbox class="w-8 h-8 text-slate-300" />
        </div>
        <p class="text-[14px] font-medium text-slate-500">
          {{ keyword.trim() ? '没有匹配的已删除任务' : '回收站是空的' }}
        </p>
        <p class="text-[12px] text-slate-400 mt-1">
          {{ keyword.trim() ? '换个关键字试试' : '删除的任务会先放到这里，可随时恢复' }}
        </p>
      </div>

      <template v-else>
        <!-- 全选 -->
        <div class="flex items-center px-4 py-2 mb-1">
          <button @click="toggleSelectAll" class="flex items-center space-x-2 text-[12px] text-slate-500 hover:text-slate-700">
            <span
              class="w-3.5 h-3.5 rounded-[4px] border flex items-center justify-center transition-colors"
              :class="allSelected ? 'bg-blue-600 border-blue-600' : 'bg-white border-slate-300'"
            >
              <Check v-if="allSelected" class="w-2.5 h-2.5 text-white" />
            </span>
            <span>全选</span>
          </button>
        </div>

        <div
          v-for="task in tasks"
          :key="task.id"
          class="group flex items-center px-4 py-3 mb-1.5 rounded-xl border border-slate-100 bg-white hover:border-slate-200 transition-colors"
        >
          <button @click="toggleSelect(task.id)" class="mr-3 flex-shrink-0">
            <span
              class="w-3.5 h-3.5 rounded-[4px] border flex items-center justify-center transition-colors"
              :class="selected.has(task.id) ? 'bg-blue-600 border-blue-600' : 'bg-white border-slate-300'"
            >
              <Check v-if="selected.has(task.id)" class="w-2.5 h-2.5 text-white" />
            </span>
          </button>

          <div class="flex-1 min-w-0">
            <div class="text-[14px] text-slate-700 truncate">{{ task.title }}</div>
            <div class="flex items-center gap-2 mt-1 text-[11px] text-slate-400">
              <span class="flex items-center gap-1">
                <span
                  class="w-1.5 h-1.5 rounded-full"
                  :style="{ backgroundColor: projectOf(task)?.color || '#cbd5e1' }"
                ></span>
                <span>{{ projectOf(task)?.name || '未分类' }}</span>
              </span>
              <span>·</span>
              <span>{{ STATUS_LABEL[task.status] || task.status }}</span>
              <span>·</span>
              <span>{{ relativeTime(task.deleted_at) }}删除</span>
            </div>
          </div>

          <button
            @click="restore(task)"
            class="ml-3 flex items-center space-x-1 px-2.5 py-1.5 rounded-lg text-[12px] text-blue-600 bg-blue-50 hover:bg-blue-100 transition-colors flex-shrink-0"
            title="恢复到原项目"
          >
            <RotateCcw class="w-3 h-3" />
            <span>恢复</span>
          </button>
          <button
            @click="purge(task)"
            class="ml-2 flex items-center space-x-1 px-2.5 py-1.5 rounded-lg text-[12px] text-rose-600 bg-rose-50 hover:bg-rose-100 transition-colors flex-shrink-0"
            title="永久删除，无法恢复"
          >
            <Trash2 class="w-3 h-3" />
            <span>彻底删除</span>
          </button>
        </div>
      </template>
    </div>

    <p class="pt-3 text-[11px] text-slate-400">{{ retentionHint }}</p>
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

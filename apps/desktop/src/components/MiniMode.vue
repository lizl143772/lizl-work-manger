<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { api } from '../lib/api';
import { useProjectStore } from '../stores/projectStore';
import type { Task } from '../types/task';
import { Circle, PlayCircle, Inbox, Calendar } from 'lucide-vue-next';

const projectStore = useProjectStore();

const tasks = ref<Task[]>([]);
const loading = ref(true);
const activeTab = ref<'in_progress' | 'todo'>('in_progress');

const load = async () => {
  loading.value = true;
  try {
    const res = await api.listTasks({ statuses: ['todo', 'in_progress'], page: 1, page_size: 200 });
    tasks.value = res.items;
  } catch {
    // keep previous list on error
  } finally {
    loading.value = false;
  }
};

onMounted(load);

const inProgressCount = computed(() => tasks.value.filter(t => t.status === 'in_progress').length);
const todoCount = computed(() => tasks.value.filter(t => t.status === 'todo').length);
const filtered = computed(() => tasks.value.filter(t => t.status === activeTab.value));

const projectOf = (task: Task) => projectStore.projects.find(p => p.id === task.project_id);

const formatDate = (iso: string) => {
  return new Date(iso).toLocaleString('zh-CN', {
    month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit'
  });
};

const toggleStatus = async (task: Task) => {
  const next = task.status === 'todo' ? 'in_progress' : 'todo';
  await api.updateTaskStatus(task.id, next);
  await load();
};

const priorityColor = (p: number) => {
  switch (p) {
    case 1: return '#3b82f6';
    case 2: return '#f59e0b';
    case 3: return '#f43f5e';
    default: return 'transparent';
  }
};
</script>

<template>
  <div class="flex-1 flex flex-col min-w-0 bg-white overflow-hidden">
    <!-- Mini Header: Status Tabs -->
    <div class="flex items-center px-3 pt-3 pb-2 border-b border-slate-100">
      <div class="flex items-center space-x-1 bg-slate-100 rounded-lg p-0.5">
        <button
          @click="activeTab = 'in_progress'"
          :class="['px-2.5 py-1 rounded-md text-xs font-medium transition-all',
                   activeTab === 'in_progress' ? 'bg-white text-blue-600 shadow-sm' : 'text-slate-500 hover:text-slate-700']"
        >
          进行中 {{ inProgressCount }}
        </button>
        <button
          @click="activeTab = 'todo'"
          :class="['px-2.5 py-1 rounded-md text-xs font-medium transition-all',
                   activeTab === 'todo' ? 'bg-white text-slate-800 shadow-sm' : 'text-slate-500 hover:text-slate-700']"
        >
          未开始 {{ todoCount }}
        </button>
      </div>
    </div>

    <!-- Task List -->
    <div class="flex-1 overflow-y-auto py-2">
      <div v-if="loading && tasks.length === 0" class="flex justify-center py-16 text-slate-400">
        <div class="w-6 h-6 border-2 border-slate-200 border-t-blue-500 rounded-full animate-spin"></div>
      </div>

      <div v-else-if="filtered.length === 0" class="flex flex-col items-center justify-center py-16 text-slate-400">
        <Inbox class="w-8 h-8 text-slate-200 mb-2" />
        <p class="text-xs">暂无任务</p>
      </div>

      <div
        v-else
        v-for="task in filtered"
        :key="task.id"
        class="flex items-center px-2.5 py-2 mx-2 mb-1 rounded-lg hover:bg-slate-50 border-l-[3px] transition-colors"
        :style="{ borderLeftColor: priorityColor(task.priority) }"
      >
        <button @click="toggleStatus(task)" class="mr-2.5 flex-shrink-0 transition-transform active:scale-95" :title="task.status === 'todo' ? '开始任务' : '移回未开始'">
          <Circle v-if="task.status === 'todo'" class="w-[18px] h-[18px] text-slate-300 hover:text-blue-500 transition-colors" />
          <PlayCircle v-else class="w-[18px] h-[18px] text-blue-500" />
        </button>
        <div class="flex-1 min-w-0">
          <div class="truncate text-[13px] text-slate-700">{{ task.title }}</div>
          <div class="flex items-center gap-2 mt-0.5 text-[11px] text-slate-400">
            <span v-if="projectOf(task)" class="flex items-center gap-1 flex-shrink-0">
              <span class="w-1.5 h-1.5 rounded-full" :style="{ backgroundColor: projectOf(task)!.color }"></span>
              <span>{{ projectOf(task)!.name }}</span>
            </span>
            <span v-if="task.due_date" class="flex items-center gap-0.5" title="任务时间">
              <Calendar class="w-3 h-3" />
              <span>{{ formatDate(task.due_date) }}</span>
            </span>
            <span class="flex items-center gap-0.5" title="录入时间">
              <span>录入 {{ formatDate(task.created_at) }}</span>
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

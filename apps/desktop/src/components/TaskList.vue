<script setup lang="ts">
import { ref, computed } from 'vue';
import { useTaskStore } from '../stores/taskStore';
import { useProjectStore } from '../stores/projectStore';
import TaskItem from './TaskItem.vue';
import { Inbox, RotateCcw } from 'lucide-vue-next';

const taskStore = useTaskStore();
const projectStore = useProjectStore();

const isCompletedView = computed(() => projectStore.currentViewId === 'completed');

type StatusTab = 'all' | 'todo' | 'in_progress';
const activeTab = ref<StatusTab>('all');

const todoCount = computed(() => taskStore.tasks.filter(t => t.status === 'todo').length);
const inProgressCount = computed(() => taskStore.tasks.filter(t => t.status === 'in_progress').length);

const tabs = computed(() => [
  { id: 'all' as StatusTab, label: '全部', count: taskStore.tasks.length },
  { id: 'todo' as StatusTab, label: '未开始', count: todoCount.value },
  { id: 'in_progress' as StatusTab, label: '进行中', count: inProgressCount.value },
]);

const filteredTasks = computed(() => {
  if (activeTab.value === 'todo') return taskStore.tasks.filter(t => t.status === 'todo');
  if (activeTab.value === 'in_progress') return taskStore.tasks.filter(t => t.status === 'in_progress');
  return taskStore.tasks;
});
</script>

<template>
  <div class="flex-1 overflow-y-auto relative pt-4 pb-24 scroll-smooth">
    
    <!-- Undo Toast -->
    <Transition name="fade-up">
      <div 
        v-if="taskStore.lastDeletedTask" 
        class="fixed bottom-8 left-1/2 -translate-x-1/2 bg-slate-800 text-slate-100 px-5 py-3 rounded-full shadow-xl flex items-center space-x-4 z-50 border border-slate-700/50"
      >
        <span class="text-sm font-medium">任务已删除</span>
        <div class="w-px h-4 bg-slate-600"></div>
        <button 
          @click="taskStore.undoDelete()" 
          class="text-sm text-blue-400 hover:text-blue-300 font-semibold flex items-center space-x-1.5 transition-colors"
        >
          <RotateCcw class="w-4 h-4" />
          <span>撤销</span>
        </button>
      </div>
    </Transition>

    <!-- Status Tabs -->
    <div v-if="!isCompletedView" class="sticky top-0 z-10 px-6 pt-1 pb-3 mb-1 bg-gradient-to-b from-white via-white to-transparent">
      <div class="flex items-center space-x-1 bg-slate-100 rounded-xl p-1 w-fit">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          @click="activeTab = tab.id"
          :class="['flex items-center space-x-1.5 px-4 py-1.5 rounded-lg text-[13px] font-medium transition-all duration-200',
                   activeTab === tab.id
                     ? 'bg-white text-slate-800 shadow-sm'
                     : 'text-slate-500 hover:text-slate-700']"
        >
          <span>{{ tab.label }}</span>
          <span :class="['text-xs px-1.5 py-0.5 rounded-full',
                         activeTab === tab.id ? 'bg-blue-50 text-blue-600' : 'bg-slate-200/70 text-slate-400']">
            {{ tab.count }}
          </span>
        </button>
      </div>
    </div>

    <!-- List -->
    <div v-if="taskStore.loading && taskStore.tasks.length === 0" class="flex justify-center py-20 text-slate-400">
      <div class="w-8 h-8 border-2 border-slate-200 border-t-blue-500 rounded-full animate-spin"></div>
    </div>
    
    <!-- Error State -->
    <div v-else-if="taskStore.error" class="mx-8 my-12 p-4 bg-rose-50 text-rose-600 rounded-xl text-sm border border-rose-100 flex items-center">
      {{ taskStore.error }}
    </div>
    
    <!-- Empty State -->
    <div v-else-if="taskStore.tasks.length === 0" class="flex flex-col items-center justify-center py-32 text-slate-400">
      <div class="w-20 h-20 bg-slate-100 rounded-full flex items-center justify-center mb-6">
        <Inbox class="w-10 h-10 text-slate-300" />
      </div>
      <p class="text-[15px] font-medium text-slate-500">没有待办任务</p>
      <p class="text-sm text-slate-400 mt-1">享受片刻宁静，或添加新任务</p>
    </div>
    
    <!-- List -->
    <div v-else>
      <TransitionGroup name="list" tag="div">
        <TaskItem 
          v-for="task in (isCompletedView ? taskStore.tasks : filteredTasks)" 
          :key="task.id" 
          :task="task" 
        />
      </TransitionGroup>
      <p v-if="!isCompletedView && filteredTasks.length === 0" class="mx-6 mt-4 text-xs text-slate-400 border border-dashed border-slate-200 rounded-xl py-6 text-center">
        该分类下暂无任务
      </p>
    </div>
  </div>
</template>

<style scoped>
/* Undo Toast Animation */
.fade-up-enter-active,
.fade-up-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.fade-up-enter-from,
.fade-up-leave-to {
  opacity: 0;
  transform: translate(-50%, 20px) scale(0.95);
}

/* List Item Animations */
.list-move,
.list-enter-active,
.list-leave-active {
  transition: all 0.3s ease;
}
.list-enter-from {
  opacity: 0;
  transform: translateX(-20px);
}
.list-leave-to {
  opacity: 0;
  transform: translateX(20px);
}
.list-leave-active {
  position: absolute;
}
</style>

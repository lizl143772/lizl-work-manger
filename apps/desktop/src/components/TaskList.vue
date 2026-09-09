<script setup lang="ts">
import { useTaskStore } from '../stores/taskStore';
import TaskItem from './TaskItem.vue';
import { Inbox, RotateCcw } from 'lucide-vue-next';

const taskStore = useTaskStore();
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

    <!-- Loading State -->
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
          v-for="task in taskStore.tasks" 
          :key="task.id" 
          :task="task" 
        />
      </TransitionGroup>
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

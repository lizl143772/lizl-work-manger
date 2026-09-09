<script setup lang="ts">
import { onMounted } from 'vue';
import { useProjectStore } from './stores/projectStore';
import { useTaskStore } from './stores/taskStore';
import Sidebar from './components/Sidebar.vue';
import QuickInput from './components/QuickInput.vue';
import TaskList from './components/TaskList.vue';
import GlobalOverlays from './components/GlobalOverlays.vue';
import { CheckCircle2, ListTodo } from 'lucide-vue-next';

const projectStore = useProjectStore();
const taskStore = useTaskStore();

onMounted(async () => {
  await projectStore.loadProjects();
  await taskStore.loadTasks();
});
</script>

<template>
  <div class="flex h-screen w-full bg-[#f4f4f5] text-slate-800 font-sans overflow-hidden">
    <!-- Global Overlays (Toasts/Modals) -->
    <GlobalOverlays />

    <Sidebar />
    <div class="flex-1 flex flex-col min-w-0 bg-white m-2 rounded-2xl shadow-sm border border-slate-100 overflow-hidden relative">
      
      <!-- Header -->
      <header class="px-8 py-6 flex items-center justify-between bg-white/80 backdrop-blur-md sticky top-0 z-10 border-b border-slate-50">
        <div class="flex items-center space-x-3">
          <div class="w-8 h-8 rounded-full bg-blue-50 text-blue-600 flex items-center justify-center">
            <CheckCircle2 v-if="projectStore.currentViewId === 'completed'" class="w-5 h-5" />
            <ListTodo v-else class="w-5 h-5" />
          </div>
          <h1 class="text-2xl font-bold tracking-tight text-slate-800">{{ projectStore.currentViewName }}</h1>
        </div>
      </header>

      <!-- Task List Area -->
      <main class="flex-1 overflow-hidden flex flex-col bg-slate-50/30">
        <TaskList />
      </main>

      <!-- Quick Input Anchored at Bottom -->
      <div v-if="projectStore.currentViewId !== 'completed'" class="px-8 pt-2 pb-6 bg-white border-t border-slate-50 shadow-[0_-4px_6px_-1px_rgba(0,0,0,0.02)] z-10">
        <QuickInput />
      </div>
    </div>
  </div>
</template>

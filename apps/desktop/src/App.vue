<script setup lang="ts">
import { onMounted } from 'vue';
import { useProjectStore } from './stores/projectStore';
import { useTaskStore } from './stores/taskStore';
import { useUiStore } from './stores/uiStore';
import { useSettingsStore } from './stores/settingsStore';
import { api } from './lib/api';
import TitleBar from './components/TitleBar.vue';
import Sidebar from './components/Sidebar.vue';
import QuickInput from './components/QuickInput.vue';
import TaskList from './components/TaskList.vue';
import MiniMode from './components/MiniMode.vue';
import CalendarView from './components/CalendarView.vue';
import TrashView from './components/TrashView.vue';
import StatsView from './components/StatsView.vue';
import SettingsView from './components/SettingsView.vue';
import GlobalOverlays from './components/GlobalOverlays.vue';
import { CheckCircle2, ListTodo, CalendarDays, Trash2, BarChart3, Settings } from 'lucide-vue-next';

const projectStore = useProjectStore();
const taskStore = useTaskStore();
const uiStore = useUiStore();
const settingsStore = useSettingsStore();

onMounted(async () => {
  uiStore.applyPinOnStartup();
  await projectStore.loadProjects();
  await taskStore.loadTasks();

  // 按设置的保留期清理回收站里过期的任务（0 表示永久保留）
  try {
    const purged = await api.purgeExpiredDeletedTasks(settingsStore.settings.trashRetentionDays);
    if (purged > 0) {
      uiStore.showMessage(`已自动清理 ${purged} 条超出保留期的已删除任务`, 'info');
    }
  } catch {
    // 清理失败不影响正常使用
  }
});
</script>

<template>
  <div class="flex flex-col h-screen w-full bg-[#f4f4f5] text-slate-800 font-sans overflow-hidden">
    <!-- Global Overlays (Toasts/Modals) -->
    <GlobalOverlays />

    <!-- Custom Title Bar -->
    <TitleBar />

    <!-- Mini Mode -->
    <MiniMode v-if="uiStore.isMiniMode" />

    <div v-else class="flex-1 flex min-h-0">
      <Sidebar />
      <div class="flex-1 flex flex-col min-w-0 bg-white m-2 rounded-2xl shadow-sm border border-slate-100 overflow-hidden relative">
        
        <!-- Header -->
        <header class="px-8 py-6 flex items-center justify-between bg-white/80 backdrop-blur-md sticky top-0 z-10 border-b border-slate-50">
          <div class="flex items-center space-x-3">
            <div class="w-8 h-8 rounded-full bg-blue-50 text-blue-600 flex items-center justify-center">
              <CheckCircle2 v-if="projectStore.currentViewId === 'completed'" class="w-5 h-5" />
              <CalendarDays v-else-if="projectStore.isCalendarView" class="w-5 h-5" />
              <Trash2 v-else-if="projectStore.isTrashView" class="w-5 h-5" />
              <BarChart3 v-else-if="projectStore.isStatsView" class="w-5 h-5" />
              <Settings v-else-if="projectStore.isSettingsView" class="w-5 h-5" />
              <ListTodo v-else class="w-5 h-5" />
            </div>
            <h1 class="text-2xl font-bold tracking-tight text-slate-800">{{ projectStore.currentViewName }}</h1>
          </div>
        </header>

        <!-- Calendar View -->
        <main v-if="projectStore.isCalendarView" class="flex-1 overflow-hidden flex flex-col bg-slate-50/30">
          <CalendarView />
        </main>

        <!-- Trash View -->
        <main v-else-if="projectStore.isTrashView" class="flex-1 overflow-hidden flex flex-col bg-slate-50/30">
          <TrashView />
        </main>

        <!-- Stats View -->
        <main v-else-if="projectStore.isStatsView" class="flex-1 overflow-hidden flex flex-col bg-slate-50/30">
          <StatsView />
        </main>

        <!-- Settings View -->
        <main v-else-if="projectStore.isSettingsView" class="flex-1 overflow-hidden flex flex-col bg-slate-50/30">
          <SettingsView />
        </main>

        <!-- Task List Area -->
        <main v-else class="flex-1 overflow-hidden flex flex-col bg-slate-50/30">
          <TaskList />
        </main>

        <!-- Quick Input Anchored at Bottom（仅任务列表视图且非「已完成」时出现） -->
        <div v-if="projectStore.isTaskListView && projectStore.currentViewId !== 'completed'" class="px-8 pt-2 pb-6 bg-white border-t border-slate-50 shadow-[0_-4px_6px_-1px_rgba(0,0,0,0.02)] z-10">
          <QuickInput />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useUiStore } from '../stores/uiStore';
import { CheckCircle, Pin, PinOff, Minimize2, Maximize2, Minus, Square, Copy, X } from 'lucide-vue-next';

const uiStore = useUiStore();
const win = getCurrentWindow();
const isMaximized = ref(false);

onMounted(async () => {
  try {
    isMaximized.value = await win.isMaximized();
  } catch {}
});

const startDrag = async () => {
  try {
    await win.startDragging();
  } catch {}
};

const minimize = async () => {
  try {
    await win.minimize();
  } catch {}
};

const toggleMaximize = async () => {
  try {
    await win.toggleMaximize();
    isMaximized.value = await win.isMaximized();
  } catch {}
};

const closeWindow = async () => {
  try {
    await win.close();
  } catch {}
};
</script>

<template>
  <div
    class="h-10 flex items-center justify-between bg-white border-b border-slate-100 select-none flex-shrink-0"
    @mousedown="startDrag"
    @dblclick="toggleMaximize"
  >
    <!-- Brand -->
    <div class="flex items-center px-3 space-x-2">
      <div class="w-5 h-5 bg-blue-600 rounded-md flex items-center justify-center">
        <CheckCircle class="w-3.5 h-3.5 text-white" />
      </div>
      <span class="text-xs font-bold text-slate-700 tracking-tight">lizl 待办管理</span>
    </div>

    <!-- Window Actions -->
    <div class="flex items-center h-full">
      <button
        @mousedown.stop
        @click="uiStore.togglePin()"
        :class="['h-full px-3 flex items-center transition-colors', uiStore.isPinned ? 'text-blue-600 bg-blue-50' : 'text-slate-400 hover:text-slate-600 hover:bg-slate-100']"
        :title="uiStore.isPinned ? '取消置顶' : '钉在桌面（置顶）'"
      >
        <component :is="uiStore.isPinned ? PinOff : Pin" class="w-4 h-4" />
      </button>
      <button
        @mousedown.stop
        @click="uiStore.toggleMiniMode()"
        class="h-full px-3 flex items-center text-slate-400 hover:text-slate-600 hover:bg-slate-100 transition-colors"
        :title="uiStore.isMiniMode ? '返回完整窗口' : '切换到小窗口'"
      >
        <component :is="uiStore.isMiniMode ? Maximize2 : Minimize2" class="w-4 h-4" />
      </button>

      <div class="w-px h-4 bg-slate-200 mx-1"></div>

      <button
        @mousedown.stop
        @click="minimize"
        class="h-full px-3.5 flex items-center text-slate-400 hover:text-slate-600 hover:bg-slate-100 transition-colors"
        title="最小化"
      >
        <Minus class="w-4 h-4" />
      </button>
      <button
        @mousedown.stop
        @click="toggleMaximize"
        class="h-full px-3.5 flex items-center text-slate-400 hover:text-slate-600 hover:bg-slate-100 transition-colors"
        :title="isMaximized ? '还原' : '最大化'"
      >
        <Copy v-if="isMaximized" class="w-3.5 h-3.5" />
        <Square v-else class="w-3.5 h-3.5" />
      </button>
      <button
        @mousedown.stop
        @click="closeWindow"
        class="h-full px-3.5 flex items-center text-slate-400 hover:text-white hover:bg-rose-500 transition-colors"
        title="关闭"
      >
        <X class="w-4 h-4" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { CalendarRange, Check, ChevronDown } from 'lucide-vue-next';
import { useTaskStore } from '../stores/taskStore';
import { DATE_RANGE_OPTIONS, type DateRangeKey } from '../types/task';

const taskStore = useTaskStore();

const open = ref(false);
const rootEl = ref<HTMLElement | null>(null);

const isFiltered = computed(() => taskStore.dateRange !== 'all');

function toggle() {
  open.value = !open.value;
}

async function select(key: DateRangeKey) {
  open.value = false;
  await taskStore.setDateRange(key);
}

function onDocumentMouseDown(e: MouseEvent) {
  if (rootEl.value && !rootEl.value.contains(e.target as Node)) {
    open.value = false;
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') open.value = false;
}

onMounted(() => {
  document.addEventListener('mousedown', onDocumentMouseDown);
  document.addEventListener('keydown', onKeydown);
});

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', onDocumentMouseDown);
  document.removeEventListener('keydown', onKeydown);
});
</script>

<template>
  <div ref="rootEl" class="relative" @mousedown.stop @dblclick.stop>
    <!-- Trigger -->
    <button
      @click="toggle"
      :class="['flex items-center space-x-1.5 pl-2.5 pr-2 py-1.5 rounded-lg text-xs font-medium transition-colors',
               isFiltered
                 ? 'bg-blue-50 text-blue-700 hover:bg-blue-100'
                 : 'text-slate-500 hover:bg-slate-100 hover:text-slate-700']"
      :title="`当前时间范围：${taskStore.currentDateRangeLabel}（全局生效）`"
    >
      <CalendarRange class="w-3.5 h-3.5" />
      <span>{{ taskStore.currentDateRangeLabel }}</span>
      <ChevronDown class="w-3 h-3 transition-transform duration-200" :class="open ? 'rotate-180' : ''" />
    </button>

    <!-- Dropdown -->
    <Transition name="dropdown">
      <div
        v-if="open"
        class="absolute left-0 top-full mt-1.5 w-56 bg-white rounded-2xl shadow-xl shadow-slate-900/10 border border-slate-100 p-2 z-50"
      >
        <div class="px-3 pt-1.5 pb-2.5 border-b border-slate-50 mb-1">
          <p class="text-[11px] font-semibold text-slate-400 tracking-wider">时间范围</p>
          <p class="text-[11px] text-slate-400 mt-0.5">按任务创建时间筛选，全局生效</p>
        </div>

        <button
          v-for="opt in DATE_RANGE_OPTIONS"
          :key="opt.key"
          @click="select(opt.key)"
          :class="['w-full flex items-center justify-between px-3 py-2 rounded-xl text-[13px] transition-colors',
                   opt.key === taskStore.dateRange
                     ? 'bg-blue-50 text-blue-700 font-semibold'
                     : 'text-slate-600 hover:bg-slate-50']"
        >
          <span>{{ opt.label }}</span>
          <Check v-if="opt.key === taskStore.dateRange" class="w-4 h-4 text-blue-600" />
        </button>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.18s cubic-bezier(0.4, 0, 0.2, 1);
}
.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-6px) scale(0.97);
}
</style>

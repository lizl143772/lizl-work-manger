<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue';
import { useTaskStore } from '../stores/taskStore';
import { useProjectStore } from '../stores/projectStore';
import TaskItem from './TaskItem.vue';
import { Inbox, RotateCcw, Search, ArrowDown, ArrowUp, X } from 'lucide-vue-next';
import { TASK_SORT_OPTIONS } from '../types/task';

const taskStore = useTaskStore();
const projectStore = useProjectStore();

const isCompletedView = computed(() => projectStore.currentViewId === 'completed');
const isFilteredByTime = computed(() => taskStore.dateRange !== 'all');

// ---------- 「已完成」视图的搜索与排序 ----------

/** 输入框本地值，配合防抖避免每敲一个字就查一次库 */
const keywordInput = ref(taskStore.completedKeyword);
let searchTimer: number | null = null;

function onKeywordInput(e: Event) {
  keywordInput.value = (e.target as HTMLInputElement).value;
  if (searchTimer) clearTimeout(searchTimer);
  searchTimer = window.setTimeout(() => {
    taskStore.setCompletedKeyword(keywordInput.value);
  }, 250);
}

function clearKeyword() {
  if (searchTimer) clearTimeout(searchTimer);
  keywordInput.value = '';
  taskStore.setCompletedKeyword('');
}

// 关键字被外部改动（例如重置）时同步回输入框
watch(
  () => taskStore.completedKeyword,
  (v) => {
    if (v !== keywordInput.value) keywordInput.value = v;
  }
);

onBeforeUnmount(() => {
  if (searchTimer) clearTimeout(searchTimer);
});

const sortDescLabel = computed(() => (taskStore.completedSortDesc ? '降序' : '升序'));

const hasKeyword = computed(() => !!taskStore.completedKeyword.trim());

const emptyTitle = computed(() => {
  if (hasKeyword.value) return `没有匹配「${taskStore.completedKeyword.trim()}」的任务`;
  if (isCompletedView.value) {
    return isFilteredByTime.value ? `${taskStore.currentDateRangeLabel}没有已完成任务` : '还没有已完成的任务';
  }
  return isFilteredByTime.value ? `${taskStore.currentDateRangeLabel}没有任务` : '没有待办任务';
});

const emptyHint = computed(() =>
  hasKeyword.value
    ? '换个关键字试试，或清空搜索'
    : isFilteredByTime.value
      ? '可切换时间范围查看更早的任务'
      : '享受片刻宁静，或添加新任务'
);

type StatusTab = 'all' | 'todo' | 'in_progress' | 'completed';
const activeTab = ref<StatusTab>('all');

/** 是否是具体项目视图（既不是 inbox 也不是全局已完成） */
const isProjectView = computed(() => !isCompletedView.value && projectStore.currentViewId !== 'inbox');

// 切换视图时，若当前 tab 在新视图中不存在则回退到 '全部'
watch(() => projectStore.currentViewId, () => {
  if (activeTab.value === 'completed' && !isProjectView.value) {
    activeTab.value = 'all';
  }
});

const todoCount = computed(() => taskStore.tasks.filter(t => t.status === 'todo').length);
const inProgressCount = computed(() => taskStore.tasks.filter(t => t.status === 'in_progress').length);
const completedCount = computed(() => taskStore.tasks.filter(t => t.status === 'completed').length);

const tabs = computed(() => {
  const base = [
    { id: 'all' as StatusTab, label: '全部', count: taskStore.tasks.length },
    { id: 'todo' as StatusTab, label: '未开始', count: todoCount.value },
    { id: 'in_progress' as StatusTab, label: '进行中', count: inProgressCount.value },
  ];
  // 项目视图含已完成任务，增加已完成 tab
  if (isProjectView.value) {
    base.push({ id: 'completed' as StatusTab, label: '已完成', count: completedCount.value });
  }
  return base;
});

const filteredTasks = computed(() => {
  if (activeTab.value === 'todo') return taskStore.tasks.filter(t => t.status === 'todo');
  if (activeTab.value === 'in_progress') return taskStore.tasks.filter(t => t.status === 'in_progress');
  if (activeTab.value === 'completed') return taskStore.tasks.filter(t => t.status === 'completed');
  return taskStore.tasks;
});

/**
 * 点击「当前展开的那张卡片」以外的任何区域，都收起当前展开的卡片。
 * 采用捕获阶段监听：内部元素即使写了 stopPropagation 也拦不住，
 * 保证列表空白、其他卡片、侧边栏、标题栏等任意位置点击都能收起。
 */
function onGlobalPointerDown(e: MouseEvent) {
  const openId = taskStore.expandedTaskId;
  if (!openId) return;
  const target = e.target as Element | null;
  if (!target || typeof target.closest !== 'function') return;
  // 浮层（时间规划弹窗、右键菜单、图片预览、全局弹窗）内点击不收起
  if (target.closest('[data-no-collapse]')) return;
  // 点击任意任务卡片：交给卡片自身的展开/切换逻辑，
  // 直接 A → B 一次切换，避免先收起（null）再展开造成的高度二次跳变
  if (target.closest('[data-task-item]')) return;
  taskStore.collapseExpanded();
}

onMounted(() => document.addEventListener('mousedown', onGlobalPointerDown, true));
onBeforeUnmount(() => document.removeEventListener('mousedown', onGlobalPointerDown, true));
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

    <!-- 已完成视图：关键词搜索 + 按时间排序 -->
    <div v-else class="sticky top-0 z-10 px-6 pt-1 pb-3 mb-1 bg-gradient-to-b from-white via-white to-transparent">
      <div class="flex items-center gap-3 flex-wrap">
        <!-- 搜索 -->
        <div class="relative flex items-center">
          <Search class="w-3.5 h-3.5 text-slate-400 absolute left-2.5 pointer-events-none" />
          <input
            :value="keywordInput"
            @input="onKeywordInput"
            type="text"
            placeholder="搜索已完成的任务…"
            class="w-56 bg-slate-100 rounded-lg pl-8 pr-7 py-1.5 text-[13px] text-slate-700 placeholder:text-slate-400 outline-none focus:ring-2 focus:ring-blue-200"
          />
          <button
            v-if="keywordInput"
            @click="clearKeyword"
            class="absolute right-2 text-slate-400 hover:text-slate-600 transition-colors"
            title="清空"
          >
            <X class="w-3.5 h-3.5" />
          </button>
        </div>

        <span class="text-[12px] text-slate-400 tabular-nums">{{ taskStore.tasks.length }} 项</span>

        <div class="flex-1"></div>

        <!-- 排序 -->
        <div class="flex items-center gap-2">
          <span class="text-[12px] text-slate-400">按</span>
          <div class="flex items-center space-x-1 bg-slate-100 rounded-xl p-1 w-fit">
            <button
              v-for="opt in TASK_SORT_OPTIONS"
              :key="opt.key"
              @click="taskStore.setCompletedSortBy(opt.key)"
              :class="['px-3 py-1 rounded-lg text-[13px] font-medium transition-all duration-200',
                       taskStore.completedSortBy === opt.key
                         ? 'bg-white text-slate-800 shadow-sm'
                         : 'text-slate-500 hover:text-slate-700']"
            >
              {{ opt.label }}
            </button>
          </div>
          <button
            @click="taskStore.toggleCompletedSortDesc()"
            :title="taskStore.completedSortDesc ? '当前降序，点击切换为升序' : '当前升序，点击切换为降序'"
            class="flex items-center space-x-1 px-2.5 py-1.5 rounded-lg text-[12px] text-slate-500 bg-slate-100 hover:bg-slate-200/70 transition-colors"
          >
            <ArrowDown v-if="taskStore.completedSortDesc" class="w-3.5 h-3.5" />
            <ArrowUp v-else class="w-3.5 h-3.5" />
            <span>{{ sortDescLabel }}</span>
          </button>
        </div>
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
      <p class="text-[15px] font-medium text-slate-500">{{ emptyTitle }}</p>
      <p class="text-sm text-slate-400 mt-1">{{ emptyHint }}</p>
      <button
        v-if="hasKeyword"
        @click="clearKeyword"
        class="mt-5 px-4 py-1.5 text-[13px] font-medium text-blue-600 bg-blue-50 hover:bg-blue-100 rounded-full transition-colors"
      >
        清空搜索
      </button>
      <button
        v-else-if="isFilteredByTime"
        @click="taskStore.setDateRange('all')"
        class="mt-5 px-4 py-1.5 text-[13px] font-medium text-blue-600 bg-blue-50 hover:bg-blue-100 rounded-full transition-colors"
      >
        查看全部时间
      </button>
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
/* 只过渡位移并缩短时长：展开/收起会让卡片高度突变，
   过长的 all 过渡会让下方卡片"追不上"展开动作，产生粘滞感 */
.list-move {
  transition: transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}
.list-enter-active,
.list-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
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

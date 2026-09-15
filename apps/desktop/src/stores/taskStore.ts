import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { api } from '../lib/api';
import type { Task, TaskStatus, DateRangeKey, TaskSortKey } from '../types/task';
import { DATE_RANGE_OPTIONS, DEFAULT_TASK_SORT, dateRangeToFrom } from '../types/task';
import { useProjectStore } from './projectStore';
import { useSettingsStore } from './settingsStore';

export const useTaskStore = defineStore('task', () => {
  const tasks = ref<Task[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const undoTimeout = ref<number | null>(null);
  const lastDeletedTask = ref<Task | null>(null);
  const expandedTaskId = ref<string | null>(null);

  const projectStore = useProjectStore();
  const settingsStore = useSettingsStore();

  /** 时间范围筛选改为由 settingsStore 统一持久化，这里只做只读投影 */
  const dateRange = computed(() => settingsStore.settings.dateRange);

  // 「已完成」视图的搜索与排序（仅会话内保留，不落盘）
  const completedKeyword = ref('');
  const completedSortBy = ref<TaskSortKey>(DEFAULT_TASK_SORT);
  const completedSortDesc = ref(true);

  /** 当前时间范围的展示文案 */
  const currentDateRangeLabel = computed(
    () => DATE_RANGE_OPTIONS.find(o => o.key === dateRange.value)?.label ?? '近半年'
  );

  function toggleExpanded(id: string) {
    if (expandedTaskId.value === id) {
      expandedTaskId.value = null;
    } else {
      expandedTaskId.value = id;
    }
  }

  /** 收起当前展开（选中）的任务卡片 */
  function collapseExpanded() {
    if (expandedTaskId.value !== null) {
      expandedTaskId.value = null;
    }
  }

  async function loadTasks() {
    // 日历 / 回收站 / 统计 / 设置这些视图不渲染任务列表，数据由各自组件取
    if (!projectStore.isTaskListView) {
      tasks.value = [];
      error.value = null;
      loading.value = false;
      return;
    }
    loading.value = true;
    error.value = null;
    try {
      const isCompletedView = projectStore.currentViewId === 'completed';
      const isInboxView = projectStore.currentViewId === 'inbox';
      // 项目视图展示全部状态（含已完成），待办列表只展示未完成
      const isProjectView = !isInboxView && !isCompletedView;
      const statuses: TaskStatus[] = isCompletedView
        ? ['completed']
        : isProjectView
          ? ['todo', 'in_progress', 'completed']
          : ['todo', 'in_progress'];
      const query = {
        project_id: isInboxView ? undefined : projectStore.currentViewActualId,
        statuses,
        created_from: dateRangeToFrom(dateRange.value),
        // 「已完成」视图支持关键字搜索与按时间排序，其余视图走默认排序
        keyword: isCompletedView ? completedKeyword.value.trim() || undefined : undefined,
        sort_by: isCompletedView ? completedSortBy.value : undefined,
        sort_desc: isCompletedView ? completedSortDesc.value : undefined,
        page: 1,
        page_size: 100
      };
      const res = await api.listTasks(query);
      tasks.value = res.items;
    } catch (e: any) {
      error.value = e.message || '加载任务失败';
    } finally {
      loading.value = false;
    }
  }

  /** 设置「已完成」视图的搜索关键字 */
  async function setCompletedKeyword(kw: string) {
    if (completedKeyword.value === kw) return;
    completedKeyword.value = kw;
    await loadTasks();
  }

  /** 设置「已完成」视图的排序字段 */
  async function setCompletedSortBy(key: TaskSortKey) {
    if (completedSortBy.value === key) return;
    completedSortBy.value = key;
    await loadTasks();
  }

  /** 切换「已完成」视图的排序方向 */
  async function toggleCompletedSortDesc() {
    completedSortDesc.value = !completedSortDesc.value;
    await loadTasks();
  }

  /** 设置时间范围并重新加载任务 */
  async function setDateRange(key: DateRangeKey) {
    if (settingsStore.settings.dateRange === key) return;
    settingsStore.patch({ dateRange: key });
    await loadTasks();
  }

  async function quickAdd(title: string, attachments?: string | null) {
    const isCompletedView = projectStore.currentViewId === 'completed';
    const projectId = isCompletedView ? projectStore.defaultProject?.id : projectStore.currentViewActualId;
    
    try {
      const newTask = await api.createTask({ title, project_id: projectId });
      if (attachments) {
        await api.updateTaskDetails(newTask.id, { attachments });
        newTask.attachments = attachments;
      }
      if (!isCompletedView) {
        tasks.value.unshift(newTask);
      }
      await projectStore.loadCounts();
      return newTask;
    } catch (e: any) {
      error.value = e.message || '创建任务失败';
      throw e;
    }
  }

  async function updateStatus(id: string, status: TaskStatus) {
    const idx = tasks.value.findIndex(t => t.id === id);
    if (idx === -1) return;
    
    const task = tasks.value[idx];
    const oldStatus = task.status;
    task.status = status; // Optimistic
    
    try {
      await api.updateTaskStatus(id, status);
      await loadTasks(); // Reload to apply sort/filter
      await projectStore.loadCounts();
    } catch (e) {
      task.status = oldStatus; // Revert
    }
  }

  async function updateDetails(id: string, input: import('../types/task').TaskUpdateInput) {
    const updated = await api.updateTaskDetails(id, input);
    const index = tasks.value.findIndex(t => t.id === id);
    if (index !== -1) tasks.value[index] = updated;
  }

  async function updatePriority(id: string, priority: number) {
    const idx = tasks.value.findIndex(t => t.id === id);
    if (idx === -1) return;
    try {
      await api.updateTaskPriority(id, priority);
      await loadTasks(); // Reload for sort
    } catch (e) {}
  }

  async function moveTask(id: string, projectId: string) {
    try {
      await api.moveTask(id, projectId);
      await loadTasks();
      await projectStore.loadCounts();
    } catch (e) {}
  }

  async function removeTask(id: string) {
    const idx = tasks.value.findIndex(t => t.id === id);
    if (idx === -1) return;
    
    const task = tasks.value[idx];
    tasks.value.splice(idx, 1); // Optimistic remove
    lastDeletedTask.value = task;
    
    try {
      await api.deleteTask(id);
      await projectStore.loadCounts();
      
      if (undoTimeout.value) clearTimeout(undoTimeout.value);
      undoTimeout.value = window.setTimeout(() => {
        lastDeletedTask.value = null;
      }, 5000);
      
    } catch (e) {
      tasks.value.splice(idx, 0, task); // Revert
    }
  }

  async function undoDelete() {
    if (!lastDeletedTask.value) return;
    try {
      await api.restoreTask(lastDeletedTask.value.id);
      lastDeletedTask.value = null;
      if (undoTimeout.value) clearTimeout(undoTimeout.value);
      await loadTasks();
      await projectStore.loadCounts();
    } catch (e) {}
  }

  return {
    tasks,
    loading,
    error,
    lastDeletedTask,
    dateRange,
    currentDateRangeLabel,
    setDateRange,
    completedKeyword,
    completedSortBy,
    completedSortDesc,
    setCompletedKeyword,
    setCompletedSortBy,
    toggleCompletedSortDesc,
    loadTasks,
    quickAdd,
    updateStatus,
    updateDetails,
    updatePriority,
    moveTask,
    removeTask,
    undoDelete,
    expandedTaskId,
    toggleExpanded,
    collapseExpanded
  };
});

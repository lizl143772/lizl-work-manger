import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { api } from '../lib/api';
import type { Task, TaskStatus, DateRangeKey } from '../types/task';
import { DATE_RANGE_OPTIONS, DEFAULT_DATE_RANGE, dateRangeToFrom } from '../types/task';
import { useProjectStore } from './projectStore';

const DATE_RANGE_STORAGE_KEY = 'workmanager.dateRange';

function readStoredDateRange(): DateRangeKey {
  const raw = localStorage.getItem(DATE_RANGE_STORAGE_KEY) as DateRangeKey | null;
  return raw && DATE_RANGE_OPTIONS.some(o => o.key === raw) ? raw : DEFAULT_DATE_RANGE;
}

export const useTaskStore = defineStore('task', () => {
  const tasks = ref<Task[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const undoTimeout = ref<number | null>(null);
  const lastDeletedTask = ref<Task | null>(null);
  const expandedTaskId = ref<string | null>(null);
  // 时间范围筛选，默认近半年
  const dateRange = ref<DateRangeKey>(readStoredDateRange());

  const projectStore = useProjectStore();

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
    loading.value = true;
    error.value = null;
    try {
      const isCompletedView = projectStore.currentViewId === 'completed';
      const isInboxView = projectStore.currentViewId === 'inbox';
      const query = {
        project_id: isInboxView ? undefined : projectStore.currentViewActualId,
        statuses: isCompletedView ? ['completed' as TaskStatus] : ['todo' as TaskStatus, 'in_progress' as TaskStatus],
        created_from: dateRangeToFrom(dateRange.value),
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

  /** 设置时间范围并重新加载任务 */
  async function setDateRange(key: DateRangeKey) {
    if (dateRange.value === key) return;
    dateRange.value = key;
    localStorage.setItem(DATE_RANGE_STORAGE_KEY, key);
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

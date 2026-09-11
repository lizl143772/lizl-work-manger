import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { api } from '../lib/api';
import type { Project, TaskCountsSummary } from '../types/task';

/** 非任务列表视图：这些视图有自己的主区组件，不加载任务列表 */
const NON_TASK_VIEWS = ['calendar', 'trash', 'stats', 'settings'];

/** 系统视图的展示名 */
const SYSTEM_VIEW_NAMES: Record<string, string> = {
  inbox: '待办列表',
  completed: '已完成',
  calendar: '日历',
  trash: '回收站',
  stats: '统计',
  settings: '设置',
};

export const useProjectStore = defineStore('project', () => {
  const projects = ref<Project[]>([]);
  const counts = ref<TaskCountsSummary>({ inbox: 0, projects: {}, completed: 0 });
  // 'inbox' | 'completed' | 'calendar' | 'trash' | 'stats' | 'settings' | project_id
  const currentViewId = ref<string>('inbox');

  const activeProjects = computed(() => projects.value.filter(p => !p.is_archived));
  const defaultProject = computed(() => projects.value.find(p => p.is_default));

  const isCalendarView = computed(() => currentViewId.value === 'calendar');
  const isTrashView = computed(() => currentViewId.value === 'trash');
  const isStatsView = computed(() => currentViewId.value === 'stats');
  const isSettingsView = computed(() => currentViewId.value === 'settings');

  /** 主区是否渲染任务列表（待办列表 / 已完成 / 某个项目） */
  const isTaskListView = computed(() => !NON_TASK_VIEWS.includes(currentViewId.value));

  const currentViewName = computed(() => {
    const sys = SYSTEM_VIEW_NAMES[currentViewId.value];
    if (sys) return sys;
    const proj = projects.value.find(p => p.id === currentViewId.value);
    return proj ? proj.name : '未知';
  });

  const currentViewActualId = computed(() => {
    if (currentViewId.value === 'inbox') return defaultProject.value?.id;
    if (NON_TASK_VIEWS.includes(currentViewId.value)) return undefined;
    if (currentViewId.value === 'completed') return undefined;
    return currentViewId.value;
  });

  async function loadProjects() {
    projects.value = await api.listProjects();
    await loadCounts();
  }

  async function loadCounts() {
    counts.value = await api.getTaskCounts();
  }

  async function addProject(name: string, color?: string, icon?: string) {
    const res = await api.createProject({ name, color, icon });
    projects.value.push(res);
    return res;
  }
  
  async function editProject(id: string, name: string, color?: string, icon?: string) {
    const res = await api.updateProject(id, { name, color, icon });
    const idx = projects.value.findIndex(p => p.id === id);
    if (idx !== -1) projects.value[idx] = res;
  }

  async function delProject(id: string) {
    await api.deleteProject(id);
    projects.value = projects.value.filter(p => p.id !== id);
    if (currentViewId.value === id) {
      currentViewId.value = 'inbox';
    }
    await loadCounts();
  }

  return {
    projects,
    counts,
    currentViewId,
    activeProjects,
    defaultProject,
    isCalendarView,
    isTrashView,
    isStatsView,
    isSettingsView,
    isTaskListView,
    currentViewName,
    currentViewActualId,
    loadProjects,
    loadCounts,
    addProject,
    editProject,
    delProject
  };
});

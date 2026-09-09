import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { api } from '../lib/api';
import type { Project, TaskCountsSummary } from '../types/task';

export const useProjectStore = defineStore('project', () => {
  const projects = ref<Project[]>([]);
  const counts = ref<TaskCountsSummary>({ inbox: 0, projects: {}, completed: 0 });
  const currentViewId = ref<string>('inbox'); // 'inbox', 'completed', or project_id

  const activeProjects = computed(() => projects.value.filter(p => !p.is_archived));
  const defaultProject = computed(() => projects.value.find(p => p.is_default));
  
  const currentViewName = computed(() => {
    if (currentViewId.value === 'inbox') return '待办列表';
    if (currentViewId.value === 'completed') return '已完成';
    const proj = projects.value.find(p => p.id === currentViewId.value);
    return proj ? proj.name : '未知';
  });

  const currentViewActualId = computed(() => {
    if (currentViewId.value === 'inbox') return defaultProject.value?.id;
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

  async function addProject(name: string) {
    const res = await api.createProject({ name });
    projects.value.push(res);
    return res;
  }
  
  async function editProject(id: string, name: string) {
    const res = await api.updateProject(id, { name });
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
    currentViewName,
    currentViewActualId,
    loadProjects,
    loadCounts,
    addProject,
    editProject,
    delProject
  };
});

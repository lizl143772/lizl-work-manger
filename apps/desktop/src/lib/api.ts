import { invoke } from '@tauri-apps/api/core';
import type { Task, Project, TaskPage, TaskCountsSummary, CreateTaskInput, TaskQuery, CreateProjectInput, UpdateProjectInput, TaskStatus, TaskUpdateInput } from '../types/task';

export const api = {
  // Tasks
  listTasks: (query: TaskQuery) => invoke<TaskPage>('list_tasks', { query }),
  createTask: (input: CreateTaskInput) => invoke<Task>('create_task', { input }),
  updateTaskStatus: (taskId: string, status: TaskStatus) => invoke<Task>('update_task_status', { taskId, status }),
  updateTaskPriority: (taskId: string, priority: number) => invoke<Task>('update_task_priority', { taskId, priority }),
  updateTaskDetails: (id: string, input: TaskUpdateInput) => 
    invoke<Task>('update_task_details', { id, input }),
  moveTask: (taskId: string, projectId: string) => invoke<Task>('move_task', { taskId, projectId }),
  deleteTask: (taskId: string) => invoke<Task>('delete_task', { taskId }),
  restoreTask: (taskId: string) => invoke<Task>('restore_task', { taskId }),

  // Projects
  listProjects: (includeArchived: boolean = false) => invoke<Project[]>('list_projects', { includeArchived }),
  createProject: (input: CreateProjectInput) => invoke<Project>('create_project', { input }),
  updateProject: (projectId: string, input: UpdateProjectInput) => invoke<Project>('update_project', { projectId, input }),
  archiveProject: (projectId: string) => invoke<Project>('archive_project', { projectId }),
  deleteProject: (projectId: string) => invoke<void>('delete_project', { projectId }),
  getTaskCounts: () => invoke<TaskCountsSummary>('get_task_counts'),
};

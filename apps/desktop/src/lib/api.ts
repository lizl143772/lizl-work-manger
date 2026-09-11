import { invoke } from '@tauri-apps/api/core';
import type { Task, Project, TaskPage, TaskCountsSummary, CreateTaskInput, TaskQuery, CreateProjectInput, UpdateProjectInput, TaskStatus, TaskUpdateInput, DailyActivity, TaskStats, AppInfo } from '../types/task';

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

  // Calendar —— 取 from~to（本地日期 YYYY-MM-DD，闭区间）内每天的活跃度汇总
  getDailyActivity: (from: string, to: string) => invoke<DailyActivity[]>('get_daily_activity', { from, to }),

  // Trash —— 回收站（仅含已软删除的任务）
  listDeletedTasks: (keyword: string | null = null, page = 1, pageSize = 200) =>
    invoke<TaskPage>('list_deleted_tasks', { keyword, page, pageSize }),
  purgeTask: (taskId: string) => invoke<void>('purge_task', { taskId }),
  purgeDeletedTasks: () => invoke<number>('purge_deleted_tasks'),
  purgeExpiredDeletedTasks: (retentionDays: number) =>
    invoke<number>('purge_expired_deleted_tasks', { retentionDays }),

  // Stats
  getTaskStats: () => invoke<TaskStats>('get_task_stats'),

  // App
  getAppInfo: () => invoke<AppInfo>('get_app_info'),

  // Projects
  listProjects: (includeArchived: boolean = false) => invoke<Project[]>('list_projects', { includeArchived }),
  createProject: (input: CreateProjectInput) => invoke<Project>('create_project', { input }),
  updateProject: (projectId: string, input: UpdateProjectInput) => invoke<Project>('update_project', { projectId, input }),
  archiveProject: (projectId: string) => invoke<Project>('archive_project', { projectId }),
  deleteProject: (projectId: string) => invoke<void>('delete_project', { projectId }),
  getTaskCounts: () => invoke<TaskCountsSummary>('get_task_counts'),
};

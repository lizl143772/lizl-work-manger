export type TaskStatus = 'todo' | 'in_progress' | 'completed';

export interface Task {
  id: string;
  project_id: string;
  title: string;
  description: string | null;
  status: TaskStatus;
  priority: number; // 0, 1, 2, 3
  due_date: string | null;
  completed_at: string | null;
  deleted_at: string | null;
  attachments: string | null;
  time_spent: number | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface TaskUpdateInput {
  title?: string | null;
  description?: string | null;
  attachments?: string | null;
  due_date?: string | null;
  completed_at?: string | null;
  time_spent?: number | null;
}

export interface Project {
  id: string;
  name: string;
  color: string;
  icon: string;
  is_default: boolean;
  is_archived: boolean;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface TaskPage {
  items: Task[];
  total: number;
  page: number;
  page_size: number;
}

export interface TaskCountsSummary {
  inbox: number;
  projects: Record<string, number>;
  completed: number;
}

export interface CreateTaskInput {
  title: string;
  project_id?: string;
  description?: string;
  priority?: number;
}

export interface TaskQuery {
  project_id?: string;
  statuses?: TaskStatus[];
  page?: number;
  page_size?: number;
}

export interface CreateProjectInput {
  name: string;
  color?: string;
  icon?: string;
}

export interface UpdateProjectInput {
  name?: string;
  color?: string;
  icon?: string;
  sort_order?: number;
}

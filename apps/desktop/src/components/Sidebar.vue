<script setup lang="ts">
import { useProjectStore } from '../stores/projectStore';
import { useTaskStore } from '../stores/taskStore';
import { useUiStore } from '../stores/uiStore';
import { Inbox, Briefcase, User, CheckCircle, Plus, Trash2, Hash } from 'lucide-vue-next';

const projectStore = useProjectStore();
const taskStore = useTaskStore();
const uiStore = useUiStore();

const getIcon = (iconName: string) => {
  switch (iconName) {
    case 'inbox': return Inbox;
    case 'briefcase': return Briefcase;
    case 'user': return User;
    default: return Hash;
  }
};

const selectProject = async (id: string) => {
  projectStore.currentViewId = id;
  await taskStore.loadTasks();
};

const addProject = async () => {
  const name = await uiStore.prompt('新建项目', '输入新项目名称...');
  if (name && name.trim()) {
    try {
      await projectStore.addProject(name.trim());
      uiStore.showMessage('项目创建成功', 'success');
    } catch (e: any) {
      uiStore.showMessage(e.message || '创建项目失败', 'error');
    }
  }
};

const confirmDelete = async (id: string) => {
  const isConfirmed = await uiStore.confirm('确认删除该项目吗？', '删除项目后，该项目下的所有任务将被移动到「待办列表」。');
  if (isConfirmed) {
    try {
      await projectStore.delProject(id);
      uiStore.showMessage('项目已删除', 'info');
    } catch (e: any) {
      uiStore.showMessage(e.message || '删除失败', 'error');
    }
  }
};
</script>

<template>
  <div class="w-64 h-full flex flex-col bg-transparent">
    
    <!-- App Brand -->
    <div class="p-6 pt-8 pb-4 flex items-center space-x-3">
      <div class="w-8 h-8 bg-blue-600 rounded-lg shadow-sm flex items-center justify-center">
        <CheckCircle class="w-5 h-5 text-white" />
      </div>
      <span class="font-bold text-slate-800 text-lg tracking-tight">WorkManager</span>
    </div>
    
    <div class="flex-1 overflow-y-auto px-4 py-4 space-y-8 custom-scrollbar">
      
      <!-- System Views -->
      <div class="space-y-1.5">
        <button 
          @click="selectProject('inbox')"
          :class="['w-full flex items-center px-3 py-2.5 rounded-xl text-[14px] transition-all duration-200 group', 
                   projectStore.currentViewId === 'inbox' 
                     ? 'bg-white text-blue-600 shadow-sm font-semibold' 
                     : 'text-slate-600 hover:bg-slate-200/50 hover:text-slate-900']"
        >
          <Inbox class="w-4 h-4 mr-3" :class="projectStore.currentViewId === 'inbox' ? 'text-blue-600' : 'text-slate-400 group-hover:text-blue-500'" />
          <span class="flex-1 text-left">待办列表</span>
          <span v-if="projectStore.counts.inbox > 0" 
                :class="['text-xs font-medium px-2 py-0.5 rounded-full transition-colors', 
                         projectStore.currentViewId === 'inbox' ? 'bg-blue-100 text-blue-600' : 'bg-slate-200 text-slate-500 group-hover:bg-white']">
            {{ projectStore.counts.inbox }}
          </span>
        </button>
        
        <button 
          @click="selectProject('completed')"
          :class="['w-full flex items-center px-3 py-2.5 rounded-xl text-[14px] transition-all duration-200 group', 
                   projectStore.currentViewId === 'completed' 
                     ? 'bg-white text-emerald-600 shadow-sm font-semibold' 
                     : 'text-slate-600 hover:bg-slate-200/50 hover:text-slate-900']"
        >
          <CheckCircle class="w-4 h-4 mr-3" :class="projectStore.currentViewId === 'completed' ? 'text-emerald-600' : 'text-slate-400 group-hover:text-emerald-500'" />
          <span class="flex-1 text-left">已完成</span>
          <span v-if="projectStore.counts.completed > 0" 
                :class="['text-xs font-medium px-2 py-0.5 rounded-full transition-colors', 
                         projectStore.currentViewId === 'completed' ? 'bg-emerald-100 text-emerald-600' : 'bg-slate-200 text-slate-500 group-hover:bg-white']">
            {{ projectStore.counts.completed }}
          </span>
        </button>
      </div>

      <!-- Projects -->
      <div>
        <div class="flex items-center justify-between px-3 mb-3 group/header">
          <span class="text-xs font-bold text-slate-400 uppercase tracking-widest">我的项目</span>
          <button @click="addProject" class="text-slate-400 hover:text-blue-600 transition-colors opacity-0 group-hover/header:opacity-100 p-1 rounded-md hover:bg-slate-200/50" title="新建项目">
            <Plus class="w-4 h-4" />
          </button>
        </div>
        
        <div class="space-y-1">
          <div v-for="proj in projectStore.activeProjects.filter(p => !p.is_default)" :key="proj.id" class="group/item flex items-center relative">
            <button 
              @click="selectProject(proj.id)"
              :class="['flex-1 flex items-center px-3 py-2 rounded-xl text-[14px] transition-all duration-200', 
                       projectStore.currentViewId === proj.id 
                         ? 'bg-white text-slate-800 shadow-sm font-semibold' 
                         : 'text-slate-600 hover:bg-slate-200/50 hover:text-slate-900']"
            >
              <component :is="getIcon(proj.icon)" class="w-4 h-4 mr-3" :style="{ color: proj.color }" />
              <span class="flex-1 text-left truncate">{{ proj.name }}</span>
              
              <span v-if="projectStore.counts.projects[proj.id] > 0" 
                    :class="['text-xs font-medium px-2 py-0.5 rounded-full transition-colors', 
                             projectStore.currentViewId === proj.id ? 'bg-slate-100 text-slate-600' : 'bg-slate-200 text-slate-500 group-hover/item:bg-white']">
                {{ projectStore.counts.projects[proj.id] }}
              </span>
            </button>
            
            <button 
              @click="confirmDelete(proj.id)" 
              class="absolute right-2 opacity-0 group-hover/item:opacity-100 p-1.5 text-slate-400 hover:text-rose-500 hover:bg-rose-50 transition-all rounded-lg"
              title="删除项目"
            >
              <Trash2 class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      </div>
      
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: transparent;
  border-radius: 4px;
}
.custom-scrollbar:hover::-webkit-scrollbar-thumb {
  background-color: #cbd5e1;
}
</style>

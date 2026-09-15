<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import { useProjectStore } from '../stores/projectStore';
import { useTaskStore } from '../stores/taskStore';
import { useUiStore } from '../stores/uiStore';
import { useSettingsStore, type SidebarTab } from '../stores/settingsStore';
import { Inbox, Briefcase, User, CheckCircle, Plus, Trash2, Hash, Book, Star, Home, Code, Coffee, ShoppingCart, Check, Pencil, CalendarDays, LayoutGrid, BarChart3, Settings } from 'lucide-vue-next';
import type { Project } from '../types/task';

const projectStore = useProjectStore();
const taskStore = useTaskStore();
const uiStore = useUiStore();
const settingsStore = useSettingsStore();

// 底部 Tab：常用（系统视图 + 项目）/ 菜单（其他功能入口）
const activeTab = computed(() => settingsStore.settings.sidebarTab);

const sidebarTabs = [
  { id: 'common' as SidebarTab, label: '常用', icon: Star },
  { id: 'menu' as SidebarTab, label: '菜单', icon: LayoutGrid },
];

const setTab = (tab: SidebarTab) => settingsStore.patch({ sidebarTab: tab });

/** 菜单页的功能项 */
const menuItems = [
  { id: 'calendar', name: '日历', icon: CalendarDays, available: true },
  { id: 'trash', name: '回收站', icon: Trash2, available: true },
  { id: 'stats', name: '统计', icon: BarChart3, available: true },
  { id: 'settings', name: '设置', icon: Settings, available: true },
];

const selectMenuItem = (item: { id: string; name: string; available: boolean }) => {
  if (!item.available) {
    uiStore.showMessage(`「${item.name}」还在开发中`, 'info');
    return;
  }
  selectProject(item.id);
};

const getIcon = (iconName: string) => {
  switch (iconName) {
    case 'inbox': return Inbox;
    case 'briefcase': return Briefcase;
    case 'user': return User;
    case 'book': return Book;
    case 'star': return Star;
    case 'home': return Home;
    case 'code': return Code;
    case 'coffee': return Coffee;
    case 'cart': return ShoppingCart;
    default: return Hash;
  }
};

const iconOptions = ['briefcase', 'user', 'book', 'star', 'home', 'code', 'coffee', 'cart'];
const colorOptions = ['#3b82f6', '#ef4444', '#10b981', '#f59e0b', '#8b5cf6', '#ec4899', '#64748b'];

const showProjectDialog = ref(false);
const dialogMode = ref<'create' | 'edit'>('create');
const editingProjectId = ref<string | null>(null);
const newProjectName = ref('');
const selectedIcon = ref('briefcase');
const selectedColor = ref('#3b82f6');
const nameInputRef = ref<HTMLInputElement | null>(null);

watch(showProjectDialog, async (val) => {
  if (val) {
    if (dialogMode.value === 'create') {
      newProjectName.value = '';
      selectedIcon.value = 'briefcase';
      selectedColor.value = '#3b82f6';
    }
    await nextTick();
    nameInputRef.value?.focus();
  }
});

// Project Context Menu
const projectMenu = ref<{ show: boolean; x: number; y: number; project: Project | null }>({ show: false, x: 0, y: 0, project: null });

const onProjectContextMenu = (e: MouseEvent, proj: Project) => {
  const menuWidth = 150;
  const menuHeight = 100;
  projectMenu.value = {
    show: true,
    x: Math.min(e.clientX, window.innerWidth - menuWidth - 8),
    y: Math.min(e.clientY, window.innerHeight - menuHeight - 8),
    project: proj,
  };
};

const closeProjectMenu = () => {
  projectMenu.value.show = false;
};

const openEditDialog = () => {
  const proj = projectMenu.value.project;
  closeProjectMenu();
  if (!proj) return;
  dialogMode.value = 'edit';
  editingProjectId.value = proj.id;
  newProjectName.value = proj.name;
  selectedIcon.value = proj.icon;
  selectedColor.value = proj.color;
  showProjectDialog.value = true;
};

const deleteFromMenu = () => {
  const proj = projectMenu.value.project;
  closeProjectMenu();
  if (proj) confirmDelete(proj.id);
};

const selectProject = async (id: string) => {
  projectStore.currentViewId = id;
  await taskStore.loadTasks();
};

const submitProjectDialog = async () => {
  const name = newProjectName.value.trim();
  if (!name) return;
  try {
    if (dialogMode.value === 'create') {
      await projectStore.addProject(name, selectedColor.value, selectedIcon.value);
      uiStore.showMessage('项目创建成功', 'success');
    } else if (editingProjectId.value) {
      await projectStore.editProject(editingProjectId.value, name, selectedColor.value, selectedIcon.value);
      uiStore.showMessage('项目已更新', 'success');
    }
    showProjectDialog.value = false;
  } catch (e: any) {
    uiStore.showMessage(e.message || '操作失败', 'error');
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
    
    <div class="flex-1 overflow-y-auto px-4 py-4 pt-6 space-y-8 custom-scrollbar">

      <!-- 常用页：系统视图 + 我的项目 -->
      <template v-if="activeTab === 'common'">

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
          <button @click="dialogMode = 'create'; showProjectDialog = true" class="text-slate-400 hover:text-blue-600 transition-colors opacity-0 group-hover/header:opacity-100 p-1 rounded-md hover:bg-slate-200/50" title="新建项目">
            <Plus class="w-4 h-4" />
          </button>
        </div>
        
        <div class="space-y-1">
          <div v-for="proj in projectStore.activeProjects" :key="proj.id" class="group/item flex items-center relative" @contextmenu.prevent="!proj.is_default && onProjectContextMenu($event, proj)">
            <button 
              @click="selectProject(proj.id)"
              :class="['flex-1 flex items-center px-3 py-2 rounded-xl text-[14px] transition-all duration-200', 
                       projectStore.currentViewId === proj.id 
                         ? 'bg-white text-slate-800 shadow-sm font-semibold' 
                         : 'text-slate-600 hover:bg-slate-200/50 hover:text-slate-900']"
            >
              <component :is="getIcon(proj.icon)" class="w-4 h-4 mr-3" :style="{ color: proj.color }" />
              <span class="flex-1 text-left truncate">{{ proj.is_default ? '收件箱' : proj.name }}</span>
              
              <span v-if="projectStore.counts.projects[proj.id] > 0" 
                    :class="['text-xs font-medium px-2 py-0.5 rounded-full transition-colors', 
                             projectStore.currentViewId === proj.id ? 'bg-slate-100 text-slate-600' : 'bg-slate-200 text-slate-500 group-hover/item:bg-white']">
                {{ projectStore.counts.projects[proj.id] }}
              </span>
            </button>
            
            <button 
              v-if="!proj.is_default"
              @click="confirmDelete(proj.id)" 
              class="absolute right-2 opacity-0 group-hover/item:opacity-100 p-1.5 text-slate-400 hover:text-rose-500 hover:bg-rose-50 transition-all rounded-lg"
              title="删除项目"
            >
              <Trash2 class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      </div>

      </template>

      <!-- 菜单页：其他功能入口 -->
      <template v-else>
        <div>
          <div class="px-3 mb-3">
            <span class="text-xs font-bold text-slate-400 uppercase tracking-widest">功能</span>
          </div>

          <div class="space-y-1">
            <button
              v-for="item in menuItems"
              :key="item.id"
              @click="selectMenuItem(item)"
              :title="item.available ? item.name : `${item.name}（开发中）`"
              :class="['w-full flex items-center px-3 py-2.5 rounded-xl text-[14px] transition-all duration-200 group',
                       !item.available
                         ? 'text-slate-400 cursor-default'
                         : projectStore.currentViewId === item.id
                           ? 'bg-white text-blue-600 shadow-sm font-semibold'
                           : 'text-slate-600 hover:bg-slate-200/50 hover:text-slate-900']"
            >
              <component
                :is="item.icon"
                class="w-4 h-4 mr-3"
                :class="projectStore.currentViewId === item.id && item.available ? 'text-blue-600' : 'text-slate-400'"
              />
              <span class="flex-1 text-left">{{ item.name }}</span>
              <span v-if="!item.available" class="text-[10px] px-1.5 py-0.5 rounded-md bg-slate-200/70 text-slate-400">开发中</span>
            </button>
          </div>
        </div>
      </template>

    </div>

    <!-- 底部 Tab：常用 / 菜单 -->
    <div class="px-4 pt-2 pb-3 border-t border-slate-200/60">
      <div class="flex items-center bg-slate-200/60 rounded-xl p-0.5">
        <button
          v-for="tab in sidebarTabs"
          :key="tab.id"
          @click="setTab(tab.id)"
          :class="['flex-1 flex items-center justify-center space-x-1.5 py-1.5 rounded-lg text-xs font-medium transition-all duration-200',
                   activeTab === tab.id ? 'bg-white text-slate-800 shadow-sm' : 'text-slate-500 hover:text-slate-700']"
        >
          <component :is="tab.icon" class="w-3.5 h-3.5" />
          <span>{{ tab.label }}</span>
        </button>
      </div>
    </div>

    <!-- Project Dialog (Create / Edit) -->
    <Teleport to="body">
      <Transition name="fade">
        <div v-if="showProjectDialog" class="fixed inset-0 z-[120] flex items-center justify-center p-4">
          <div class="absolute inset-0 bg-slate-900/30 backdrop-blur-sm" @click="showProjectDialog = false"></div>

          <div class="relative bg-white rounded-xl shadow-2xl border border-slate-100 w-full max-w-sm overflow-hidden p-6" @click.stop>
            <h3 class="text-lg font-bold text-slate-900 mb-4">{{ dialogMode === 'create' ? '新建项目' : '编辑项目' }}</h3>

            <div class="space-y-4">
              <div>
                <label class="block text-xs font-medium text-slate-500 mb-1">项目名称</label>
                <input
                  ref="nameInputRef"
                  v-model="newProjectName"
                  @keydown.enter="submitProjectDialog"
                  @keydown.esc="showProjectDialog = false"
                  type="text"
                  placeholder="输入项目名称..."
                  class="w-full bg-slate-50 rounded-md px-3 py-2 text-sm text-slate-700 ring-1 ring-inset ring-slate-200 focus:ring-blue-500 focus:outline-none"
                />
              </div>

              <div>
                <label class="block text-xs font-medium text-slate-500 mb-2">图标</label>
                <div class="grid grid-cols-8 gap-2">
                  <button
                    v-for="icon in iconOptions"
                    :key="icon"
                    @click="selectedIcon = icon"
                    :class="['aspect-square flex items-center justify-center rounded-lg transition-all',
                             selectedIcon === icon
                               ? 'bg-slate-100 ring-2 ring-blue-500'
                               : 'hover:bg-slate-50 ring-1 ring-inset ring-slate-200']"
                  >
                    <component :is="getIcon(icon)" class="w-4 h-4" :style="{ color: selectedColor }" />
                  </button>
                </div>
              </div>

              <div>
                <label class="block text-xs font-medium text-slate-500 mb-2">颜色</label>
                <div class="flex items-center gap-2.5">
                  <button
                    v-for="color in colorOptions"
                    :key="color"
                    @click="selectedColor = color"
                    class="w-7 h-7 rounded-full flex items-center justify-center transition-transform hover:scale-110"
                    :style="{ backgroundColor: color }"
                  >
                    <Check v-if="selectedColor === color" class="w-4 h-4 text-white" />
                  </button>
                </div>
              </div>
            </div>

            <div class="mt-6 flex justify-end space-x-3">
              <button @click="showProjectDialog = false" class="px-4 py-2 rounded-lg text-sm text-slate-600 hover:bg-slate-100">取消</button>
              <button
                @click="submitProjectDialog"
                :disabled="!newProjectName.trim()"
                class="px-4 py-2 rounded-lg text-sm text-white bg-blue-600 hover:bg-blue-700 shadow-sm disabled:opacity-50 disabled:cursor-not-allowed"
              >{{ dialogMode === 'create' ? '创建' : '保存' }}</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Project Context Menu -->
    <Teleport to="body">
      <div v-if="projectMenu.show" class="fixed inset-0 z-[130]" @click="closeProjectMenu" @contextmenu.prevent="closeProjectMenu">
        <div
          class="absolute w-36 bg-white rounded-xl shadow-xl border border-slate-100 py-1.5 overflow-hidden"
          :style="{ left: projectMenu.x + 'px', top: projectMenu.y + 'px' }"
          @click.stop
        >
          <button
            @click="openEditDialog"
            class="w-full flex items-center px-3 py-1.5 text-[13px] text-slate-700 hover:bg-slate-50 transition-colors"
          >
            <Pencil class="w-3.5 h-3.5 mr-2.5 text-slate-400" />
            <span>编辑项目</span>
          </button>
          <button
            @click="deleteFromMenu"
            class="w-full flex items-center px-3 py-1.5 text-[13px] text-rose-600 hover:bg-rose-50 transition-colors"
          >
            <Trash2 class="w-3.5 h-3.5 mr-2.5" />
            <span>删除项目</span>
          </button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
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

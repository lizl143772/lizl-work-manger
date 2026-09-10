<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import type { Task, TaskStatus } from '../types/task';
import { useTaskStore } from '../stores/taskStore';
import { useUiStore } from '../stores/uiStore';
import { useProjectStore } from '../stores/projectStore';
import { Circle, CheckCircle2, PlayCircle, Trash2, Flag, X, Clock, Calendar, Check } from 'lucide-vue-next';
import Vditor from 'vditor';
import 'vditor/dist/index.css';
import { marked } from 'marked';

const props = defineProps<{ task: Task }>();
const taskStore = useTaskStore();
const uiStore = useUiStore();
const projectStore = useProjectStore();

const isExpanded = computed(() => taskStore.expandedTaskId === props.task.id);

const editTitle = ref(props.task.title);
const editDesc = ref(props.task.description || '');

const formatTimeInput = (iso?: string | null) => {
  if (!iso) return '';
  const d = new Date(iso);
  const tzOffset = d.getTimezoneOffset() * 60000;
  return new Date(d.getTime() - tzOffset).toISOString().slice(0, 16);
};

const showTimeModal = ref(false);
const editDueDate = ref(formatTimeInput(props.task.due_date));
const editCompletedAt = ref(formatTimeInput(props.task.completed_at));

// Watch inputs and auto-calculate time spent
const autoTimeSpent = computed(() => {
  if (editDueDate.value && editCompletedAt.value) {
    const start = new Date(editDueDate.value).getTime();
    const end = new Date(editCompletedAt.value).getTime();
    const diff = Math.round((end - start) / 60000);
    return diff > 0 ? diff : 0;
  }
  return props.task.time_spent || 0;
});

const attachments = computed<string[]>(() => {
  if (!props.task.attachments) return [];
  try {
    return JSON.parse(props.task.attachments);
  } catch {
    return [];
  }
});

const renderedMarkdown = computed(() => {
  if (!props.task.description) return '';
  return marked.parse(props.task.description, { gfm: true, breaks: true }) as string;
});

const formatDate = (iso: string) => {
  return new Date(iso).toLocaleString('zh-CN', {
    month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit'
  });
};

const toggleStatus = () => {
  let next: TaskStatus = 'todo';
  if (props.task.status === 'todo') next = 'in_progress';
  else if (props.task.status === 'in_progress') next = 'completed';
  else next = 'todo';
  taskStore.updateStatus(props.task.id, next);
};

const saveDetails = async () => {
  const t = editTitle.value.trim();
  if (t) {
    await taskStore.updateDetails(props.task.id, {
      title: t,
      description: editDesc.value.trim() || null,
      due_date: editDueDate.value ? new Date(editDueDate.value).toISOString() : null,
      completed_at: editCompletedAt.value ? new Date(editCompletedAt.value).toISOString() : null,
      time_spent: autoTimeSpent.value > 0 ? autoTimeSpent.value : null
    });
  }
};

const saveTimeAndClose = () => {
  saveDetails();
  showTimeModal.value = false;
};

const vditorContainer = ref<HTMLElement | null>(null);
let vditorInstance: Vditor | null = null;

const toggleExpand = () => {
  taskStore.toggleExpanded(props.task.id);
};

const handleVditorKeydown = (e: KeyboardEvent) => {
  if (!vditorInstance || !vditorContainer.value) return;

  // Auto-word selection for formatting (Ctrl+B, Ctrl+I, Ctrl+U)
  if (e.ctrlKey && !e.altKey && !e.shiftKey) {
    const key = e.key.toLowerCase();
    if (key === 'b' || key === 'i' || key === 'u') {
      const sel = window.getSelection();
      if (sel && sel.isCollapsed) {
        e.preventDefault();
        sel.modify('extend', 'backward', 'word');
        const text = sel.toString();
        if (text.trim().length > 0) {
          if (key === 'b') document.execCommand('bold');
          else if (key === 'i') document.execCommand('italic');
          else if (key === 'u') document.execCommand('underline');
        } else {
          // No text, trigger Vditor's native toolbar button if available
          const btnName = key === 'b' ? 'bold' : key === 'i' ? 'italic' : 'strike';
          const btn = vditorInstance.vditor?.toolbar?.elements?.[btnName]?.firstElementChild as HTMLElement;
          if (btn) btn.click();
          else if (key === 'b') vditorInstance.insertValue('****');
          else if (key === 'i') vditorInstance.insertValue('**');
        }
        sel.collapseToEnd();
        return;
      }
    }
  }

  // Ctrl+1 ~ Ctrl+6 Headings
  if (e.ctrlKey && !e.shiftKey && !e.altKey && e.key >= '1' && e.key <= '6') {
    e.preventDefault();
    document.execCommand('formatBlock', false, `H${e.key}`);
    return;
  }

  // Ctrl+0 Paragraph
  if (e.ctrlKey && !e.shiftKey && !e.altKey && e.key === '0') {
    e.preventDefault();
    document.execCommand('formatBlock', false, 'P');
    return;
  }

  // Ctrl+Shift+M Math
  if (e.ctrlKey && e.shiftKey && (e.key === 'm' || e.key === 'M')) {
    e.preventDefault();
    vditorInstance.insertValue('$$\n\n$$');
    return;
  }

  // Ctrl+Shift+K Code Block
  if (e.ctrlKey && e.shiftKey && (e.key === 'k' || e.key === 'K')) {
    e.preventDefault();
    vditorInstance.insertValue('```\n\n```');
    return;
  }

  // Ctrl+Shift+Q Quote
  if (e.ctrlKey && e.shiftKey && (e.key === 'q' || e.key === 'Q')) {
    e.preventDefault();
    document.execCommand('formatBlock', false, 'BLOCKQUOTE');
    return;
  }

  // Ctrl+Shift+[ Ordered List
  if (e.ctrlKey && e.shiftKey && e.code === 'BracketLeft') {
    e.preventDefault();
    document.execCommand('insertOrderedList');
    return;
  }

  // Ctrl+Shift+] Unordered List
  if (e.ctrlKey && e.shiftKey && e.code === 'BracketRight') {
    e.preventDefault();
    document.execCommand('insertUnorderedList');
    return;
  }

  // Ctrl+Shift+X Task List
  if (e.ctrlKey && e.shiftKey && (e.key === 'x' || e.key === 'X')) {
    e.preventDefault();
    vditorInstance.insertValue('- [ ] ');
    return;
  }

  // ~~~~ quick code block on Enter
  if (e.key === 'Enter' && !e.shiftKey && !e.ctrlKey) {
    const sel = window.getSelection();
    if (!sel || sel.rangeCount === 0) return;
    const node = sel.anchorNode;
    if (node && node.nodeType === 3) {
      const text = node.textContent || '';
      if (/^~{3,4}$/.test(text.trim()) || /^`{3,4}$/.test(text.trim())) {
        e.preventDefault();
        const range = document.createRange();
        range.selectNodeContents(node);
        sel.removeAllRanges();
        sel.addRange(range);
        document.execCommand('delete');
        vditorInstance.insertValue('```\n\n```');
        return;
      }
    }
  }
};

watch(isExpanded, async (val) => {
  if (val) {
    editTitle.value = props.task.title;
    editDesc.value = props.task.description || '';
    
    await nextTick();
    if (vditorContainer.value && !vditorInstance) {
      vditorInstance = new Vditor(vditorContainer.value, {
        mode: 'ir',
        value: editDesc.value,
        cache: { enable: false },
        outline: { enable: false, position: 'left' },
        toolbar: ['headings', 'bold', 'italic', 'strike', '|', 'list', 'ordered-list', 'check', '|', 'quote', 'code', 'inline-code', 'table', '|', 'undo', 'redo'],
        height: 'auto',
        minHeight: 150,
        input(value) {
          editDesc.value = value;
        },
        blur() {
          saveDetails();
        },
        upload: {
          handler(files) {
            onUploadImg(files);
            return null;
          }
        }
      });
      // Attach the keydown listener to support Typora shortcuts
      vditorContainer.value.addEventListener('keydown', handleVditorKeydown, true);
    }
  } else {
    saveDetails();
    if (vditorInstance) {
      if (vditorContainer.value) {
        vditorContainer.value.removeEventListener('keydown', handleVditorKeydown, true);
      }
      vditorInstance.destroy();
      vditorInstance = null;
    }
  }
});

const onPaste = async (e: ClipboardEvent) => {
  if (!isExpanded.value) return;
  const items = e.clipboardData?.items;
  if (!items || items.length === 0) return;
  
  const newImages: string[] = [];
  
  for (let i = 0; i < items.length; i++) {
    const item = items[i];
    if (item.type.startsWith('image/') || item.kind === 'file') {
      const file = item.getAsFile();
      if (file && file.type.startsWith('image/')) {
        const reader = new FileReader();
        const p = new Promise<string>((resolve) => {
          reader.onload = (ev) => resolve(ev.target?.result as string);
        });
        reader.readAsDataURL(file);
        newImages.push(await p);
      }
    }
  }
  
  if (newImages.length > 0) {
    e.preventDefault();
    const current = [...attachments.value, ...newImages];
    await taskStore.updateDetails(props.task.id, { attachments: JSON.stringify(current) });
  }
};

const onUploadImg = async (files: Array<File>) => {
  const newImages: string[] = [];
  for (const file of files) {
    if (file.type.startsWith('image/')) {
      const reader = new FileReader();
      const p = new Promise<string>((resolve) => {
        reader.onload = (ev) => resolve(ev.target?.result as string);
      });
      reader.readAsDataURL(file);
      newImages.push(await p);
    }
  }
  if (newImages.length > 0) {
    const current = [...attachments.value, ...newImages];
    await taskStore.updateDetails(props.task.id, { attachments: JSON.stringify(current) });
  }
};

const removeImage = async (index: number) => {
  const current = [...attachments.value];
  current.splice(index, 1);
  await taskStore.updateDetails(props.task.id, { attachments: JSON.stringify(current) });
};

const priorityInfo = computed(() => {
  switch(props.task.priority) {
    case 1: return { text: '低', color: 'text-blue-600 bg-blue-50 border-blue-100', iconColor: 'text-blue-500' };
    case 2: return { text: '中', color: 'text-amber-600 bg-amber-50 border-amber-100', iconColor: 'text-amber-500' };
    case 3: return { text: '高', color: 'text-rose-600 bg-rose-50 border-rose-100', iconColor: 'text-rose-500' };
    default: return { text: '无', color: 'text-slate-500 bg-slate-50 border-slate-200', iconColor: 'text-slate-400' };
  }
});

const cardBorderClass = computed(() => {
  switch(props.task.priority) {
    case 1: return 'border-blue-200 hover:border-blue-400';
    case 2: return 'border-amber-300 hover:border-amber-500';
    case 3: return 'border-rose-400 hover:border-rose-600';
    default: return 'border-slate-100 hover:border-slate-300';
  }
});

const taskProject = computed(() => projectStore.projects.find(p => p.id === props.task.project_id));
const showProjectBadge = computed(() => projectStore.currentViewId === 'inbox' && !!taskProject.value);

const changePriority = () => {
  const next = (props.task.priority + 1) % 4;
  taskStore.updatePriority(props.task.id, next);
};

const del = () => {
  taskStore.removeTask(props.task.id);
};

// Context Menu
const contextMenu = ref({ show: false, x: 0, y: 0 });

const priorityOptions = [
  { value: 0, text: '无优先级', iconColor: 'text-slate-400' },
  { value: 1, text: '低优先级', iconColor: 'text-blue-500' },
  { value: 2, text: '中优先级', iconColor: 'text-amber-500' },
  { value: 3, text: '高优先级', iconColor: 'text-rose-500' },
];

const onContextMenu = (e: MouseEvent) => {
  const menuWidth = 180;
  const menuHeight = 360;
  contextMenu.value = {
    show: true,
    x: Math.min(e.clientX, window.innerWidth - menuWidth - 8),
    y: Math.min(e.clientY, window.innerHeight - menuHeight - 8),
  };
};

const closeContextMenu = () => {
  contextMenu.value.show = false;
};

const setPriority = (p: number) => {
  closeContextMenu();
  if (p !== props.task.priority) taskStore.updatePriority(props.task.id, p);
};

const moveToProject = async (projectId: string) => {
  closeContextMenu();
  if (projectId === props.task.project_id) return;
  try {
    await taskStore.moveTask(props.task.id, projectId);
    const proj = projectStore.projects.find(p => p.id === projectId);
    uiStore.showMessage(`已移动到「${proj?.name || '项目'}」`, 'success');
  } catch (e: any) {
    uiStore.showMessage(e.message || '移动任务失败', 'error');
  }
};

const deleteFromMenu = () => {
  closeContextMenu();
  del();
};

</script>

<template>
  <div 
    class="group relative flex flex-col px-4 py-3 mx-6 mb-3 bg-white rounded-xl border shadow-[0_1px_2px_rgba(0,0,0,0.02)] hover:shadow-sm transition-all duration-200 cursor-pointer"
    :class="cardBorderClass"
    @click="toggleExpand"
    @contextmenu.prevent="onContextMenu"
    @paste="onPaste"
    tabindex="0"
  >
    <!-- Top Row -->
    <div class="flex items-start">
      <button @click.stop="toggleStatus" class="mr-4 mt-0.5 flex-shrink-0 transition-transform active:scale-95 focus:outline-none">
        <Circle v-if="task.status === 'todo'" class="w-5 h-5 text-slate-300 hover:text-blue-500 transition-colors" />
        <PlayCircle v-else-if="task.status === 'in_progress'" class="w-5 h-5 text-blue-500" />
        <CheckCircle2 v-else class="w-5 h-5 text-emerald-500" />
      </button>
      
      <div class="flex-1 min-w-0">
        
        <!-- Expanded Title Edit -->
        <div v-if="isExpanded" @click.stop>
          <input 
            v-model="editTitle" 
            @change="saveDetails"
            @keydown.enter="($event.target as HTMLInputElement).blur()"
            type="text"
            class="w-full bg-transparent border-0 px-0 py-0 text-[15px] font-medium text-slate-800 focus:outline-none focus:ring-0 mb-2"
            placeholder="任务标题"
          />
        </div>
        
        <!-- Collapsed Title Display -->
        <span v-else 
          :class="[
            'text-[15px] font-medium block transition-colors duration-200', 
            task.status === 'completed' ? 'text-slate-400 line-through' : 'text-slate-800'
          ]"
        >
          {{ task.title }}
        </span>
        
        <!-- Quick Metadata & Rendered Markdown when collapsed -->
        <div v-if="!isExpanded">
          <div v-if="renderedMarkdown" class="prose prose-sm prose-slate max-w-none prose-p:my-1 prose-ul:my-1 prose-ol:my-1 opacity-70 mb-2 line-clamp-3 text-[13px]" v-html="renderedMarkdown"></div>
          
          <div class="flex flex-wrap items-center gap-3 mt-1.5 pr-10 text-[11px] text-slate-400">
            <span v-if="showProjectBadge" class="flex items-center space-x-1 px-1.5 py-0.5 rounded-md bg-slate-50 border border-slate-100 text-slate-500" :title="'所属项目: ' + taskProject!.name">
              <span class="w-1.5 h-1.5 rounded-full flex-shrink-0" :style="{ backgroundColor: taskProject!.color }"></span>
              <span>{{ taskProject!.name }}</span>
            </span>
            <span class="flex items-center space-x-1" title="录入时间">
              <span>录入: {{ formatDate(task.created_at) }}</span>
            </span>
            <span v-if="task.due_date" class="flex items-center space-x-1" title="任务时间">
              <Calendar class="w-3 h-3" />
              <span>{{ formatDate(task.due_date) }}</span>
            </span>
            <span v-if="task.completed_at || task.status === 'completed'" class="flex items-center space-x-1 text-emerald-500" title="完成时间">
              <CheckCircle2 class="w-3 h-3" />
              <span>{{ task.completed_at ? formatDate(task.completed_at) : '已完成' }}</span>
            </span>
            <span v-if="task.time_spent" class="flex items-center space-x-1" title="已耗时">
              <Clock class="w-3 h-3" />
              <span>{{ task.time_spent }} min</span>
            </span>
          </div>
        </div>
      </div>

      <!-- Collapsed Priority Toggle -->
      <button
        v-if="!isExpanded"
        @click.stop="changePriority"
        :class="['ml-3 mt-0.5 flex-shrink-0 flex items-center space-x-1 text-[11px] px-2 py-1 rounded-md border font-medium transition-colors', priorityInfo.color]"
        title="切换优先级"
      >
        <Flag class="w-3 h-3" :class="priorityInfo.iconColor" />
        <span>{{ priorityInfo.text }}</span>
      </button>
    </div>
    
    <!-- Expanded Area -->
    <div v-if="isExpanded" class="mt-2 pl-9" @click.stop>
      
      <!-- Vditor WYSIWYG Editor -->
      <div class="mb-4">
        <div ref="vditorContainer" class="!border-slate-200 !rounded-lg !shadow-sm !bg-slate-50 min-h-[150px]"></div>
      </div>
      
      <!-- Actions: Bottom Left -->
      <div class="flex items-center space-x-3 pt-1 pr-10">
        <button 
          @click.stop="changePriority" 
          :class="['flex items-center space-x-1 text-xs px-2 py-1 rounded-md border font-medium transition-colors', priorityInfo.color]"
          title="切换优先级"
        >
          <Flag class="w-3 h-3" :class="priorityInfo.iconColor" />
          <span>{{ priorityInfo.text }}</span>
        </button>

        <button 
          @click.stop="showTimeModal = true" 
          class="flex items-center space-x-1 px-2 py-1 text-xs text-slate-500 bg-slate-50 hover:bg-slate-100 border border-slate-200 rounded-md transition-colors"
          title="编辑时间与耗时"
        >
          <Calendar class="w-3 h-3" />
          <span>时间规划</span>
        </button>
        
        <div class="flex-1"></div>
        <span class="text-[11px] text-slate-400 italic">在卡片内使用 Ctrl+V 可直接上传图片</span>
      </div>
    </div>
    
    <!-- Image Thumbnails (Always visible at bottom) -->
    <div v-if="attachments.length > 0" class="flex flex-wrap gap-2 mt-3 pl-9 pr-10">
      <div v-for="(img, idx) in attachments" :key="idx" class="relative group/img">
        <img @click.stop="uiStore.openPreview(attachments, idx)" :src="img" class="h-16 w-16 object-cover rounded-md border border-slate-200 shadow-sm cursor-pointer hover:opacity-90 transition-opacity" title="点击放大预览" />
        <button @click.stop="removeImage(idx)" class="absolute -top-1.5 -right-1.5 bg-rose-500 text-white rounded-full p-0.5 opacity-0 group-hover/img:opacity-100 transition-opacity z-10" title="删除图片">
          <X class="w-3 h-3" />
        </button>
      </div>
    </div>

    <!-- Time Setting Modal (Local to TaskItem) -->
    <Teleport to="body">
      <Transition name="fade">
        <div v-if="showTimeModal" class="fixed inset-0 z-[120] flex items-center justify-center p-4">
          <div class="absolute inset-0 bg-slate-900/30 backdrop-blur-sm" @click="showTimeModal = false"></div>
          
          <div class="relative bg-white rounded-xl shadow-2xl border border-slate-100 w-full max-w-sm overflow-hidden p-6" @click.stop>
            <h3 class="text-lg font-bold text-slate-900 mb-4">时间规划</h3>
            
            <div class="space-y-4">
              <div>
                <label class="block text-xs font-medium text-slate-500 mb-1">任务时间 (预定)</label>
                <input v-model="editDueDate" type="datetime-local" class="w-full bg-slate-50 rounded-md px-3 py-2 text-sm text-slate-700 ring-1 ring-inset ring-slate-200 focus:ring-blue-500 focus:outline-none" />
              </div>
              <div>
                <label class="block text-xs font-medium text-slate-500 mb-1">完成时间 (实际)</label>
                <input v-model="editCompletedAt" type="datetime-local" class="w-full bg-slate-50 rounded-md px-3 py-2 text-sm text-slate-700 ring-1 ring-inset ring-slate-200 focus:ring-blue-500 focus:outline-none" />
              </div>
              <div>
                <label class="block text-xs font-medium text-slate-500 mb-1">耗时 (分钟)</label>
                <div class="w-full bg-slate-100 rounded-md px-3 py-2 text-sm text-slate-500 ring-1 ring-inset ring-slate-200 cursor-not-allowed">
                  {{ autoTimeSpent > 0 ? autoTimeSpent + ' 分钟' : '自动计算...' }}
                </div>
                <p class="text-[10px] text-slate-400 mt-1">耗时由完成时间减去任务时间自动计算得出</p>
              </div>
            </div>
            
            <div class="mt-6 flex justify-end space-x-3">
              <button @click="showTimeModal = false" class="px-4 py-2 rounded-lg text-sm text-slate-600 hover:bg-slate-100">取消</button>
              <button @click="saveTimeAndClose" class="px-4 py-2 rounded-lg text-sm text-white bg-blue-600 hover:bg-blue-700 shadow-sm">保存并关闭</button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Context Menu -->
    <Teleport to="body">
      <div v-if="contextMenu.show" class="fixed inset-0 z-[130]" @click="closeContextMenu" @contextmenu.prevent="closeContextMenu">
        <div 
          class="absolute w-44 bg-white rounded-xl shadow-xl border border-slate-100 py-1.5 overflow-hidden"
          :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
          @click.stop
        >
          <div class="px-3 py-1 text-[10px] font-bold text-slate-400 uppercase tracking-wider">标记优先级</div>
          <button 
            v-for="opt in priorityOptions" 
            :key="opt.value"
            @click="setPriority(opt.value)"
            class="w-full flex items-center px-3 py-1.5 text-[13px] text-slate-700 hover:bg-slate-50 transition-colors"
          >
            <Flag class="w-3.5 h-3.5 mr-2.5" :class="opt.iconColor" />
            <span class="flex-1 text-left">{{ opt.text }}</span>
            <Check v-if="task.priority === opt.value" class="w-3.5 h-3.5 text-blue-500" />
          </button>

          <div class="my-1 border-t border-slate-100"></div>

          <div class="px-3 py-1 text-[10px] font-bold text-slate-400 uppercase tracking-wider">设置项目</div>
          <div class="max-h-36 overflow-y-auto">
            <button 
              v-for="proj in projectStore.activeProjects" 
              :key="proj.id"
              @click="moveToProject(proj.id)"
              class="w-full flex items-center px-3 py-1.5 text-[13px] text-slate-700 hover:bg-slate-50 transition-colors"
            >
              <span class="w-2 h-2 rounded-full mr-2.5 flex-shrink-0" :style="{ backgroundColor: proj.color }"></span>
              <span class="flex-1 text-left truncate">{{ proj.name }}</span>
              <Check v-if="task.project_id === proj.id" class="w-3.5 h-3.5 text-blue-500 flex-shrink-0" />
            </button>
          </div>

          <div class="my-1 border-t border-slate-100"></div>

          <button 
            @click="deleteFromMenu"
            class="w-full flex items-center px-3 py-1.5 text-[13px] text-rose-600 hover:bg-rose-50 transition-colors"
          >
            <Trash2 class="w-3.5 h-3.5 mr-2.5" />
            <span>删除任务</span>
          </button>
        </div>
      </div>
    </Teleport>

    <!-- Delete Button (always visible at bottom-right) -->
    <button
      @click.stop="del"
      class="absolute bottom-3 right-3 p-1.5 text-slate-300 hover:text-rose-500 hover:bg-rose-50 rounded-lg transition-colors z-10"
      title="删除任务"
    >
      <Trash2 class="w-4 h-4" />
    </button>

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
</style>

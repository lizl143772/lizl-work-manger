<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useTaskStore } from '../stores/taskStore';
import { useProjectStore } from '../stores/projectStore';
import { useUiStore } from '../stores/uiStore';
import { Plus, X } from 'lucide-vue-next';

const taskStore = useTaskStore();
const projectStore = useProjectStore();
const uiStore = useUiStore();

const inputVal = ref('');
const inputRef = ref<HTMLInputElement | null>(null);
const isSubmitting = ref(false);
const pendingImages = ref<string[]>([]);

const submit = async () => {
  const val = inputVal.value.trim();
  if (!val && pendingImages.value.length === 0) return;
  if (isSubmitting.value) return;
  
  isSubmitting.value = true;
  try {
    const finalTitle = val || '图片任务';
    const attachments = pendingImages.value.length > 0 ? JSON.stringify(pendingImages.value) : null;
    
    await taskStore.quickAdd(finalTitle, attachments);
    
    inputVal.value = '';
    pendingImages.value = [];
    inputRef.value?.focus();
    uiStore.showMessage('任务已添加', 'success');
  } catch (e: any) {
    uiStore.showMessage(e.message || '录入失败', 'error');
  } finally {
    isSubmitting.value = false;
  }
};

const onPaste = async (e: ClipboardEvent) => {
  const items = e.clipboardData?.items;
  if (!items || items.length === 0) return;
  
  let foundImage = false;
  for (let i = 0; i < items.length; i++) {
    const item = items[i];
    if (item.type.startsWith('image/') || item.kind === 'file') {
      const file = item.getAsFile();
      if (file && file.type.startsWith('image/')) {
        foundImage = true;
        const reader = new FileReader();
        reader.onload = (ev) => {
          if (ev.target?.result) {
            pendingImages.value.push(ev.target.result as string);
            uiStore.showMessage('图片粘贴成功', 'success');
          }
        };
        reader.onerror = () => {
          uiStore.showMessage('读取图片失败', 'error');
        };
        reader.readAsDataURL(file);
      }
    }
  }
  
  if (foundImage) {
    e.preventDefault(); // Prevent default text paste if it contains images
  }
};

const removePendingImage = (index: number) => {
  pendingImages.value.splice(index, 1);
};

onMounted(() => {
  inputRef.value?.focus();
});
</script>

<template>
  <div class="relative flex flex-col group">
    <!-- Pending Image Thumbnails -->
    <div v-if="pendingImages.length > 0" class="flex flex-wrap gap-2 mb-3">
      <div v-for="(img, idx) in pendingImages" :key="idx" class="relative group/img">
        <img :src="img" class="h-16 w-16 object-cover rounded-md border border-slate-200 shadow-sm" />
        <button @click.stop="removePendingImage(idx)" class="absolute -top-1.5 -right-1.5 bg-rose-500 text-white rounded-full p-0.5 opacity-0 group-hover/img:opacity-100 transition-opacity z-10">
          <X class="w-3 h-3" />
        </button>
      </div>
    </div>

    <!-- Input Field -->
    <div class="relative">
      <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none text-slate-400 group-focus-within:text-blue-500 transition-colors">
        <Plus class="w-5 h-5" />
      </div>
      <input 
        ref="inputRef"
        v-model="inputVal"
        @keydown.enter="submit"
        @paste="onPaste"
        :disabled="isSubmitting"
        type="text" 
        class="block w-full pl-12 pr-16 py-3.5 bg-slate-50 border-0 text-slate-800 rounded-xl ring-1 ring-inset ring-slate-200 placeholder:text-slate-400 focus:ring-2 focus:ring-inset focus:ring-blue-500 transition-shadow shadow-sm text-sm"
        :placeholder="`添加任务至「${projectStore.currentViewName}」... (支持 Ctrl+V 粘贴图片)`"
      />
      <div class="absolute inset-y-0 right-0 pr-4 flex items-center">
        <span class="inline-flex items-center rounded-md bg-white px-2 py-1 text-xs font-medium text-slate-400 ring-1 ring-inset ring-slate-200 shadow-sm transition-opacity"
              :class="inputVal || pendingImages.length > 0 ? 'opacity-100' : 'opacity-0'">
          Enter ↵
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';
import { useUiStore } from '../stores/uiStore';
import { CheckCircle2, AlertCircle, Info, X } from 'lucide-vue-next';

const uiStore = useUiStore();
const promptInput = ref('');
const promptInputRef = ref<HTMLInputElement | null>(null);

const previewContainer = ref<HTMLElement | null>(null);

// Auto-focus prompt input when modal opens
watch(() => uiStore.modal, async (newVal) => {
  if (newVal?.type === 'prompt') {
    promptInput.value = newVal.defaultValue || '';
    await nextTick();
    promptInputRef.value?.focus();
  }
});

// Auto-focus preview container when preview opens for keyboard navigation
watch(() => uiStore.isPreviewOpen, async (isOpen) => {
  if (isOpen) {
    await nextTick();
    previewContainer.value?.focus();
  }
});

const submitModal = () => {
  if (!uiStore.modal) return;
  if (uiStore.modal.type === 'confirm') {
    uiStore.closeModal(true);
  } else {
    uiStore.closeModal(promptInput.value);
  }
};

const cancelModal = () => {
  if (!uiStore.modal) return;
  if (uiStore.modal.type === 'confirm') {
    uiStore.closeModal(false);
  } else {
    uiStore.closeModal(null);
  }
};
</script>

<template>
  <div>
    <!-- Global Toasts (Message) -->
    <div class="fixed top-6 left-1/2 -translate-x-1/2 z-[100] flex flex-col items-center space-y-3 pointer-events-none">
      <TransitionGroup name="toast">
        <div 
          v-for="toast in uiStore.toasts" 
          :key="toast.id"
          class="pointer-events-auto flex items-center px-4 py-3 rounded-lg shadow-lg border bg-white min-w-[300px] transition-all duration-300"
          :class="{
            'border-emerald-100 shadow-emerald-500/10': toast.type === 'success',
            'border-rose-100 shadow-rose-500/10': toast.type === 'error',
            'border-blue-100 shadow-blue-500/10': toast.type === 'info',
          }"
        >
          <CheckCircle2 v-if="toast.type === 'success'" class="w-5 h-5 text-emerald-500 mr-3 shrink-0" />
          <AlertCircle v-else-if="toast.type === 'error'" class="w-5 h-5 text-rose-500 mr-3 shrink-0" />
          <Info v-else class="w-5 h-5 text-blue-500 mr-3 shrink-0" />
          
          <span class="text-[14px] text-slate-700 flex-1">{{ toast.message }}</span>
        </div>
      </TransitionGroup>
    </div>

    <!-- Global Modals (Dialog) -->
    <Transition name="fade">
      <div v-if="uiStore.modal" class="fixed inset-0 z-[90] flex items-center justify-center p-4" data-no-collapse>
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-slate-900/20 backdrop-blur-sm" @click="cancelModal"></div>
        
        <!-- Dialog content -->
        <div class="relative bg-white rounded-2xl shadow-2xl border border-slate-100 w-full max-w-sm overflow-hidden scale-100 transition-transform">
          
          <div class="px-6 pt-6 pb-4">
            <h3 class="text-lg font-bold text-slate-900">{{ uiStore.modal.title }}</h3>
            
            <!-- Confirm specific -->
            <p v-if="uiStore.modal.type === 'confirm' && uiStore.modal.message" class="mt-2 text-sm text-slate-500">
              {{ uiStore.modal.message }}
            </p>

            <!-- Prompt specific -->
            <div v-if="uiStore.modal.type === 'prompt'" class="mt-4">
              <input 
                ref="promptInputRef"
                v-model="promptInput"
                @keydown.enter="submitModal"
                @keydown.esc="cancelModal"
                type="text" 
                class="w-full px-3 py-2 bg-slate-50 border border-slate-200 rounded-lg text-sm text-slate-800 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-blue-500 transition-all"
              />
            </div>
          </div>
          
          <div class="px-6 py-4 bg-slate-50 flex items-center justify-end space-x-3 border-t border-slate-100">
            <button 
              @click="cancelModal" 
              class="px-4 py-2 rounded-lg text-sm font-medium text-slate-600 hover:bg-slate-200/50 hover:text-slate-900 transition-colors"
            >
              取消
            </button>
            <button 
              @click="submitModal" 
              class="px-4 py-2 rounded-lg text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 shadow-sm transition-colors"
            >
              确定
            </button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Global Image Preview -->
    <Transition name="fade">
      <div v-if="uiStore.isPreviewOpen" class="fixed inset-0 z-[110] flex items-center justify-center bg-slate-900/70 backdrop-blur-md" data-no-collapse tabindex="0" @keydown.esc="uiStore.closePreview" @keydown.left="uiStore.prevPreview" @keydown.right="uiStore.nextPreview" ref="previewContainer">
        
        <!-- Close Button -->
        <button @click="uiStore.closePreview" class="absolute top-6 right-6 p-2 rounded-full bg-white/10 text-white/70 hover:text-white hover:bg-white/20 transition-colors z-[111]">
          <X class="w-6 h-6" />
        </button>

        <!-- Image Number indicator -->
        <div v-if="uiStore.previewImages.length > 1" class="absolute top-6 left-1/2 -translate-x-1/2 px-3 py-1 rounded-full bg-black/50 text-white/80 text-sm font-medium z-[111] backdrop-blur-sm">
          {{ uiStore.previewIndex + 1 }} / {{ uiStore.previewImages.length }}
        </div>

        <!-- Left Arrow -->
        <button v-if="uiStore.previewImages.length > 1" @click.stop="uiStore.prevPreview" class="absolute left-6 top-1/2 -translate-y-1/2 p-3 rounded-full bg-white/10 text-white/70 hover:text-white hover:bg-white/20 transition-colors z-[111]">
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 18-6-6 6-6"/></svg>
        </button>

        <!-- Right Arrow -->
        <button v-if="uiStore.previewImages.length > 1" @click.stop="uiStore.nextPreview" class="absolute right-6 top-1/2 -translate-y-1/2 p-3 rounded-full bg-white/10 text-white/70 hover:text-white hover:bg-white/20 transition-colors z-[111]">
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m9 18 6-6-6-6"/></svg>
        </button>

        <!-- Main Image -->
        <div class="w-full h-full p-4 flex items-center justify-center cursor-default" @click.self="uiStore.closePreview">
          <img :src="uiStore.previewImages[uiStore.previewIndex]" class="max-w-full max-h-full object-contain drop-shadow-2xl select-none" />
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
/* Toast Animations */
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.toast-enter-from {
  opacity: 0;
  transform: translateY(-20px) scale(0.95);
}
.toast-leave-to {
  opacity: 0;
  transform: translateY(-20px) scale(0.95);
}

/* Modal Animations */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>

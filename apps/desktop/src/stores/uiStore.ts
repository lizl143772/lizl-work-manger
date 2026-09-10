import { defineStore } from 'pinia';
import { setPinned, enterMiniMode, exitMiniMode } from '../lib/window';

const PIN_STORAGE_KEY = 'workmanager.alwaysOnTop';

interface Toast {
  id: string;
  message: string;
  type: 'success' | 'error' | 'info';
}

interface ModalState {
  type: 'confirm' | 'prompt';
  title: string;
  message?: string;
  defaultValue?: string;
  resolve: (value: any) => void;
}

export const useUiStore = defineStore('ui', {
  state: () => ({
    toasts: [] as Toast[],
    modal: null as ModalState | null,
    previewImages: [] as string[],
    previewIndex: 0,
    isPreviewOpen: false,
    isPinned: localStorage.getItem(PIN_STORAGE_KEY) === '1',
    isMiniMode: false,
  }),
  actions: {
    async togglePin() {
      this.isPinned = !this.isPinned;
      localStorage.setItem(PIN_STORAGE_KEY, this.isPinned ? '1' : '0');
      await setPinned(this.isPinned);
    },
    async applyPinOnStartup() {
      if (this.isPinned) await setPinned(true);
    },
    async toggleMiniMode() {
      this.isMiniMode = !this.isMiniMode;
      if (this.isMiniMode) {
        await enterMiniMode();
      } else {
        await exitMiniMode(this.isPinned);
      }
    },
    showMessage(msg: string, type: 'success' | 'error' | 'info' = 'info') {
      const id = Math.random().toString(36).substring(2);
      this.toasts.push({ id, message: msg, type });
      setTimeout(() => {
        this.toasts = this.toasts.filter(t => t.id !== id);
      }, 3000);
    },
    async confirm(title: string, message: string = ''): Promise<boolean> {
      return new Promise((resolve) => {
        this.modal = { type: 'confirm', title, message, resolve };
      });
    },
    async prompt(title: string, defaultValue: string = ''): Promise<string | null> {
      return new Promise((resolve) => {
        this.modal = { type: 'prompt', title, defaultValue, resolve };
      });
    },
    closeModal(value?: any) {
      if (this.modal && this.modal.resolve) {
        this.modal.resolve(value ?? null);
      }
      this.modal = null;
    },
    openPreview(images: string[], index: number = 0) {
      if (images.length === 0) return;
      this.previewImages = images;
      this.previewIndex = index;
      this.isPreviewOpen = true;
    },
    closePreview() {
      this.isPreviewOpen = false;
      setTimeout(() => {
        this.previewImages = [];
      }, 200); // Wait for transition
    },
    nextPreview() {
      if (this.previewImages.length > 0) {
        this.previewIndex = (this.previewIndex + 1) % this.previewImages.length;
      }
    },
    prevPreview() {
      if (this.previewImages.length > 0) {
        this.previewIndex = (this.previewIndex - 1 + this.previewImages.length) % this.previewImages.length;
      }
    }
  }
});

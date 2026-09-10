import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';

const win = getCurrentWindow();

const MINI_SIZE = { width: 380, height: 560 };
const MINI_MIN_SIZE = { width: 320, height: 400 };
const MAIN_MIN_SIZE = { width: 800, height: 600 };

let wasMaximized = false;

const guard = async (fn: () => Promise<unknown>) => {
  try {
    await fn();
  } catch {
    // Not running inside Tauri (e.g. plain vite dev in browser)
  }
};

export async function setPinned(pinned: boolean) {
  await guard(() => win.setAlwaysOnTop(pinned));
}

export async function enterMiniMode() {
  await guard(async () => {
    wasMaximized = await win.isMaximized();
    await win.setMinSize(new LogicalSize(MINI_MIN_SIZE.width, MINI_MIN_SIZE.height));
    await win.setSize(new LogicalSize(MINI_SIZE.width, MINI_SIZE.height));
    await win.setAlwaysOnTop(true);
  });
}

export async function exitMiniMode(restorePinned: boolean) {
  await guard(async () => {
    await win.setMinSize(new LogicalSize(MAIN_MIN_SIZE.width, MAIN_MIN_SIZE.height));
    await win.setSize(new LogicalSize(1000, 700));
    await win.setAlwaysOnTop(restorePinned);
    if (wasMaximized) await win.maximize();
  });
}

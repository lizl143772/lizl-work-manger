import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import type { DateRangeKey } from '../types/task';
import { DATE_RANGE_OPTIONS, DEFAULT_DATE_RANGE } from '../types/task';

export type SidebarTab = 'common' | 'menu';
export type CalendarMetric = 'completed' | 'created';

/**
 * 应用设置。
 *
 * 此前这些偏好被拆成 4 个 localStorage 键散落在各组件里（alwaysOnTop / dateRange /
 * sidebarTab / calendarDetailOpen），这里统一收敛到单一的 `workmanager.settings`，
 * 首次读取时会读取旧键做一次性迁移。
 */
export interface AppSettings {
  /** 任务列表默认时间范围 */
  dateRange: DateRangeKey;
  /** 侧边栏默认停留在哪个 Tab */
  sidebarTab: SidebarTab;
  /** 日历默认统计口径 */
  calendarMetric: CalendarMetric;
  /** 日历右侧明细面板是否默认展开 */
  calendarDetailOpen: boolean;
  /** 每周起始日：1 = 周一，0 = 周日 */
  weekStartsOn: 0 | 1;
  /** 保持置顶（含启动时自动置顶） */
  alwaysOnTop: boolean;
  /** 小窗模式窗口尺寸 */
  miniWidth: number;
  miniHeight: number;
  /** 回收站保留天数；0 表示永久保留、不自动清理 */
  trashRetentionDays: number;
}

const SETTINGS_KEY = 'workmanager.settings';

/** 历史遗留的分散键，仅用于一次性迁移后清除 */
const LEGACY_KEYS = {
  dateRange: 'workmanager.dateRange',
  sidebarTab: 'workmanager.sidebarTab',
  calendarDetailOpen: 'workmanager.calendarDetailOpen',
  alwaysOnTop: 'workmanager.alwaysOnTop',
};

export const DEFAULT_SETTINGS: AppSettings = {
  dateRange: DEFAULT_DATE_RANGE,
  sidebarTab: 'common',
  calendarMetric: 'completed',
  calendarDetailOpen: true,
  weekStartsOn: 1,
  alwaysOnTop: false,
  miniWidth: 380,
  miniHeight: 560,
  trashRetentionDays: 30,
};

function readSettings(): AppSettings {
  const raw = localStorage.getItem(SETTINGS_KEY);
  if (raw) {
    try {
      // 与默认值合并，保证后续新增字段不会因为旧数据缺字段而变成 undefined
      return { ...DEFAULT_SETTINGS, ...JSON.parse(raw) };
    } catch {
      // 数据损坏，回退默认值
    }
  }

  const migrated: AppSettings = { ...DEFAULT_SETTINGS };

  const legacyRange = localStorage.getItem(LEGACY_KEYS.dateRange) as DateRangeKey | null;
  if (legacyRange && DATE_RANGE_OPTIONS.some(o => o.key === legacyRange)) {
    migrated.dateRange = legacyRange;
  }
  if (localStorage.getItem(LEGACY_KEYS.sidebarTab) === 'menu') {
    migrated.sidebarTab = 'menu';
  }
  if (localStorage.getItem(LEGACY_KEYS.calendarDetailOpen) === '0') {
    migrated.calendarDetailOpen = false;
  }
  if (localStorage.getItem(LEGACY_KEYS.alwaysOnTop) === '1') {
    migrated.alwaysOnTop = true;
  }

  localStorage.setItem(SETTINGS_KEY, JSON.stringify(migrated));
  Object.values(LEGACY_KEYS).forEach(k => localStorage.removeItem(k));

  return migrated;
}

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<AppSettings>(readSettings());

  // 任何改动都立刻落盘
  watch(
    settings,
    (val) => localStorage.setItem(SETTINGS_KEY, JSON.stringify(val)),
    { deep: true }
  );

  function patch(partial: Partial<AppSettings>) {
    settings.value = { ...settings.value, ...partial };
  }

  function reset() {
    settings.value = { ...DEFAULT_SETTINGS };
  }

  return { settings, patch, reset };
});

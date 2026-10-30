<script setup lang="ts">
import {
  Cpu,
  Fan,
  Gauge,
  Radar,
  SlidersHorizontal,
  Zap,
  type LucideIcon,
} from "lucide-vue-next";
import type { PageId, Snapshot, SystemInfo } from "../types";

defineProps<{
  activePage: PageId;
  snapshot: Snapshot;
  system: SystemInfo;
}>();

const emit = defineEmits<{
  navigate: [page: PageId];
}>();

const navItems: Array<{
  id: PageId;
  label: string;
  code: string;
  icon: LucideIcon;
}> = [
  { id: "overview", label: "系统概览", code: "01", icon: Gauge },
  { id: "power", label: "电源模式", code: "02", icon: Zap },
  { id: "cooling", label: "散热控制", code: "03", icon: Fan },
  { id: "features", label: "设备功能", code: "04", icon: SlidersHorizontal },
  { id: "hardware", label: "硬件信息", code: "05", icon: Cpu },
  { id: "diagnostics", label: "诊断工具", code: "06", icon: Radar },
];
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-logo" aria-label="CC30 Nova 控制中心">
      <svg viewBox="0 0 52 58" aria-hidden="true">
        <path d="M26 3 48 16v26L26 55 4 42V16Z" />
        <path d="m14 19 12-7 12 7-12 7Z" />
        <path d="M14 19v16l12 7V26Z" />
        <path d="M38 19v16l-12 7V26Z" />
      </svg>
      <span>NOVA</span>
    </div>

    <nav class="primary-nav" aria-label="主导航">
      <button
        v-for="(item, index) in navItems"
        :key="item.id"
        class="nav-item"
        :class="{ active: activePage === item.id }"
        :style="{ '--nav-index': index }"
        type="button"
        :aria-label="item.label"
        :data-label="item.label"
        :aria-current="activePage === item.id ? 'page' : undefined"
        @click="emit('navigate', item.id)"
      >
        <span class="nav-code">{{ item.code }}</span>
        <component :is="item.icon" :size="23" stroke-width="1.55" />
        <span class="nav-label">{{ item.label }}</span>
        <span class="nav-pulse" aria-hidden="true"></span>
      </button>
    </nav>

    <div class="sidebar-spacer"></div>

    <div
      class="rail-status"
      :title="`${system.productName || 'CLEVO 笔记本'} · ${snapshot.connected ? 'DCHU 在线' : '未连接'}`"
    >
      <span class="signal-dot" :class="{ offline: !snapshot.connected }"></span>
      <span>{{ snapshot.connected ? "LINK" : "SIM" }}</span>
    </div>
  </aside>
</template>

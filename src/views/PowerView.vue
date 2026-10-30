<script setup lang="ts">
import { computed } from "vue";
import { Check, CircleDot, Info, Leaf, Moon, ScrollText, Sparkles, Zap } from "lucide-vue-next";
import StatusPill from "../components/StatusPill.vue";
import { powerModeById, powerModePosition, powerModesFor } from "../powerModes";
import type { Snapshot } from "../types";

const props = defineProps<{
  snapshot: Snapshot;
  busyMode: number | null;
}>();

const emit = defineEmits<{
  selectMode: [mode: number];
}>();

const currentMode = computed(() => powerModeById(props.snapshot.powerMode));
const powerModeOptions = computed(
  () => props.snapshot.capabilities.powerModeOptions ?? [0, 1, 2, 3],
);
const availableModes = computed(() => powerModesFor(powerModeOptions.value));
const currentPosition = computed(() =>
  powerModePosition(props.snapshot.powerMode, powerModeOptions.value),
);
const icons = { moon: Moon, leaf: Leaf, zap: Zap, sparkles: Sparkles };
</script>

<template>
  <div class="view-stack power-view">
    <section class="view-intro reveal">
      <div>
        <span class="eyebrow">电源配置</span>
        <h2>电源模式</h2>
        <p>选择系统电源配置，并在切换后读取硬件状态确认结果。</p>
      </div>
      <div class="intro-aside">
        <StatusPill :label="snapshot.connected ? '电源服务在线' : '未连接'" />
        <p>电源配置由 CLEVO PowerBiosServer 服务处理。</p>
      </div>
    </section>

    <section class="power-current reveal reveal-delay-1" :style="{ '--mode-color': currentMode.color }">
      <span class="power-current-label">当前配置</span>
      <div class="power-current-number">0{{ currentPosition }}</div>
      <div class="power-current-copy">
        <span>{{ currentMode.code }}</span>
        <h3>{{ currentMode.name }}</h3>
        <p>{{ currentMode.detail }}</p>
      </div>
      <Zap :size="96" stroke-width=".8" />
    </section>

    <section class="mode-card-grid">
      <button
        v-for="(mode, index) in availableModes"
        :key="mode.id"
        class="mode-card reveal"
        :class="{ active: snapshot.powerMode === mode.id, loading: busyMode === mode.id }"
        :style="{
          '--mode-color': mode.color,
          '--mode-accent': mode.accent,
          '--reveal-delay': `${120 + index * 70}ms`,
        }"
        type="button"
        :disabled="busyMode !== null"
        :aria-pressed="snapshot.powerMode === mode.id"
        @click="emit('selectMode', mode.id)"
      >
        <div class="mode-card-top">
          <span class="mode-card-number">0{{ mode.id + 1 }}</span>
          <component :is="icons[mode.icon]" :size="28" stroke-width="1.4" />
        </div>
        <span class="mode-code">{{ mode.code }}</span>
        <h3>{{ mode.name }}</h3>
        <p>{{ mode.summary }}</p>
        <div class="mode-card-footer">
          <span v-if="snapshot.powerMode === mode.id"><Check :size="15" /> 已启用</span>
          <span v-else>应用配置</span>
          <span v-if="busyMode === mode.id" class="mini-loader"></span>
          <CircleDot v-else :size="14" />
        </div>
      </button>
    </section>

    <div class="two-column-notes">
      <section class="panel note-panel reveal reveal-delay-4">
        <div class="icon-box"><ScrollText :size="20" /></div>
        <div>
          <span class="eyebrow">配置切换</span>
          <h3>写入与回读</h3>
          <p>切换配置后，程序会重新读取 PowerBiosServer 状态，确认新模式已生效。</p>
        </div>
      </section>
      <section class="panel note-panel reveal reveal-delay-5">
        <div class="icon-box"><Info :size="20" /></div>
        <div>
          <span class="eyebrow">服务依赖</span>
          <h3>CLEVO 电源服务</h3>
          <p>服务不可用时，配置切换会失败并保留当前设置。</p>
        </div>
      </section>
      <section v-if="availableModes.length === 0" class="panel note-panel">
        <div class="icon-box"><Info :size="20" /></div>
        <div>
          <span class="eyebrow">固件能力</span>
          <h3>未返回可选电源配置</h3>
          <p>当前固件未声明可写入的电源模式，程序不会发送配置请求。</p>
        </div>
      </section>
    </div>
  </div>
</template>

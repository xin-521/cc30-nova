<script setup lang="ts">
import { computed } from "vue";
import { Leaf, Moon, Sparkles, Zap, type LucideIcon } from "lucide-vue-next";
import type { PowerMode } from "../powerModes";
import { powerModes, powerModesFor } from "../powerModes";

const props = withDefaults(
  defineProps<{
    selected: number;
    busy?: number | null;
    variant?: "compact" | "full" | "cockpit";
    options?: number[];
  }>(),
  { busy: null, variant: "compact", options: () => powerModes.map((mode) => mode.id) },
);

const emit = defineEmits<{
  select: [mode: number];
}>();

const icons: Record<string, LucideIcon> = {
  moon: Moon,
  leaf: Leaf,
  zap: Zap,
  sparkles: Sparkles,
};

function iconFor(mode: PowerMode): LucideIcon {
  return icons[mode.icon] ?? Zap;
}

const availableModes = computed(() => powerModesFor(props.options));
</script>

<template>
  <div class="power-rail" :class="`power-rail-${variant}`" role="radiogroup" aria-label="电源模式">
    <button
      v-for="(mode, index) in availableModes"
      :key="mode.id"
      class="power-mode"
      :class="[{ selected: selected === mode.id, busy: busy === mode.id }, `mode-${mode.id}`]"
      type="button"
      role="radio"
      :aria-checked="selected === mode.id"
      :disabled="busy !== null"
      :style="{ '--mode-color': mode.color }"
      @click="emit('select', mode.id)"
    >
      <span class="power-mode-icon">
        <component :is="iconFor(mode)" :size="variant === 'full' ? 25 : 18" stroke-width="1.65" />
      </span>
      <span class="power-mode-copy">
        <span class="power-mode-number">0{{ index + 1 }}</span>
        <strong>{{ mode.name }}</strong>
        <small v-if="variant === 'full'">{{ mode.summary }}</small>
        <small v-else-if="variant === 'cockpit'">{{ mode.code }}</small>
      </span>
      <span class="power-mode-state">
        <span v-if="busy === mode.id" class="mini-loader"></span>
        <span v-else-if="selected === mode.id" class="selected-mark"></span>
      </span>
    </button>
  </div>
</template>

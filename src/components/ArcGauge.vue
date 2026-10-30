<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    label: string;
    value: number;
    max?: number;
    unit: string;
    tone?: "ember" | "ice" | "lime";
    caption?: string;
    digits?: number;
  }>(),
  {
    max: 100,
    tone: "ember",
    caption: "",
    digits: 0,
  },
);

const id = `gauge-${Math.random().toString(36).slice(2)}`;
const ratio = computed(() => Math.max(0, Math.min(1, props.value / props.max)));
const needleAngle = computed(() => -90 + ratio.value * 180);
const shownValue = computed(() =>
  props.value.toLocaleString(undefined, {
    minimumFractionDigits: props.digits,
    maximumFractionDigits: props.digits,
  }),
);
</script>

<template>
  <div class="arc-gauge" :class="`tone-${tone}`" :aria-label="`${label}: ${value} ${unit}`">
    <div class="gauge-topline">
      <span>{{ label }}</span>
      <span class="gauge-index">/ {{ Math.round(ratio * 100) }}</span>
    </div>
    <svg class="gauge-svg" viewBox="0 0 180 122" role="img" aria-hidden="true">
      <defs>
        <linearGradient :id="id" x1="0" y1="0" x2="1" y2="0">
          <stop offset="0%" stop-color="currentColor" stop-opacity=".35" />
          <stop offset="72%" stop-color="currentColor" stop-opacity=".95" />
          <stop offset="100%" stop-color="currentColor" />
        </linearGradient>
        <filter :id="`${id}-glow`">
          <feGaussianBlur stdDeviation="2.4" result="blur" />
          <feMerge>
            <feMergeNode in="blur" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>
      <path
        class="gauge-rail"
        d="M 22 98 A 68 68 0 0 1 158 98"
        pathLength="100"
      />
      <path
        class="gauge-value"
        d="M 22 98 A 68 68 0 0 1 158 98"
        pathLength="100"
        :stroke="`url(#${id})`"
        :style="{ strokeDashoffset: 100 - ratio * 100 }"
        :filter="`url(#${id}-glow)`"
      />
      <g class="gauge-ticks">
        <line
          v-for="tick in 11"
          :key="tick"
          x1="90"
          y1="22"
          x2="90"
          y2="29"
          :transform="`rotate(${-90 + (tick - 1) * 18} 90 98)`"
          :class="{ major: (tick - 1) % 5 === 0 }"
        />
      </g>
      <g class="gauge-needle" :transform="`rotate(${needleAngle} 90 98)`">
        <line x1="90" y1="98" x2="90" y2="45" />
        <circle cx="90" cy="98" r="4.5" />
      </g>
    </svg>
    <div class="gauge-reading">
      <strong>{{ shownValue }}</strong>
      <span>{{ unit }}</span>
    </div>
    <div v-if="caption" class="gauge-caption">{{ caption }}</div>
  </div>
</template>

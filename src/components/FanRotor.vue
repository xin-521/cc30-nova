<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    rpm: number;
    size?: number;
  }>(),
  { size: 64 },
);

const duration = computed(() => {
  if (props.rpm <= 0) return "0s";
  return `${Math.max(0.35, 3.3 - props.rpm / 2500)}s`;
});
</script>

<template>
  <div
    class="fan-rotor"
    :class="{ idle: rpm <= 0 }"
    :style="{ width: `${size}px`, height: `${size}px` }"
    aria-hidden="true"
  >
    <svg viewBox="0 0 100 100">
      <g class="fan-blades" :style="{ animationDuration: duration }">
        <ellipse
          v-for="blade in 7"
          :key="blade"
          cx="50"
          cy="32"
          rx="8"
          ry="28"
          :transform="`rotate(${(blade - 1) * (360 / 7)} 50 50)`"
        />
      </g>
      <circle class="fan-hub" cx="50" cy="50" r="12" />
      <circle class="fan-hub-inner" cx="50" cy="50" r="4" />
    </svg>
  </div>
</template>

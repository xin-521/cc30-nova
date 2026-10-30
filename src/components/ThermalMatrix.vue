<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  cpuTemp: number;
  gpuTemp: number;
  fanDuty: number;
  powerMode: number;
  modeName: string;
}>();

const centerX = 150;
const centerY = 146;
const radius = 92;
const performanceByMode = [0.3, 0.42, 0.72, 0.94];
const quietByMode = [0.94, 0.78, 0.38, 0.2];

function polar(index: number, scale: number) {
  const angle = ((-90 + index * 72) * Math.PI) / 180;
  return {
    x: centerX + Math.cos(angle) * radius * scale,
    y: centerY + Math.sin(angle) * radius * scale,
  };
}

function polygon(scale: number) {
  return Array.from({ length: 5 }, (_, index) => {
    const point = polar(index, scale);
    return `${point.x},${point.y}`;
  }).join(" ");
}

const values = computed(() => [
  Math.max(0.24, Math.min(1, props.cpuTemp / 100)),
  Math.max(0.18, Math.min(1, props.fanDuty / 100)),
  performanceByMode[props.powerMode] ?? 0.72,
  quietByMode[props.powerMode] ?? 0.38,
  Math.max(0.2, Math.min(1, props.gpuTemp / 100)),
]);

const valuePolygon = computed(() =>
  values.value
    .map((value, index) => {
      const point = polar(index, value);
      return `${point.x},${point.y}`;
    })
    .join(" "),
);

const grid = [0.25, 0.5, 0.75, 1].map((scale) => polygon(scale));

const labels = [
  { text: "CPU", ...polar(0, 1.25), align: "middle" },
  { text: "散热", ...polar(1, 1.27), align: "start" },
  { text: "性能", ...polar(2, 1.27), align: "start" },
  { text: "静音", ...polar(3, 1.27), align: "end" },
  { text: "GPU", ...polar(4, 1.25), align: "end" },
];
</script>

<template>
  <div class="thermal-matrix">
    <svg viewBox="0 0 300 300" role="img" aria-label="性能与散热状态">
      <defs>
        <radialGradient id="matrix-fill" cx="50%" cy="48%" r="56%">
          <stop offset="0%" stop-color="#ff5064" stop-opacity=".7" />
          <stop offset="62%" stop-color="#ff3048" stop-opacity=".28" />
          <stop offset="100%" stop-color="#ff3048" stop-opacity=".06" />
        </radialGradient>
        <filter id="matrix-glow" x="-40%" y="-40%" width="180%" height="180%">
          <feGaussianBlur stdDeviation="4" result="blur" />
          <feMerge>
            <feMergeNode in="blur" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      <circle class="matrix-orbit" cx="150" cy="146" r="112" />
      <polygon
        v-for="(points, index) in grid"
        :key="index"
        class="matrix-grid"
        :points="points"
      />
      <line
        v-for="index in 5"
        :key="`axis-${index}`"
        class="matrix-axis"
        x1="150"
        y1="146"
        :x2="polar(index - 1, 1.04).x"
        :y2="polar(index - 1, 1.04).y"
      />
      <polygon
        class="matrix-value"
        :points="valuePolygon"
        fill="url(#matrix-fill)"
        filter="url(#matrix-glow)"
      />
      <circle class="matrix-core" cx="150" cy="146" r="34" />
      <circle class="matrix-core-ring" cx="150" cy="146" r="24" />
      <text class="matrix-mode" x="150" y="143" text-anchor="middle">{{ modeName }}</text>
      <text class="matrix-caption" x="150" y="159" text-anchor="middle">BALANCE</text>
      <text
        v-for="label in labels"
        :key="label.text"
        class="matrix-label"
        :x="label.x"
        :y="label.y"
        :text-anchor="label.align"
        dominant-baseline="middle"
      >
        {{ label.text }}
      </text>
    </svg>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    label: string;
    value: number;
    max?: number;
    unit?: string;
    rpm: number;
    duty: number;
    accent?: string;
    index?: string;
  }>(),
  {
    max: 100,
    unit: "°C",
    accent: "#ff344c",
    index: "01",
  },
);

const center = 140;
const radius = 104;
const arcPath = "M 66.46 213.54 A 104 104 0 1 1 213.54 213.54";
const ratio = computed(() => Math.max(0, Math.min(1, props.value / props.max)));

const ticks = Array.from({ length: 13 }, (_, index) => {
  const angle = ((135 + index * 22.5) * Math.PI) / 180;
  const isMajor = index % 3 === 0;
  const outer = radius + 14;
  const inner = outer - (isMajor ? 10 : 5);

  return {
    x1: center + Math.cos(angle) * inner,
    y1: center + Math.sin(angle) * inner,
    x2: center + Math.cos(angle) * outer,
    y2: center + Math.sin(angle) * outer,
    major: isMajor,
  };
});

const labels = [20, 40, 60, 80, 100].map((value) => {
  const angle = ((135 + (value / 100) * 270) * Math.PI) / 180;
  return {
    value,
    x: center + Math.cos(angle) * 132,
    y: center + Math.sin(angle) * 132,
  };
});
</script>

<template>
  <article
    class="performance-ring"
    :style="{ '--ring-accent': accent }"
    :aria-label="`${label}: ${value}${unit}, ${rpm} RPM, 占空 ${duty}%`"
  >
    <header class="ring-header">
      <span>CHANNEL {{ index }}</span>
      <strong>{{ label }}</strong>
    </header>

    <div class="ring-visual">
      <svg viewBox="0 0 280 280" aria-hidden="true">
        <defs>
          <linearGradient :id="`ring-gradient-${index}`" x1="0" y1="1" x2="1" y2="0">
            <stop offset="0%" stop-color="var(--ring-accent)" stop-opacity=".24" />
            <stop offset="55%" stop-color="var(--ring-accent)" stop-opacity=".88" />
            <stop offset="100%" stop-color="#fff" stop-opacity=".98" />
          </linearGradient>
          <filter :id="`ring-glow-${index}`" x="-30%" y="-30%" width="160%" height="160%">
            <feGaussianBlur stdDeviation="3.2" result="blur" />
            <feMerge>
              <feMergeNode in="blur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
          <radialGradient :id="`ring-core-${index}`">
            <stop offset="0%" stop-color="#18243a" />
            <stop offset="72%" stop-color="#0b1221" />
            <stop offset="100%" stop-color="#070c16" />
          </radialGradient>
        </defs>

        <circle class="ring-outer-shell" cx="140" cy="140" r="126" />
        <circle class="ring-inner-shell" cx="140" cy="140" r="88" />
        <circle
          class="ring-core"
          cx="140"
          cy="140"
          r="73"
          :fill="`url(#ring-core-${index})`"
        />
        <path class="ring-track" :d="arcPath" pathLength="100" />
        <path
          class="ring-value"
          :d="arcPath"
          pathLength="100"
          :stroke="`url(#ring-gradient-${index})`"
          :style="{ strokeDashoffset: 100 - ratio * 100 }"
          :filter="`url(#ring-glow-${index})`"
        />

        <g class="ring-ticks">
          <line
            v-for="(tick, tickIndex) in ticks"
            :key="tickIndex"
            :x1="tick.x1"
            :y1="tick.y1"
            :x2="tick.x2"
            :y2="tick.y2"
            :class="{ major: tick.major }"
          />
        </g>
        <g class="ring-scale">
          <text
            v-for="labelItem in labels"
            :key="labelItem.value"
            :x="labelItem.x"
            :y="labelItem.y"
            text-anchor="middle"
            dominant-baseline="middle"
          >
            {{ labelItem.value }}
          </text>
        </g>

        <circle
          class="ring-orbit-dot"
          :cx="center + Math.cos((135 + ratio * 270) * Math.PI / 180) * radius"
          :cy="center + Math.sin((135 + ratio * 270) * Math.PI / 180) * radius"
          r="4"
        />
      </svg>

      <div class="ring-center">
        <span class="ring-channel">{{ label }}</span>
        <strong>{{ value }}<small>{{ unit }}</small></strong>
        <span class="ring-micro">TEMP SENSOR</span>
      </div>
    </div>

    <footer class="ring-footer">
      <span><i>RPM</i>{{ rpm.toLocaleString() }}</span>
      <span><i>DUTY</i>{{ duty }}%</span>
    </footer>
  </article>
</template>

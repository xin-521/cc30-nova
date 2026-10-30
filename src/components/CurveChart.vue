<script setup lang="ts">
import { computed } from "vue";
import type { FanCurve } from "../types";

const props = defineProps<{
  curves: FanCurve[];
  activeId?: string;
}>();

const chart = { width: 680, height: 252, left: 42, right: 18, top: 18, bottom: 34 };
const colors = ["#ff4055", "#8da8ff", "#56d6c2"];

function x(temp: number) {
  return chart.left + ((temp - 30) / 70) * (chart.width - chart.left - chart.right);
}

function y(duty: number) {
  return chart.top + (1 - duty / 100) * (chart.height - chart.top - chart.bottom);
}

const paths = computed(() =>
  props.curves.map((curve, index) => ({
    ...curve,
    color: colors[index % colors.length],
    points: curve.points
      .map((point) => `${x(point.temperatureC)},${y(point.dutyPercent)}`)
      .join(" "),
    circles: curve.points.map((point) => ({
      ...point,
      cx: x(point.temperatureC),
      cy: y(point.dutyPercent),
    })),
  })),
);

const xTicks = [30, 40, 50, 60, 70, 80, 90, 100];
const yTicks = [0, 25, 50, 75, 100];
</script>

<template>
  <div class="curve-chart">
    <svg :viewBox="`0 0 ${chart.width} ${chart.height}`" role="img" aria-label="风扇曲线">
      <g class="chart-grid">
        <line
          v-for="tick in yTicks"
          :key="`y-${tick}`"
          :x1="chart.left"
          :x2="chart.width - chart.right"
          :y1="y(tick)"
          :y2="y(tick)"
        />
        <line
          v-for="tick in xTicks"
          :key="`x-${tick}`"
          :x1="x(tick)"
          :x2="x(tick)"
          :y1="chart.top"
          :y2="chart.height - chart.bottom"
        />
      </g>
      <g class="chart-labels">
        <text
          v-for="tick in yTicks"
          :key="`yl-${tick}`"
          :x="chart.left - 11"
          :y="y(tick) + 4"
          text-anchor="end"
        >
          {{ tick }}%
        </text>
        <text
          v-for="tick in xTicks"
          :key="`xl-${tick}`"
          :x="x(tick)"
          :y="chart.height - 10"
          text-anchor="middle"
        >
          {{ tick }}°
        </text>
      </g>
      <g v-for="curve in paths" :key="curve.id" class="chart-series">
        <polyline
          :points="curve.points"
          fill="none"
          :stroke="curve.color"
          :class="{ active: !activeId || activeId === curve.id }"
        />
        <circle
          v-for="point in curve.circles"
          :key="`${curve.id}-${point.temperatureC}`"
          :cx="point.cx"
          :cy="point.cy"
          r="4.5"
          :fill="curve.color"
          :class="{ active: !activeId || activeId === curve.id }"
        />
      </g>
    </svg>
    <div class="chart-legend">
      <span v-for="(curve, index) in paths" :key="curve.id">
        <i :style="{ background: colors[index % colors.length] }"></i>
        {{ curve.label }}
      </span>
      <span class="read-only-legend">只读</span>
    </div>
  </div>
</template>

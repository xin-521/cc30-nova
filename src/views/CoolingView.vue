<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  ArrowDownToLine,
  Fan,
  Gauge,
  Info,
  RotateCcw,
  ShieldAlert,
  SlidersHorizontal,
  Thermometer,
} from "lucide-vue-next";
import CurveChart from "../components/CurveChart.vue";
import FanRotor from "../components/FanRotor.vue";
import StatusPill from "../components/StatusPill.vue";
import type { Snapshot } from "../types";

const props = defineProps<{
  snapshot: Snapshot;
  busyFan: boolean;
}>();

const emit = defineEmits<{
  setFanMode: [mode: number];
  setFanOffset: [offset: number];
}>();

const offsetDraft = ref(props.snapshot.fan.offset);
const offsetDirty = ref(false);
const activeCurve = ref<string>("cpu");

watch(
  () => props.snapshot.fan.offset,
  (offset) => {
    if (!offsetDirty.value) offsetDraft.value = offset;
  },
);

const maxRpm = computed(() =>
  Math.max(1, ...props.snapshot.fan.channels.map((channel) => channel.rpm)),
);
const maxTemp = computed(() =>
  Math.max(0, ...props.snapshot.fan.channels.map((channel) => channel.temperatureC)),
);

function updateOffset(event: Event) {
  offsetDirty.value = true;
  offsetDraft.value = Number((event.target as HTMLInputElement).value);
}

function applyOffset() {
  offsetDirty.value = false;
  emit("setFanOffset", offsetDraft.value);
}
</script>

<template>
  <div class="view-stack cooling-view">
    <section class="view-intro reveal">
      <div>
        <span class="eyebrow">散热配置</span>
        <h2>散热控制</h2>
        <p>查看温度和风扇转速，切换散热模式或调整风扇偏移。</p>
      </div>
      <div class="intro-aside">
        <StatusPill :label="snapshot.fan.modeLabel" />
        <p>自动模式使用固件温控策略，最大模式使用当前设备支持的最高风扇响应。</p>
      </div>
    </section>

    <section class="cooling-hero">
      <div
        v-for="channel in snapshot.fan.channels"
        :key="channel.id"
        class="cooling-hero-card reveal"
      >
        <div class="cooling-hero-head">
          <span>{{ channel.label }}</span>
          <Thermometer :size="17" />
        </div>
        <FanRotor :rpm="channel.rpm" :size="94" />
        <strong>{{ channel.rpm.toLocaleString() }}</strong>
        <span class="rpm-label">RPM</span>
        <div class="cooling-hero-stats">
          <span><i>占空</i>{{ channel.dutyPercent }}%</span>
          <span><i>温度</i>{{ channel.temperatureC }}°C</span>
        </div>
      </div>
    </section>

    <div class="cooling-grid">
      <section class="panel cooling-control-panel reveal reveal-delay-2">
        <div class="panel-heading">
          <div>
            <span class="eyebrow">运行模式</span>
            <h3>散热策略</h3>
          </div>
          <Fan :size="20" class="muted-icon" />
        </div>

        <div class="fan-mode-grid">
          <button
            type="button"
            :class="{ selected: snapshot.fan.mode === 0 }"
            :disabled="busyFan"
            @click="emit('setFanMode', 0)"
          >
            <Gauge :size="24" />
            <strong>自动</strong>
            <span>使用固件自动温控策略。</span>
          </button>
          <button
            type="button"
            :class="{ selected: snapshot.fan.mode === 1 }"
            :disabled="busyFan"
            @click="emit('setFanMode', 1)"
          >
            <ArrowDownToLine :size="24" />
            <strong>最大</strong>
            <span>提高风扇转速以应对持续负载。</span>
          </button>
        </div>

        <div class="offset-control">
          <div class="offset-heading">
            <div>
              <span class="eyebrow">自动曲线偏移</span>
              <h4>噪声取向</h4>
            </div>
            <strong>{{ offsetDraft }}%</strong>
          </div>
          <input
            class="range-input"
            type="range"
            min="0"
            max="100"
            step="1"
            :value="offsetDraft"
            aria-label="风扇曲线偏移"
            @input="updateOffset"
          />
          <div class="range-scale"><span>更安静</span><span>更凉爽</span></div>
          <button
            class="primary-action"
            type="button"
            :disabled="busyFan || (!offsetDirty && offsetDraft === snapshot.fan.offset)"
            @click="applyOffset"
          >
            <SlidersHorizontal :size="16" />
            应用偏移
          </button>
        </div>
      </section>

      <section class="panel curve-panel cooling-curve reveal reveal-delay-3">
        <div class="panel-heading">
          <div>
            <span class="eyebrow">当前固件表</span>
            <h3>温度 → 占空比映射</h3>
          </div>
          <div class="curve-tabs">
            <button
              v-for="curve in snapshot.fan.curves"
              :key="curve.id"
              type="button"
              :class="{ selected: activeCurve === curve.id }"
              @click="activeCurve = curve.id"
            >
              {{ curve.label }}
            </button>
            <button type="button" @click="activeCurve = ''">全部</button>
          </div>
        </div>
        <CurveChart :curves="snapshot.fan.curves" :active-id="activeCurve" />
        <div class="curve-summary">
          <span><RotateCcw :size="15" /> 读取自 DCHU WMI 13 / APP 第 4 页</span>
          <span>可用曲线 {{ snapshot.fan.curves.length }} 条</span>
        </div>
      </section>
    </div>

    <div class="cooling-bottom">
      <section class="panel thermal-summary reveal reveal-delay-4">
        <div class="icon-box"><Thermometer :size="20" /></div>
        <div>
          <span class="eyebrow">温度快照</span>
          <h3>峰值 {{ maxTemp }}°C · {{ maxRpm.toLocaleString() }} RPM</h3>
          <p>
            传感器温度与风扇转速均为直接轮询结果。显示 0 RPM 可能表示通道空闲或已停用。
          </p>
        </div>
      </section>
      <section class="panel safety-strip reveal reveal-delay-5">
        <ShieldAlert :size="21" />
        <p>
          <strong>自定义曲线编辑暂未开放。</strong>
          在完成当前机型的散热安全验证前，程序仅支持自动、最大和风扇偏移设置。
        </p>
        <Info :size="16" class="muted-icon" />
      </section>
    </div>
  </div>
</template>

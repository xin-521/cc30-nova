<script setup lang="ts">
import { computed } from "vue";
import {
  ArrowUpRight,
  Fan,
  Gauge,
  HardDrive,
  MemoryStick,
  SlidersHorizontal,
  Thermometer,
  Zap,
} from "lucide-vue-next";
import FanRotor from "../components/FanRotor.vue";
import PerformanceRing from "../components/PerformanceRing.vue";
import PowerModeRail from "../components/PowerModeRail.vue";
import StatusPill from "../components/StatusPill.vue";
import ThermalMatrix from "../components/ThermalMatrix.vue";
import { powerModeById, powerModePosition } from "../powerModes";
import type { PageId, Snapshot, SystemInfo } from "../types";

const props = defineProps<{
  snapshot: Snapshot;
  system: SystemInfo;
  busyMode: number | null;
  busyFan: boolean;
}>();

const emit = defineEmits<{
  selectMode: [mode: number];
  setFanMode: [mode: number];
  navigate: [page: PageId];
}>();

const currentMode = computed(() => powerModeById(props.snapshot.powerMode));
const powerModeOptions = computed(
  () => props.snapshot.capabilities.powerModeOptions ?? [0, 1, 2, 3],
);
const currentModePosition = computed(() =>
  powerModePosition(props.snapshot.powerMode, powerModeOptions.value),
);
const cpu = computed(() => props.snapshot.fan.channels.find((fan) => fan.id === "cpu"));
const gpu = computed(() => props.snapshot.fan.channels.find((fan) => fan.id === "gpu1"));
const cpuTemp = computed(() => cpu.value?.temperatureC ?? 0);
const gpuTemp = computed(() => gpu.value?.temperatureC ?? 0);
const cpuRpm = computed(() => cpu.value?.rpm ?? 0);
const gpuRpm = computed(() => gpu.value?.rpm ?? 0);
const maxFanDuty = computed(() =>
  Math.max(0, ...props.snapshot.fan.channels.map((channel) => channel.dutyPercent)),
);
const maxTemp = computed(() => Math.max(cpuTemp.value, gpuTemp.value));
const capabilityCounts = computed(() => {
  const flags = props.snapshot.capabilities.featureFlags;
  return {
    writable: flags.filter((flag) => flag.access === "writable").length,
    readOnly: flags.filter((flag) => flag.access === "readOnly").length,
    closed: flags.filter(
      (flag) =>
        flag.access === "blocked" ||
        flag.access === "notImplemented" ||
        flag.access === "unsupported",
    ).length,
  };
});
const thermalLabel = computed(() => {
  if (maxTemp.value >= 85) return "高温警戒";
  if (maxTemp.value >= 70) return "负载区间";
  return "热控稳定";
});
</script>

<template>
  <div class="view-stack overview-view">
    <section class="overview-heading reveal">
      <div class="overview-heading-copy">
        <span class="eyebrow">实时状态</span>
        <h2>系统概览</h2>
        <p>温度、转速与电源配置</p>
      </div>
      <div class="cockpit-readout">
        <div>
          <span>热状态</span>
          <strong>{{ thermalLabel }}</strong>
        </div>
        <div>
          <span>当前配置</span>
          <strong>{{ currentMode.name }}</strong>
        </div>
        <StatusPill :label="snapshot.connected ? 'DCHU 在线' : '未连接'" />
      </div>
    </section>

    <section class="cockpit-stage panel reveal reveal-delay-1">
      <header class="cockpit-stage-head">
        <div>
          <span class="live-mark"><i></i> 实时数据</span>
          <strong>温度与风扇状态</strong>
        </div>
        <div class="cockpit-stage-status">
          <span>{{ system.productName || "CLEVO PLATFORM" }}</span>
          <span>BIOS {{ system.biosVersion || "—" }}</span>
        </div>
      </header>

      <div class="cockpit-dials">
        <PerformanceRing
          label="CPU"
          index="01"
          :value="cpuTemp"
          :rpm="cpuRpm"
          :duty="cpu?.dutyPercent ?? 0"
          accent="#ff344c"
        />

        <div class="matrix-center">
          <ThermalMatrix
            :cpu-temp="cpuTemp"
            :gpu-temp="gpuTemp"
            :fan-duty="maxFanDuty"
            :power-mode="snapshot.powerMode"
            :mode-name="currentMode.name"
          />
          <div class="matrix-note">
            <span>风扇配置</span>
            <strong>{{ snapshot.fan.modeLabel }}散热</strong>
            <small>{{ snapshot.fan.offset }}% 噪声偏移</small>
          </div>
        </div>

        <PerformanceRing
          label="GPU"
          index="02"
          :value="gpuTemp"
          :rpm="gpuRpm"
          :duty="gpu?.dutyPercent ?? 0"
          accent="#8da8ff"
        />
      </div>

      <div class="cockpit-mode-deck">
        <div class="mode-deck-copy">
          <span class="eyebrow">当前电源模式</span>
          <strong><b>0{{ currentModePosition }}</b>{{ currentMode.name }}</strong>
          <p>{{ snapshot.fan.modeLabel }}散热 · {{ snapshot.fan.offset }}% 偏移</p>
        </div>
        <PowerModeRail
          :selected="snapshot.powerMode"
          :busy="busyMode"
          :options="powerModeOptions"
          variant="cockpit"
          @select="emit('selectMode', $event)"
        />
      </div>
    </section>

    <div class="overview-support-grid">
      <section class="panel cockpit-device reveal reveal-delay-2">
        <div class="panel-heading compact">
          <div>
            <span class="eyebrow">设备档案</span>
            <h3>{{ system.productName || "CLEVO 笔记本" }}</h3>
          </div>
          <HardDrive :size="19" class="muted-icon" />
        </div>

        <div class="identity-hero">
          <span class="identity-monogram">{{ system.manufacturer.slice(0, 3) || "CLE" }}</span>
          <div>
            <span>{{ system.manufacturer }}</span>
            <strong>{{ system.boardProduct }}</strong>
            <small>BIOS {{ system.biosVersion }}</small>
          </div>
        </div>

        <dl class="spec-list">
          <div>
            <dt>处理器</dt>
            <dd>{{ system.cpuName }}</dd>
          </div>
          <div>
            <dt>内存</dt>
            <dd>{{ system.memoryTotalGb }} GB</dd>
          </div>
          <div>
            <dt>操作系统</dt>
            <dd>{{ system.osName }}</dd>
          </div>
        </dl>

        <button class="text-action" type="button" @click="emit('navigate', 'hardware')">
          查看硬件详情 <ArrowUpRight :size="15" />
        </button>
      </section>

      <section class="panel cockpit-cooling reveal reveal-delay-3">
        <div class="panel-heading compact">
          <div>
            <span class="eyebrow">散热阵列</span>
            <h3>{{ snapshot.fan.modeLabel }}策略</h3>
          </div>
          <Fan :size="19" class="muted-icon" />
        </div>

        <div class="cooling-channel-list">
          <div
            v-for="channel in snapshot.fan.channels"
            :key="channel.id"
            class="cooling-channel"
          >
            <FanRotor :rpm="channel.rpm" :size="48" />
            <div>
              <span>{{ channel.label }}</span>
              <strong>{{ channel.rpm.toLocaleString() }} <small>RPM</small></strong>
            </div>
            <span class="temperature-chip">{{ channel.temperatureC }}°</span>
          </div>
        </div>

        <div class="segmented-control" aria-label="风扇模式">
          <button
            type="button"
            :class="{ selected: snapshot.fan.mode === 0 }"
            :disabled="busyFan || !snapshot.capabilities.fanControl"
            @click="emit('setFanMode', 0)"
          >
            自动
          </button>
          <button
            type="button"
            :class="{ selected: snapshot.fan.mode === 1 }"
            :disabled="busyFan || !snapshot.capabilities.fanControl"
            @click="emit('setFanMode', 1)"
          >
            最大
          </button>
        </div>
      </section>

      <section class="panel cockpit-safety reveal reveal-delay-4">
        <div class="safety-emblem">
          <SlidersHorizontal :size="28" stroke-width="1.4" />
        </div>
        <span class="eyebrow">固件能力</span>
        <h3>{{ capabilityCounts.writable }} 项可设置</h3>
        <p>{{ capabilityCounts.readOnly }} 项只读 · {{ capabilityCounts.closed }} 项未开放</p>
        <div class="safety-tags">
          <span>写入回读校验</span>
          <span>能力实时探测</span>
          <span>DCHU {{ snapshot.connected ? "在线" : "未连接" }}</span>
        </div>
        <button class="text-action capability-link" type="button" @click="emit('navigate', 'features')">
          查看设备功能 <ArrowUpRight :size="15" />
        </button>
      </section>
    </div>

    <section class="panel vitals-panel reveal reveal-delay-5">
      <div class="vital">
        <MemoryStick :size="18" />
        <span>内存</span>
        <strong>{{ system.memoryTotalGb }} <small>GB</small></strong>
      </div>
      <div class="vital">
        <Zap :size="18" />
        <span>电源配置</span>
        <strong>{{ currentModePosition }}/{{ powerModeOptions.length }}</strong>
      </div>
      <div class="vital">
        <Thermometer :size="18" />
        <span>峰值温度</span>
        <strong>{{ maxTemp }} <small>°C</small></strong>
      </div>
      <div class="vital">
        <Gauge :size="18" />
        <span>EC 版本</span>
        <strong>{{ snapshot.device.ecVersion || "—" }}</strong>
      </div>
    </section>

    <p v-if="snapshot.notice" class="inline-notice">{{ snapshot.notice }}</p>
  </div>
</template>

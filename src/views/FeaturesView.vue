<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  Activity,
  BatteryCharging,
  Cpu,
  Gauge,
  Keyboard,
  LockKeyhole,
  Monitor,
  MonitorCog,
  PlugZap,
  ShieldCheck,
  SlidersHorizontal,
  Wind,
} from "lucide-vue-next";
import type { FeatureFlag, Snapshot, SystemInfo } from "../types";

const props = defineProps<{
  snapshot: Snapshot;
  system: SystemInfo;
  busyFeature: string | null;
}>();

const emit = defineEmits<{
  setKeyboard: [fnLock: boolean, windowsKeyEnabled: boolean, winFnSwap: boolean];
  setCharging: [enabled: boolean, startPercent: number, stopPercent: number];
  setLowRefresh: [enabled: boolean];
  setBlueLight: [factor: number];
}>();

const featureGroups = computed(() => {
  const flags = props.snapshot.capabilities.featureFlags;
  return {
    active: flags.filter((flag) => flag.access === "writable" || flag.access === "readOnly"),
    pending: flags.filter((flag) => flag.access === "notImplemented"),
    blocked: flags.filter((flag) => flag.access === "blocked"),
    unsupported: flags.filter((flag) => flag.access === "unsupported"),
  };
});

const batteryAge = computed(() => {
  const battery = props.system.battery;
  if (!battery.present) return "未检测";
  if (battery.healthPercent >= 85) return "健康";
  if (battery.healthPercent >= 70) return "可用";
  return "建议检测";
});

const batteryRuntime = computed(() => {
  const runtime = props.snapshot.device.batteryRuntime;
  if (!runtime.present) return "未检测";
  if (runtime.charging) return `充电中 · ${runtime.chargePercent}%`;
  if (runtime.powerOnline) return `已接通电源 · ${runtime.chargePercent}%`;
  return `电池供电 · ${runtime.chargePercent}%`;
});

const batteryFirmwareStatus = computed(() => {
  const firmware = props.snapshot.device.batteryFirmware;
  if (!firmware.supported) return "固件未响应";
  if (firmware.permanentFailure) return "永久故障标记";
  if (firmware.refreshChargePhase) return "校准充电阶段";
  if (firmware.refreshDischargePhase) return "校准放电阶段";
  return "状态正常";
});

const antiDustSchedule = computed(() => {
  const antiDust = props.snapshot.device.antiDust;
  if (!antiDust.supported) return "固件未声明";
  if (!antiDust.schedulerEnabled) return "计划未启用";
  const days = antiDust.dayMask === 0x7f ? "每日" : `掩码 ${antiDust.dayMask}`;
  return `${days} · ${antiDust.hour.toString().padStart(2, "0")}:00`;
});

const blueLightLabel = computed(() => {
  const policy = props.snapshot.device.systemPolicy;
  if (!policy.blueLightSupported) return "固件未声明";
  if (policy.blueLightFactor <= 1.01) return "标准 · 1.0×";
  return `滤镜 · ${policy.blueLightFactor.toFixed(1)}×`;
});

const pageUpDownLabel = computed(() => {
  const policy = props.snapshot.device.systemPolicy;
  if (!policy.pageUpDownSupported) return "固件未声明";
  return policy.pageUpDownEnabled ? "启用" : "禁用";
});

const keyboardLightingLabel = computed(() => {
  const capabilities = props.snapshot.capabilities;
  if (!capabilities.keyboardRgb) return "不支持";
  return capabilities.perKeyKeyboard ? "单键灯效" : "通用灯效";
});

const chargingStart = ref(0);
const chargingStop = ref(0);
const chargingDirty = ref(false);

watch(
  () => [
    props.snapshot.device.charging.startPercent,
    props.snapshot.device.charging.stopPercent,
  ],
  ([start, stop]) => {
    if (chargingDirty.value) return;
    chargingStart.value = start;
    chargingStop.value = stop;
  },
  { immediate: true },
);

function toggleKeyboard(
  field: "fnLock" | "windowsKeyEnabled" | "winFnSwap",
) {
  const keyboard = props.snapshot.device.keyboard;
  emit(
    "setKeyboard",
    field === "fnLock" ? !keyboard.fnLock : keyboard.fnLock,
    field === "windowsKeyEnabled" ? !keyboard.windowsKeyEnabled : keyboard.windowsKeyEnabled,
    field === "winFnSwap" ? !keyboard.winFnSwap : keyboard.winFnSwap,
  );
}

function updateChargingStart(event: Event) {
  chargingStart.value = Number((event.target as HTMLSelectElement).value);
  const options = props.snapshot.device.charging.stopOptions;
  if (chargingStart.value >= chargingStop.value) {
    chargingStop.value = options.find((value) => value > chargingStart.value) ?? chargingStop.value;
  }
  chargingDirty.value = true;
}

function updateChargingStop(event: Event) {
  chargingStop.value = Number((event.target as HTMLSelectElement).value);
  const options = props.snapshot.device.charging.startOptions;
  if (chargingStart.value >= chargingStop.value) {
    chargingStart.value =
      [...options].reverse().find((value) => value < chargingStop.value) ?? chargingStart.value;
  }
  chargingDirty.value = true;
}

function applyCharging() {
  chargingDirty.value = false;
  emit(
    "setCharging",
    props.snapshot.device.charging.enabled,
    chargingStart.value,
    chargingStop.value,
  );
}

function toggleCharging() {
  emit(
    "setCharging",
    !props.snapshot.device.charging.enabled,
    chargingStart.value || props.snapshot.device.charging.startPercent,
    chargingStop.value || props.snapshot.device.charging.stopPercent,
  );
}

function commitBlueLight(event: Event) {
  emit("setBlueLight", Number((event.target as HTMLInputElement).value) / 10);
}

function accessLabel(access: FeatureFlag["access"]): string {
  const labels: Record<FeatureFlag["access"], string> = {
    writable: "可设置",
    readOnly: "只读",
    notImplemented: "待接入",
    blocked: "安全禁用",
    unsupported: "不支持",
  };
  return labels[access];
}

function mwh(value: number): string {
  if (!value) return "—";
  return `${(value / 1000).toFixed(1)} Wh`;
}
</script>

<template>
  <div class="view-stack features-view">
    <section class="view-intro compact-intro reveal">
      <div>
        <span class="eyebrow">DCHU 能力映射</span>
        <h2>设备功能</h2>
      </div>
      <div class="feature-version-readout">
        <span><i>EC</i>{{ snapshot.device.ecVersion || "未响应" }}</span>
        <span><i>BIOS PKG</i>{{ snapshot.capabilities.featurePackageVersion }}</span>
        <span><i>FAN CH</i>{{ snapshot.capabilities.fanCount }}</span>
      </div>
    </section>

    <section class="feature-summary-strip reveal reveal-delay-1">
      <div>
        <span>已接入</span>
        <strong>{{ featureGroups.active.length }}</strong>
      </div>
      <div>
        <span>待验证</span>
        <strong>{{ featureGroups.pending.length }}</strong>
      </div>
      <div>
        <span>安全禁用</span>
        <strong>{{ featureGroups.blocked.length }}</strong>
      </div>
      <div>
        <span>不支持</span>
        <strong>{{ featureGroups.unsupported.length }}</strong>
      </div>
      <div>
        <span>风扇通道</span>
        <strong>{{ snapshot.capabilities.fanCount }}</strong>
      </div>
    </section>

    <section class="panel feature-matrix-panel reveal reveal-delay-2">
      <div class="panel-heading">
        <div>
          <span class="eyebrow">固件声明与产品接入状态</span>
          <h3>功能清单</h3>
        </div>
        <ShieldCheck :size="19" class="muted-icon" />
      </div>
      <div class="feature-matrix">
        <article
          v-for="flag in snapshot.capabilities.featureFlags"
          :key="flag.id"
          class="feature-cell"
          :class="`feature-${flag.access}`"
        >
          <div class="feature-cell-head">
            <span>{{ flag.label }}</span>
            <strong>{{ accessLabel(flag.access) }}</strong>
          </div>
          <p>{{ flag.detail }}</p>
          <i aria-hidden="true"></i>
        </article>
      </div>
    </section>

    <div class="device-config-grid policy-status-grid">
      <section class="panel device-config-card reveal reveal-delay-2">
        <div class="panel-heading compact">
          <div>
            <span class="eyebrow">DISPLAY & INPUT POLICY</span>
            <h3>显示与按键策略</h3>
          </div>
          <Monitor :size="20" class="muted-icon" />
        </div>
        <dl class="readonly-setting-list">
          <div>
            <dt>低刷新率</dt>
            <dd>
              <button
                class="policy-switch"
                type="button"
                role="switch"
                :aria-checked="snapshot.device.systemPolicy.lowRefreshRateEnabled"
                :disabled="
                  !snapshot.device.systemPolicy.lowRefreshRateSupported ||
                  busyFeature === 'display' ||
                  !snapshot.connected
                "
                @click="
                  emit(
                    'setLowRefresh',
                    !snapshot.device.systemPolicy.lowRefreshRateEnabled,
                  )
                "
              >
                <span></span>
                {{
                  snapshot.device.systemPolicy.lowRefreshRateSupported
                    ? snapshot.device.systemPolicy.lowRefreshRateEnabled
                      ? "已启用"
                      : "关闭"
                    : "不支持"
                }}
              </button>
            </dd>
          </div>
          <div>
            <dt>蓝光过滤</dt>
            <dd>{{ blueLightLabel }}</dd>
          </div>
          <div>
            <dt>PgUp / PgDn</dt>
            <dd
              :class="{
                enabled:
                  snapshot.device.systemPolicy.pageUpDownSupported &&
                  snapshot.device.systemPolicy.pageUpDownEnabled,
              }"
            >
              {{ pageUpDownLabel }}
            </dd>
          </div>
          <div>
            <dt>电源模式锁定</dt>
            <dd :class="{ enabled: snapshot.device.systemPolicy.powerModeLocked }">
              {{ snapshot.device.systemPolicy.powerModeLocked ? "固件已锁定" : "未锁定" }}
            </dd>
          </div>
        </dl>
        <div class="range-control">
          <div class="range-control-head">
            <span>蓝光强度</span>
            <strong>{{ snapshot.device.systemPolicy.blueLightFactor.toFixed(1) }}×</strong>
          </div>
          <input
            type="range"
            min="10"
            max="20"
            step="1"
            :value="Math.round(snapshot.device.systemPolicy.blueLightFactor * 10)"
            :disabled="
              !snapshot.device.systemPolicy.blueLightSupported ||
              busyFeature === 'display' ||
              !snapshot.connected
            "
            aria-label="蓝光过滤强度"
            @change="commitBlueLight"
          />
          <div class="range-scale"><span>1.0×</span><span>标准</span><span>2.0×</span></div>
        </div>
        <p class="device-note">
          <ShieldCheck :size="13" />
          低刷新率与蓝光强度均会持久化并回读确认；蓝光调整使用系统显示伽马通道。
        </p>
      </section>

      <section class="panel device-config-card reveal reveal-delay-3">
        <div class="panel-heading compact">
          <div>
            <span class="eyebrow">THERMAL POLICY TELEMETRY</span>
            <h3>热策略遥测</h3>
          </div>
          <Activity :size="20" class="muted-icon" />
        </div>
        <dl class="readonly-setting-list">
          <div>
            <dt>动态散热 DTT</dt>
            <dd
              :class="{
                enabled:
                  snapshot.capabilities.dtt && snapshot.device.systemPolicy.dttEnabled,
              }"
            >
              {{
                snapshot.capabilities.dtt
                  ? snapshot.device.systemPolicy.dttEnabled
                    ? "已启用"
                    : "关闭"
                  : "不支持"
              }}
            </dd>
          </div>
          <div>
            <dt>Turbo Fan</dt>
            <dd
              :class="{
                enabled:
                  snapshot.capabilities.turboFan &&
                  snapshot.device.systemPolicy.turboFanActive,
              }"
            >
              {{
                snapshot.capabilities.turboFan
                  ? snapshot.device.systemPolicy.turboFanActive
                    ? "运行中"
                    : "待机"
                  : "不支持"
              }}
            </dd>
          </div>
          <div>
            <dt>Slow Fan</dt>
            <dd
              :class="{
                enabled:
                  snapshot.capabilities.slowFan &&
                  snapshot.device.systemPolicy.slowFanActive,
              }"
            >
              {{
                snapshot.capabilities.slowFan
                  ? snapshot.device.systemPolicy.slowFanActive
                    ? "运行中"
                    : "待机"
                  : "不支持"
              }}
            </dd>
          </div>
          <div>
            <dt>当前风扇偏移</dt>
            <dd>{{ snapshot.fan.offset }}%</dd>
          </div>
        </dl>
        <p class="device-note">
          <LockKeyhole :size="13" />
          仅显示固件运行标记；DTT、Turbo Fan 与 Slow Fan 策略写入保持关闭。
        </p>
      </section>
    </div>

    <div class="device-config-grid">
      <section class="panel device-config-card reveal reveal-delay-2">
        <div class="panel-heading compact">
          <div>
            <span class="eyebrow">GPU SWITCH</span>
            <h3>显卡工作模式</h3>
          </div>
          <MonitorCog :size="20" class="muted-icon" />
        </div>
        <div class="device-primary-value">
          <span>当前模式</span>
          <strong>{{ snapshot.device.gpuMode.currentLabel }}</strong>
          <small>{{ snapshot.device.gpuMode.supported ? "固件应答有效" : "固件未声明" }}</small>
        </div>
        <div class="readonly-option-row">
          <span
            v-for="option in snapshot.device.gpuMode.options"
            :key="option.value"
            :class="{ active: option.value === snapshot.device.gpuMode.current }"
          >
            {{ option.label }}
          </span>
        </div>
        <p class="device-note">
          <LockKeyhole :size="13" />
          切换 GPU 模式需要重启，当前版本仅显示状态。
        </p>
      </section>

      <section class="panel device-config-card reveal reveal-delay-3">
        <div class="panel-heading compact">
          <div>
            <span class="eyebrow">KEYBOARD CONTROL</span>
            <h3>键盘功能</h3>
          </div>
          <Keyboard :size="20" class="muted-icon" />
        </div>
        <div class="setting-toggle-list">
          <div>
            <span>FnLock</span>
            <button
              class="policy-switch"
              type="button"
              role="switch"
              :aria-checked="snapshot.device.keyboard.fnLock"
              :disabled="
                !snapshot.device.keyboard.fnLockSupported ||
                busyFeature === 'keyboard' ||
                !snapshot.connected
              "
              @click="toggleKeyboard('fnLock')"
            >
              <span></span>
              {{
                snapshot.device.keyboard.fnLockSupported
                  ? snapshot.device.keyboard.fnLock
                    ? "开启"
                    : "关闭"
                  : "不支持"
              }}
            </button>
          </div>
          <div>
            <span>Windows 键</span>
            <button
              class="policy-switch"
              type="button"
              role="switch"
              :aria-checked="snapshot.device.keyboard.windowsKeyEnabled"
              :disabled="
                !snapshot.device.keyboard.windowsKeySupported ||
                busyFeature === 'keyboard' ||
                !snapshot.connected
              "
              @click="toggleKeyboard('windowsKeyEnabled')"
            >
              <span></span>
              {{
                snapshot.device.keyboard.windowsKeySupported
                  ? snapshot.device.keyboard.windowsKeyEnabled
                    ? "可用"
                    : "禁用"
                  : "不支持"
              }}
            </button>
          </div>
          <div>
            <span>Win / Fn 交换</span>
            <button
              class="policy-switch"
              type="button"
              role="switch"
              :aria-checked="snapshot.device.keyboard.winFnSwap"
              :disabled="
                !snapshot.device.keyboard.winFnSupported ||
                busyFeature === 'keyboard' ||
                !snapshot.connected
              "
              @click="toggleKeyboard('winFnSwap')"
            >
              <span></span>
              {{
                snapshot.device.keyboard.winFnSupported
                  ? snapshot.device.keyboard.winFnSwap
                    ? "交换"
                    : "标准"
                  : "不支持"
              }}
            </button>
          </div>
          <div>
            <span>键盘灯效</span>
            <strong>{{ keyboardLightingLabel }}</strong>
          </div>
        </div>
        <p class="device-note">
          <ShieldCheck :size="13" />
          功能键设置经 DCHU 写入固件后回读确认；灯效控制仍保持关闭。
        </p>
      </section>

      <section class="panel device-config-card reveal reveal-delay-3">
        <div class="panel-heading compact">
          <div>
            <span class="eyebrow">FLEXICHARGER</span>
            <h3>充电阈值</h3>
          </div>
          <PlugZap :size="20" class="muted-icon" />
        </div>
        <div class="charging-toolbar">
          <button
            class="policy-switch"
            type="button"
            role="switch"
            :aria-checked="snapshot.device.charging.enabled"
            :disabled="
              !snapshot.device.charging.supported ||
              busyFeature === 'charging' ||
              !snapshot.connected
            "
            @click="toggleCharging"
          >
            <span></span>
            {{ snapshot.device.charging.supported ? (snapshot.device.charging.enabled ? "已启用" : "已关闭") : "不支持" }}
          </button>
          <span>充电区间</span>
        </div>
        <div class="charge-selectors">
          <label>
            <span>开始充电</span>
            <select
              :value="chargingStart"
              :disabled="!snapshot.device.charging.supported || busyFeature === 'charging'"
              @change="updateChargingStart"
            >
              <option
                v-for="value in snapshot.device.charging.startOptions"
                :key="value"
                :value="value"
              >
                {{ value }}%
              </option>
            </select>
          </label>
          <label>
            <span>停止充电</span>
            <select
              :value="chargingStop"
              :disabled="!snapshot.device.charging.supported || busyFeature === 'charging'"
              @change="updateChargingStop"
            >
              <option
                v-for="value in snapshot.device.charging.stopOptions"
                :key="value"
                :value="value"
              >
                {{ value }}%
              </option>
            </select>
          </label>
        </div>
        <div class="charge-track" aria-hidden="true">
          <span
            :style="{
              left: `${chargingStart}%`,
              width: `${Math.max(0, chargingStop - chargingStart)}%`,
            }"
          ></span>
        </div>
        <button
          class="feature-apply-button"
          type="button"
          :disabled="
            !snapshot.device.charging.supported ||
            chargingStart >= chargingStop ||
            busyFeature === 'charging' ||
            !snapshot.connected
          "
          @click="applyCharging"
        >
          <SlidersHorizontal :size="14" />
          {{ busyFeature === "charging" ? "正在写入…" : "应用充电阈值" }}
        </button>
        <p class="device-note">
          <ShieldCheck :size="13" />
          阈值写入受机型选项约束；关闭 FlexiCharger 后恢复固定满充策略。
        </p>
      </section>

      <section class="panel device-config-card reveal reveal-delay-4">
        <div class="panel-heading compact">
          <div>
            <span class="eyebrow">ANTI-DUST</span>
            <h3>风扇除尘</h3>
          </div>
          <Wind :size="20" class="muted-icon" />
        </div>
        <dl class="readonly-setting-list">
          <div>
            <dt>固件能力</dt>
            <dd :class="{ enabled: snapshot.device.antiDust.supported }">
              {{ snapshot.device.antiDust.supported ? "已声明" : "不支持" }}
            </dd>
          </div>
          <div>
            <dt>运行状态</dt>
            <dd :class="{ enabled: snapshot.device.antiDust.running }">
              {{ snapshot.device.antiDust.running ? "除尘中" : "待机" }}
            </dd>
          </div>
          <div>
            <dt>除尘计划</dt>
            <dd>{{ antiDustSchedule }}</dd>
          </div>
          <div>
            <dt>开机恢复</dt>
            <dd :class="{ enabled: snapshot.device.antiDust.resumeAfterBoot }">
              {{ snapshot.device.antiDust.resumeAfterBoot ? "开启" : "关闭" }}
            </dd>
          </div>
        </dl>
        <p class="device-note">
          <LockKeyhole :size="13" />
          当前版本仅读取状态；除尘启停需要独立散热与噪音验证。
        </p>
      </section>
    </div>

    <section class="panel battery-health-panel reveal reveal-delay-4">
      <div class="battery-health-main">
        <div
          class="battery-health-ring"
          :style="{ '--health': `${system.battery.healthPercent || 0}%` }"
        >
          <BatteryCharging :size="21" />
          <strong>{{ system.battery.healthPercent || "—" }}<small>%</small></strong>
          <span>{{ batteryAge }}</span>
        </div>
        <div class="battery-health-copy">
          <span class="eyebrow">WINDOWS BATTERY STATIC DATA</span>
          <h3>{{ system.battery.model || "系统电池" }}</h3>
          <p>{{ batteryRuntime }}</p>
        </div>
      </div>
      <div class="battery-facts">
        <div>
          <span>设计容量</span>
          <strong>{{ mwh(system.battery.designedCapacityMwh) }}</strong>
        </div>
        <div>
          <span>满充容量</span>
          <strong>{{ mwh(system.battery.fullChargedCapacityMwh) }}</strong>
        </div>
        <div>
          <span>循环次数</span>
          <strong>{{ system.battery.present ? system.battery.cycleCount : "—" }}</strong>
        </div>
        <div>
          <span>电池健康</span>
          <strong>{{ system.battery.present ? `${system.battery.healthPercent}%` : "—" }}</strong>
        </div>
      </div>
      <p class="battery-firmware-line">
        <span>DCHU WMI 7</span>
        <strong>{{ batteryFirmwareStatus }}</strong>
        <span>
          制造日期
          {{ snapshot.device.batteryFirmware.manufacturedOn || "未读取" }}
        </span>
        <span>
          状态字 0x{{
            snapshot.device.batteryFirmware.batteryStatus.toString(16).toUpperCase().padStart(4, "0")
          }}
        </span>
      </p>
    </section>

    <div class="feature-footnotes">
      <span><Cpu :size="14" /> EC {{ snapshot.device.ecVersion || "未响应" }}</span>
      <span><Gauge :size="14" /> 能力字 0x{{ snapshot.capabilities.featureRaw.toString(16).toUpperCase() }}</span>
      <span><LockKeyhole :size="14" /> 读取不代表已开放写入</span>
    </div>
  </div>
</template>

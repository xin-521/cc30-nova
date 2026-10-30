<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import {
  CheckCircle2,
  Clock3,
  Cpu,
  RefreshCw,
  Terminal,
  Wifi,
  WifiOff,
  X,
} from "lucide-vue-next";
import {
  getDiagnostics,
  getSnapshot,
  getSystemInfo,
  isTauriRuntime,
  setBlueLightFactor as writeBlueLightFactor,
  setChargingThreshold as writeChargingThreshold,
  setFanMode as writeFanMode,
  setFanOffset as writeFanOffset,
  setKeyboardSettings as writeKeyboardSettings,
  setLowRefreshRate as writeLowRefreshRate,
  setPowerMode as writePowerMode,
} from "./api";
import Sidebar from "./components/Sidebar.vue";
import TitleBar from "./components/TitleBar.vue";
import CoolingView from "./views/CoolingView.vue";
import DiagnosticsView from "./views/DiagnosticsView.vue";
import FeaturesView from "./views/FeaturesView.vue";
import HardwareView from "./views/HardwareView.vue";
import OverviewView from "./views/OverviewView.vue";
import PowerView from "./views/PowerView.vue";
import type { Diagnostics, PageId, Snapshot, SystemInfo } from "./types";

const pageMeta: Record<PageId, { title: string; index: string }> = {
  overview: { title: "系统概览", index: "01" },
  power: { title: "电源模式", index: "02" },
  cooling: { title: "散热控制", index: "03" },
  features: { title: "设备功能", index: "04" },
  hardware: { title: "硬件信息", index: "05" },
  diagnostics: { title: "诊断工具", index: "06" },
};

const navOrder: PageId[] = [
  "overview",
  "power",
  "cooling",
  "features",
  "hardware",
  "diagnostics",
];

const emptySystem: SystemInfo = {
  manufacturer: "",
  productName: "",
  serialNumber: "",
  family: "",
  biosVendor: "",
  biosVersion: "",
  biosDate: "",
  boardManufacturer: "",
  boardProduct: "",
  boardVersion: "",
  cpuName: "正在识别处理器",
  cpuCores: 0,
  cpuThreads: 0,
  memoryTotalGb: 0,
  memoryModules: [],
  osName: "Windows",
  osBuild: "",
  hostname: "",
  gpuNames: [],
  battery: {
    present: false,
    model: "",
    manufacturer: "",
    designedCapacityMwh: 0,
    fullChargedCapacityMwh: 0,
    cycleCount: 0,
    healthPercent: 0,
  },
};

const emptyDiagnostics: Diagnostics = {
  dllPresent: false,
  apiAvailable: false,
  outLogAvailable: false,
  powerBiosLogAvailable: false,
  featureRaw: 0,
  wmi16: 0,
  wmi96: 0,
  wmi122: 0,
  nativeApis: [],
  checks: [],
};

const emptySnapshot: Snapshot = {
  connected: false,
  source: "simulation",
  powerMode: 2,
  powerModeLabel: "性能",
  fan: {
    mode: 0,
    modeLabel: "自动",
    offset: 0,
    fanCount: 0,
    customCurveSupported: false,
    channels: [],
    curves: [],
  },
  capabilities: {
    featureRaw: 0,
    featurePackageVersion: "未响应",
    featureFlags: [],
    powerModes: false,
    powerModeOptions: [],
    fanControl: false,
    keyboardRgb: false,
    rgbZones: 0,
    perKeyKeyboard: false,
    keyboardControllerType: 0,
    cpuOverclock: false,
    gpuOverclock: false,
    flexikey: false,
    fanCount: 0,
    customFanCurves: false,
    turboFan: false,
    slowFan: false,
    maxQFan: false,
    silentFan: false,
    antiDust: false,
    dtt: false,
    fanOffsetSupported: false,
    lowRefreshRate: false,
    blueLightFilter: false,
    gpuSwitch: false,
    keyboardSettings: false,
    flexiCharger: false,
    batteryUtility: false,
    energySave: false,
    flexiAccess: false,
    pageUpDownKey: false,
    controlCenterFunction: false,
    amdManualMode: false,
  },
  device: {
    ecVersion: undefined,
    gpuMode: {
      supported: false,
      current: 0,
      currentLabel: "未知",
      options: [],
      restartRequired: true,
    },
    keyboard: {
      supported: false,
      controllerType: 0,
      fnLock: false,
      fnLockSupported: false,
      windowsKeyEnabled: false,
      windowsKeySupported: false,
      winFnSwap: false,
      winFnSupported: false,
    },
    charging: {
      supported: false,
      enabled: false,
      startPercent: 0,
      stopPercent: 0,
      startOptions: [],
      stopOptions: [],
    },
    batteryRuntime: {
      present: false,
      powerOnline: false,
      charging: false,
      discharging: false,
      chargePercent: 0,
    },
    batteryFirmware: {
      supported: false,
      version: 0,
      cycleCount: 0,
      fullChargeCapacityMwh: 0,
      designCapacityMwh: 0,
      healthPercent: 0,
      batteryStatus: 0,
      pfStatus: 0,
      operationStatus: 0,
      stopChargingThreshold: 0,
      refreshChargePhase: false,
      refreshDischargePhase: false,
      permanentFailure: false,
    },
    antiDust: {
      supported: false,
      running: false,
      schedulerEnabled: false,
      dayMask: 0,
      hour: 0,
      resumeAfterBoot: false,
    },
    systemPolicy: {
      lowRefreshRateSupported: false,
      lowRefreshRateEnabled: false,
      blueLightSupported: false,
      blueLightFactor: 1,
      pageUpDownSupported: false,
      pageUpDownEnabled: false,
      turboFanActive: false,
      slowFanActive: false,
      dttEnabled: false,
      powerModeLocked: false,
    },
  },
  capturedAt: new Date().toISOString(),
};

const activePage = ref<PageId>("overview");
const snapshot = ref<Snapshot | null>(null);
const system = ref<SystemInfo>(structuredClone(emptySystem));
const diagnostics = ref<Diagnostics>(structuredClone(emptyDiagnostics));
const booting = ref(true);
const refreshing = ref(false);
const diagnosticsLoading = ref(false);
const busyMode = ref<number | null>(null);
const busyFan = ref(false);
const busyFeature = ref<string | null>(null);
const clock = ref(new Date());
const contentScroller = ref<HTMLElement | null>(null);
const toast = ref<{ kind: "success" | "error" | "info"; message: string } | null>(null);

let pollTimer: number | undefined;
let clockTimer: number | undefined;
let toastTimer: number | undefined;
let polling = false;

const currentMeta = computed(() => pageMeta[activePage.value]);
const clockText = computed(() =>
  clock.value.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" }),
);
const dateText = computed(() =>
  clock.value.toLocaleDateString([], { month: "short", day: "2-digit" }).toUpperCase(),
);
const lastCaptured = computed(() => {
  if (!snapshot.value) return "等待中";
  return new Date(snapshot.value.capturedAt).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
});

function showToast(message: string, kind: "success" | "error" | "info" = "info") {
  toast.value = { message, kind };
  if (toastTimer) window.clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => {
    toast.value = null;
  }, 4200);
}

function errorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "硬件请求失败";
}

async function bootstrap() {
  booting.value = true;
  const [snapshotResult, systemResult, diagnosticResult] = await Promise.all([
    getSnapshot(),
    getSystemInfo(),
    getDiagnostics(),
  ]);
  snapshot.value = snapshotResult;
  system.value = systemResult;
  diagnostics.value = diagnosticResult;
  booting.value = false;
  startPolling();
}

async function refreshSnapshot(silent = false) {
  if (polling || busyMode.value !== null || busyFan.value || busyFeature.value !== null) return;
  polling = true;
  if (!silent) refreshing.value = true;
  try {
    snapshot.value = await getSnapshot();
    if (!silent && activePage.value !== "diagnostics") {
      showToast("硬件状态已刷新", "success");
    }
  } catch (error) {
    showToast(errorMessage(error), "error");
  } finally {
    polling = false;
    refreshing.value = false;
  }
}

async function refreshDiagnostics() {
  diagnosticsLoading.value = true;
  try {
    const [nextDiagnostics, nextSnapshot] = await Promise.all([getDiagnostics(), getSnapshot()]);
    diagnostics.value = nextDiagnostics;
    snapshot.value = nextSnapshot;
    showToast("诊断检查已完成", "success");
  } catch (error) {
    showToast(errorMessage(error), "error");
  } finally {
    diagnosticsLoading.value = false;
  }
}

async function refreshCurrent() {
  if (activePage.value === "diagnostics") {
    await refreshDiagnostics();
  } else {
    await refreshSnapshot(false);
  }
}

async function changePowerMode(mode: number) {
  if (busyMode.value !== null || mode === snapshot.value?.powerMode) return;
  busyMode.value = mode;
  try {
    snapshot.value = await writePowerMode(mode);
    showToast(`${snapshot.value.powerModeLabel}配置已应用`, "success");
  } catch (error) {
    showToast(errorMessage(error), "error");
    void refreshSnapshot(true);
  } finally {
    busyMode.value = null;
  }
}

async function changeFanMode(mode: number) {
  if (busyFan.value || mode === snapshot.value?.fan.mode) return;
  busyFan.value = true;
  try {
    snapshot.value = await writeFanMode(mode);
    showToast(mode === 1 ? "已请求最大散热" : "已恢复自动散热", "success");
  } catch (error) {
    showToast(errorMessage(error), "error");
    void refreshSnapshot(true);
  } finally {
    busyFan.value = false;
  }
}

async function changeFanOffset(offset: number) {
  if (busyFan.value || offset === snapshot.value?.fan.offset) return;
  busyFan.value = true;
  try {
    snapshot.value = await writeFanOffset(offset);
    showToast(`风扇曲线偏移已设为 ${offset}%`, "success");
  } catch (error) {
    showToast(errorMessage(error), "error");
    void refreshSnapshot(true);
  } finally {
    busyFan.value = false;
  }
}

async function changeKeyboardSettings(
  fnLock: boolean,
  windowsKeyEnabled: boolean,
  winFnSwap: boolean,
) {
  if (busyFeature.value) return;
  busyFeature.value = "keyboard";
  try {
    snapshot.value = await writeKeyboardSettings(fnLock, windowsKeyEnabled, winFnSwap);
    showToast("键盘按键策略已应用", "success");
  } catch (error) {
    showToast(errorMessage(error), "error");
    void refreshSnapshot(true);
  } finally {
    busyFeature.value = null;
  }
}

async function changeChargingThreshold(enabled: boolean, start: number, stop: number) {
  if (busyFeature.value) return;
  busyFeature.value = "charging";
  try {
    snapshot.value = await writeChargingThreshold(enabled, start, stop);
    showToast("FlexiCharger 阈值已应用", "success");
  } catch (error) {
    showToast(errorMessage(error), "error");
    void refreshSnapshot(true);
  } finally {
    busyFeature.value = null;
  }
}

async function changeLowRefreshRate(enabled: boolean) {
  if (busyFeature.value) return;
  busyFeature.value = "display";
  try {
    snapshot.value = await writeLowRefreshRate(enabled);
    showToast(enabled ? "低刷新率策略已启用" : "低刷新率策略已关闭", "success");
  } catch (error) {
    showToast(errorMessage(error), "error");
    void refreshSnapshot(true);
  } finally {
    busyFeature.value = null;
  }
}

async function changeBlueLightFactor(factor: number) {
  if (busyFeature.value) return;
  busyFeature.value = "display";
  try {
    snapshot.value = await writeBlueLightFactor(factor);
    showToast(`蓝光过滤强度已设为 ${factor.toFixed(1)}×`, "success");
  } catch (error) {
    showToast(errorMessage(error), "error");
    void refreshSnapshot(true);
  } finally {
    busyFeature.value = null;
  }
}

function navigate(page: PageId) {
  if (activePage.value === page) return;
  activePage.value = page;
  contentScroller.value?.scrollTo({ top: 0, behavior: "smooth" });
  if (page === "diagnostics" && !diagnosticsLoading.value) {
    void refreshDiagnostics();
  }
}

function startPolling() {
  if (pollTimer) window.clearInterval(pollTimer);
  pollTimer = window.setInterval(() => {
    if (
      snapshot.value?.connected &&
      busyMode.value === null &&
      !busyFan.value &&
      busyFeature.value === null &&
      document.visibilityState === "visible"
    ) {
      void refreshSnapshot(true);
    }
  }, 3000);
}

function onKeydown(event: KeyboardEvent) {
  const target = event.target as HTMLElement | null;
  if (target?.matches("input, textarea, select, button, [contenteditable='true']")) return;
  const index = Number(event.key) - 1;
  if (index >= 0 && index < navOrder.length) {
    navigate(navOrder[index]);
  }
  if (event.key.toLowerCase() === "r" && !event.ctrlKey && !event.metaKey) {
    void refreshCurrent();
  }
}

onMounted(() => {
  clockTimer = window.setInterval(() => {
    clock.value = new Date();
  }, 1000);
  window.addEventListener("keydown", onKeydown);
  void bootstrap();
});

onBeforeUnmount(() => {
  if (pollTimer) window.clearInterval(pollTimer);
  if (clockTimer) window.clearInterval(clockTimer);
  if (toastTimer) window.clearTimeout(toastTimer);
  window.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <div class="app-shell">
    <Sidebar
      :active-page="activePage"
      :snapshot="snapshot ?? emptySnapshot"
      :system="system"
      @navigate="navigate"
    />

    <div class="workspace">
      <TitleBar :page-title="currentMeta.title" />

      <div class="workspace-bar">
        <div class="workspace-index">
          <span>{{ currentMeta.index }}</span>
          <div>
            <small>CC30 / NOVA</small>
            <strong>{{ currentMeta.title }}</strong>
          </div>
        </div>

        <div class="workspace-tools">
          <div class="workspace-user">
            <span class="workspace-avatar"><Cpu :size="15" /></span>
            <span>
              <small>本地设备</small>
              <strong>{{ system.productName || system.hostname || "CLEVO" }}</strong>
            </span>
          </div>
          <div class="connection-readout" :class="{ offline: !snapshot?.connected }">
            <Wifi v-if="snapshot?.connected" :size="15" />
            <WifiOff v-else :size="15" />
            <span>{{ snapshot?.connected ? "DCHU 在线" : "未连接" }}</span>
          </div>
          <div class="capture-readout">
            <Clock3 :size="14" />
            <span>{{ lastCaptured }}</span>
          </div>
          <div class="clock-readout">
            <span>{{ dateText }}</span>
            <strong>{{ clockText }}</strong>
          </div>
          <button
            class="refresh-button"
            type="button"
            :disabled="refreshing || diagnosticsLoading"
            aria-label="刷新硬件状态"
            @click="refreshCurrent"
          >
            <RefreshCw :size="17" :class="{ spinning: refreshing || diagnosticsLoading }" />
          </button>
        </div>
      </div>

      <main ref="contentScroller" class="content-scroller">
        <div v-if="booting || !snapshot" class="boot-screen">
          <div class="boot-mark">
            <span></span>
            <Cpu :size="30" stroke-width="1.2" />
            <span></span>
          </div>
          <span class="eyebrow">CC30 Nova / 正在初始化</span>
          <h2>正在建立设备连接。</h2>
          <p>正在读取固件身份、温度遥测与已签名的 DCHU 桥接。</p>
          <div class="boot-progress"><i></i></div>
          <div class="boot-log">
            <Terminal :size="14" />
            <span>{{ isTauriRuntime() ? "原生桥接" : "本地预览" }}</span>
          </div>
        </div>

        <Transition v-else name="page" mode="out-in">
          <OverviewView
            v-if="activePage === 'overview'"
            key="overview"
            :snapshot="snapshot"
            :system="system"
            :busy-mode="busyMode"
            :busy-fan="busyFan"
            @select-mode="changePowerMode"
            @set-fan-mode="changeFanMode"
            @navigate="navigate"
          />
          <PowerView
            v-else-if="activePage === 'power'"
            key="power"
            :snapshot="snapshot"
            :busy-mode="busyMode"
            @select-mode="changePowerMode"
          />
          <CoolingView
            v-else-if="activePage === 'cooling'"
            key="cooling"
            :snapshot="snapshot"
            :busy-fan="busyFan"
            @set-fan-mode="changeFanMode"
            @set-fan-offset="changeFanOffset"
          />
          <FeaturesView
            v-else-if="activePage === 'features'"
            key="features"
            :snapshot="snapshot"
            :system="system"
            :busy-feature="busyFeature"
            @set-keyboard="changeKeyboardSettings"
            @set-charging="changeChargingThreshold"
            @set-low-refresh="changeLowRefreshRate"
            @set-blue-light="changeBlueLightFactor"
          />
          <HardwareView
            v-else-if="activePage === 'hardware'"
            key="hardware"
            :system="system"
          />
          <DiagnosticsView
            v-else
            key="diagnostics"
            :diagnostics="diagnostics"
            :snapshot="snapshot"
            :loading="diagnosticsLoading"
            @refresh="refreshDiagnostics"
            @toast="showToast"
          />
        </Transition>
      </main>

      <footer class="status-footer">
        <span><i class="status-dot"></i> {{ snapshot?.connected ? "硬件连接正常" : "未连接硬件" }}</span>
        <span class="footer-center">CLEVO CC30 设备管理</span>
        <span>数字键 1–6 导航 · R 刷新</span>
      </footer>
    </div>

    <Transition name="toast">
      <div v-if="toast" class="toast" :class="`toast-${toast.kind}`" role="status" aria-live="polite">
        <CheckCircle2 v-if="toast.kind === 'success'" :size="18" />
        <WifiOff v-else-if="toast.kind === 'error'" :size="18" />
        <Terminal v-else :size="18" />
        <span>{{ toast.message }}</span>
        <button type="button" aria-label="关闭通知" @click="toast = null">
          <X :size="15" />
        </button>
      </div>
    </Transition>
  </div>
</template>

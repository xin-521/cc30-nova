import { invoke } from "@tauri-apps/api/core";
import type {
  Diagnostics,
  FanSnapshot,
  FeatureFlag,
  Snapshot,
  SystemInfo,
} from "./types";

const isTauri = "__TAURI_INTERNALS__" in window;

const modeLabels = ["静音", "节能", "性能", "娱乐"];

const mockFan: FanSnapshot = {
  mode: 0,
  modeLabel: "自动",
  offset: 6,
  fanCount: 2,
  customCurveSupported: true,
  channels: [
    { id: "cpu", label: "CPU 风扇", rpm: 2840, temperatureC: 67, dutyPercent: 42 },
    { id: "gpu1", label: "GPU 风扇", rpm: 3120, temperatureC: 61, dutyPercent: 48 },
  ],
  curves: [
    {
      id: "cpu",
      label: "CPU",
      points: [
        { temperatureC: 40, dutyPercent: 24 },
        { temperatureC: 60, dutyPercent: 40 },
        { temperatureC: 80, dutyPercent: 65 },
        { temperatureC: 100, dutyPercent: 100 },
      ],
    },
    {
      id: "gpu1",
      label: "GPU",
      points: [
        { temperatureC: 40, dutyPercent: 24 },
        { temperatureC: 60, dutyPercent: 40 },
        { temperatureC: 80, dutyPercent: 64 },
        { temperatureC: 100, dutyPercent: 100 },
      ],
    },
  ],
};

const mockFeatureFlags: FeatureFlag[] = [
  { id: "power", label: "电源模式", supported: true, access: "writable", detail: "固件支持静音、性能；已实现写入与回读" },
  { id: "fan", label: "风扇模式", supported: true, access: "writable", detail: "自动与最大模式" },
  { id: "fan_offset", label: "风扇偏移", supported: true, access: "writable", detail: "0-100% 曲线偏移" },
  { id: "fan_curve", label: "风扇曲线", supported: true, access: "readOnly", detail: "读取固件温控映射" },
  { id: "gpu_switch", label: "GPU 模式", supported: true, access: "readOnly", detail: "显示当前模式与可用选项" },
  { id: "keyboard", label: "键盘设置", supported: true, access: "writable", detail: "控制器类型 6；FnLock、Win Key 与 Win/Fn 可设置" },
  { id: "charging", label: "FlexiCharger", supported: true, access: "writable", detail: "充电开关与启停阈值可设置" },
  { id: "rgb", label: "键盘 RGB", supported: true, access: "notImplemented", detail: "固件声明通用灯效能力" },
  { id: "battery", label: "电池固件诊断", supported: true, access: "readOnly", detail: "Windows 数据与 WMI 7 状态" },
  { id: "battery_refresh", label: "电池校准", supported: true, access: "blocked", detail: "放电与 AC 控制未纳入白名单" },
  { id: "energy_save", label: "节能功能", supported: false, access: "unsupported", detail: "当前固件未声明此能力" },
  { id: "anti_dust", label: "风扇除尘", supported: false, access: "unsupported", detail: "当前固件未声明" },
  { id: "dtt", label: "动态散热 DTT", supported: false, access: "unsupported", detail: "当前固件未声明" },
  { id: "turbo_fan", label: "Turbo Fan", supported: false, access: "unsupported", detail: "BIOS 未声明" },
  { id: "slow_fan", label: "Slow Fan", supported: false, access: "unsupported", detail: "BIOS 未声明" },
  { id: "max_q_fan", label: "Max-Q Fan", supported: false, access: "unsupported", detail: "当前平台未声明" },
  { id: "silent_fan", label: "Fanless", supported: false, access: "unsupported", detail: "当前平台未声明" },
  { id: "flexikey", label: "Flexikey", supported: true, access: "notImplemented", detail: "固件能力已声明，映射协议未接入" },
  { id: "flexi_access", label: "FlexiAccess", supported: false, access: "unsupported", detail: "当前固件未声明" },
  { id: "per_key", label: "单键 RGB", supported: false, access: "unsupported", detail: "当前键盘未声明" },
  { id: "page_up_down", label: "PgUp / PgDn 功能键", supported: false, access: "unsupported", detail: "当前固件未声明" },
  { id: "cc_function", label: "CC 功能键", supported: true, access: "notImplemented", detail: "固件声明，事件处理未接入" },
  { id: "low_refresh_rate", label: "低刷新率", supported: true, access: "writable", detail: "低刷新率策略可设置" },
  { id: "blue_light", label: "蓝光过滤", supported: true, access: "writable", detail: "1.0× 至 2.0× 强度可设置" },
  { id: "cpu_oc", label: "CPU 超频", supported: false, access: "blocked", detail: "安全策略不开放" },
  { id: "gpu_oc", label: "GPU 超频", supported: false, access: "blocked", detail: "安全策略不开放" },
  { id: "amd_manual", label: "AMD 手动调频", supported: false, access: "unsupported", detail: "当前平台未声明" },
];

function makeMockSnapshot(): Snapshot {
  return {
    connected: false,
    source: "simulation",
    powerMode: 2,
    powerModeLabel: modeLabels[2],
    fan: structuredClone(mockFan),
    capabilities: {
      featureRaw: 0x893,
      featurePackageVersion: "1.0",
      featureFlags: structuredClone(mockFeatureFlags),
      powerModes: true,
      powerModeOptions: [0, 2],
      fanControl: true,
      keyboardRgb: true,
      rgbZones: 0,
      perKeyKeyboard: false,
      keyboardControllerType: 6,
      cpuOverclock: false,
      gpuOverclock: false,
      flexikey: true,
      fanCount: 2,
      customFanCurves: true,
      turboFan: false,
      slowFan: false,
      maxQFan: false,
      silentFan: false,
      antiDust: false,
      dtt: false,
      fanOffsetSupported: true,
      lowRefreshRate: true,
      blueLightFilter: true,
      gpuSwitch: true,
      keyboardSettings: true,
      flexiCharger: true,
      batteryUtility: false,
      energySave: false,
      flexiAccess: false,
      pageUpDownKey: false,
      controlCenterFunction: true,
      amdManualMode: false,
    },
    device: {
      ecVersion: "1.07.03",
      gpuMode: {
        supported: true,
        current: 2,
        currentLabel: "独显模式",
        options: [
          { value: 2, label: "独显模式" },
          { value: 3, label: "混合模式" },
        ],
        restartRequired: true,
      },
      keyboard: {
        supported: true,
        controllerType: 6,
        fnLock: false,
        fnLockSupported: true,
        windowsKeyEnabled: true,
        windowsKeySupported: true,
        winFnSwap: false,
        winFnSupported: true,
      },
      charging: {
        supported: true,
        enabled: true,
        startPercent: 95,
        stopPercent: 100,
        startOptions: [40, 50, 60, 70, 80, 95],
        stopOptions: [60, 70, 80, 90, 100],
      },
      batteryRuntime: {
        present: true,
        powerOnline: true,
        charging: false,
        discharging: false,
        chargePercent: 100,
      },
      batteryFirmware: {
        supported: true,
        version: 1,
        manufacturedOn: "2023-11-29",
        cycleCount: 0,
        fullChargeCapacityMwh: 39870,
        designCapacityMwh: 51528,
        healthPercent: 77,
        batteryStatus: 0,
        pfStatus: 0,
        operationStatus: 0,
        stopChargingThreshold: 95,
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
        lowRefreshRateSupported: true,
        lowRefreshRateEnabled: true,
        blueLightSupported: true,
        blueLightFactor: 1.0,
        pageUpDownSupported: false,
        pageUpDownEnabled: false,
        turboFanActive: false,
        slowFanActive: false,
        dttEnabled: false,
        powerModeLocked: false,
      },
    },
    capturedAt: new Date().toISOString(),
    notice: "未检测到硬件桥接，当前显示模拟数据。",
  };
}

const mockSystem: SystemInfo = {
  manufacturer: "THUNDERBOT",
  productName: "911MT / V158RNXW-HL",
  serialNumber: "V158RNXW-DEMO",
  family: "游戏笔记本",
  biosVendor: "Insyde",
  biosVersion: "1.07.02",
  biosDate: "06/18/2025",
  boardManufacturer: "CLEVO",
  boardProduct: "V158RNXW-HL",
  boardVersion: "1.0",
  cpuName: "Intel Core i7-13620H",
  cpuCores: 10,
  cpuThreads: 16,
  memoryTotalGb: 16,
  memoryModules: [
    {
      locator: "ChannelA-DIMM0",
      manufacturer: "Samsung",
      partNumber: "M425R2GA3BB0-CQKOL",
      speedMhz: 4800,
      sizeGb: 8,
    },
    {
      locator: "ChannelB-DIMM0",
      manufacturer: "Samsung",
      partNumber: "M425R2GA3BB0-CQKOL",
      speedMhz: 4800,
      sizeGb: 8,
    },
  ],
  osName: "Windows 11 Pro",
  osBuild: "26100.6899",
  hostname: "NOVA",
  gpuNames: ["NVIDIA GeForce RTX 4060 Laptop GPU", "Intel UHD Graphics"],
  battery: {
    present: true,
    model: "标准电池",
    manufacturer: "CLEVO",
    designedCapacityMwh: 51528,
    fullChargedCapacityMwh: 39870,
    cycleCount: 0,
    healthPercent: 77,
  },
};

const mockDiagnostics: Diagnostics = {
  dllPath: "simulation/InsydeDCHU.dll",
  dllPresent: false,
  apiAvailable: false,
  outLogAvailable: true,
  powerBiosLogAvailable: true,
  featureRaw: 0,
  wmi16: 0x893,
  wmi96: 0x21c,
  wmi122: 0,
  nativeApis: [
    { name: "GetDCHU_Data_Integer", direction: "读取", purpose: "读取 WMI 整数能力字与遥测状态", loaded: false },
    { name: "GetDCHU_Data_Buffer", direction: "读取", purpose: "读取 WMI 12/13 遥测与配置缓冲区", loaded: false },
    { name: "ReadAppSettings", direction: "读取", purpose: "读取电源、风扇、键盘与显示策略持久化设置", loaded: false },
    { name: "WriteAppSettings", direction: "写入", purpose: "持久化已验证的固定策略值", loaded: false },
    { name: "SetDCHU_Data", direction: "写入", purpose: "发送固定风扇子命令", loaded: false },
    { name: "SetDCHU_DataEx", direction: "查询与写入", purpose: "查询 GPU、键盘、充电与 EC 固件状态", loaded: false },
  ],
  checks: [
    { id: "dll", label: "原生桥接", status: "warn", detail: "未连接" },
    { id: "api", label: "DCHU 握手", status: "warn", detail: "未执行" },
    { id: "fan", label: "散热系统", status: "ok", detail: "模拟 2 个风扇通道" },
    { id: "feature-package", label: "BIOS 功能包", status: "ok", detail: "1.0" },
    { id: "gpu-switch", label: "GPU 模式", status: "ok", detail: "独显模式" },
    { id: "keyboard", label: "键盘设置", status: "ok", detail: "FnLock / Win Key / Win-Fn" },
    { id: "charging", label: "充电阈值", status: "ok", detail: "FlexiCharger 状态" },
    { id: "battery", label: "电池健康", status: "ok", detail: "77% · 0 次循环" },
    { id: "safety", label: "写入白名单", status: "ok", detail: "安全策略已启用" },
  ],
};

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) {
    throw new Error("桌面运行环境不可用");
  }
  return invoke<T>(command, args);
}

export async function getSnapshot(): Promise<Snapshot> {
  try {
    return await call<Snapshot>("get_snapshot");
  } catch {
    return makeMockSnapshot();
  }
}

export async function getSystemInfo(): Promise<SystemInfo> {
  try {
    return await call<SystemInfo>("get_system_info");
  } catch {
    return structuredClone(mockSystem);
  }
}

export async function getDiagnostics(): Promise<Diagnostics> {
  try {
    return await call<Diagnostics>("get_diagnostics");
  } catch {
    return structuredClone(mockDiagnostics);
  }
}

export async function setPowerMode(mode: number): Promise<Snapshot> {
  if (!isTauri) {
    const snapshot = makeMockSnapshot();
    snapshot.powerMode = mode;
    snapshot.powerModeLabel = modeLabels[mode] ?? "未知";
    snapshot.notice = "未连接硬件，电源模式未变更。";
    return snapshot;
  }
  return call<Snapshot>("set_power_mode", { mode });
}

export async function setFanMode(mode: number): Promise<Snapshot> {
  if (!isTauri) {
    const snapshot = makeMockSnapshot();
    snapshot.fan.mode = mode;
    snapshot.fan.modeLabel = mode === 1 ? "最大" : "自动";
    snapshot.notice = "未连接硬件，风扇模式未变更。";
    return snapshot;
  }
  return call<Snapshot>("set_fan_mode", { mode });
}

export async function setFanOffset(offset: number): Promise<Snapshot> {
  if (!isTauri) {
    const snapshot = makeMockSnapshot();
    snapshot.fan.offset = offset;
    snapshot.notice = "未连接硬件，风扇偏移未变更。";
    return snapshot;
  }
  return call<Snapshot>("set_fan_offset", { offset });
}

export async function setKeyboardSettings(
  fnLock: boolean,
  windowsKeyEnabled: boolean,
  winFnSwap: boolean,
): Promise<Snapshot> {
  if (!isTauri) {
    const snapshot = makeMockSnapshot();
    snapshot.device.keyboard.fnLock = fnLock;
    snapshot.device.keyboard.windowsKeyEnabled = windowsKeyEnabled;
    snapshot.device.keyboard.winFnSwap = winFnSwap;
    snapshot.notice = "未连接硬件，键盘设置未变更。";
    return snapshot;
  }
  return call<Snapshot>("set_keyboard_settings", {
    fnLock,
    windowsKeyEnabled,
    winFnSwap,
  });
}

export async function setChargingThreshold(
  enabled: boolean,
  startPercent: number,
  stopPercent: number,
): Promise<Snapshot> {
  if (!isTauri) {
    const snapshot = makeMockSnapshot();
    snapshot.device.charging.enabled = enabled;
    snapshot.device.charging.startPercent = startPercent;
    snapshot.device.charging.stopPercent = stopPercent;
    snapshot.notice = "未连接硬件，充电阈值未变更。";
    return snapshot;
  }
  return call<Snapshot>("set_charging_threshold", {
    enabled,
    startPercent,
    stopPercent,
  });
}

export async function setLowRefreshRate(enabled: boolean): Promise<Snapshot> {
  if (!isTauri) {
    const snapshot = makeMockSnapshot();
    snapshot.device.systemPolicy.lowRefreshRateEnabled = enabled;
    snapshot.notice = "未连接硬件，低刷新率策略未变更。";
    return snapshot;
  }
  return call<Snapshot>("set_low_refresh_rate", { enabled });
}

export async function setBlueLightFactor(factor: number): Promise<Snapshot> {
  if (!isTauri) {
    const snapshot = makeMockSnapshot();
    snapshot.device.systemPolicy.blueLightFactor = factor;
    snapshot.notice = "未连接硬件，蓝光过滤强度未变更。";
    return snapshot;
  }
  return call<Snapshot>("set_blue_light_factor", { factor });
}

export function isTauriRuntime(): boolean {
  return isTauri;
}

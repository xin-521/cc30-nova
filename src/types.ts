export type PageId =
  | "overview"
  | "power"
  | "cooling"
  | "features"
  | "hardware"
  | "diagnostics";

export interface FanChannel {
  id: string;
  label: string;
  rpm: number;
  temperatureC: number;
  dutyPercent: number;
}

export interface FanCurvePoint {
  temperatureC: number;
  dutyPercent: number;
}

export interface FanCurve {
  id: string;
  label: string;
  points: FanCurvePoint[];
}

export interface FanSnapshot {
  mode: number;
  modeLabel: string;
  offset: number;
  fanCount: number;
  customCurveSupported: boolean;
  channels: FanChannel[];
  curves: FanCurve[];
}

export interface Capabilities {
  featureRaw: number;
  featurePackageVersion: string;
  featureFlags: FeatureFlag[];
  powerModes: boolean;
  powerModeOptions: number[];
  fanControl: boolean;
  keyboardRgb: boolean;
  rgbZones: number;
  perKeyKeyboard: boolean;
  keyboardControllerType: number;
  cpuOverclock: boolean;
  gpuOverclock: boolean;
  flexikey: boolean;
  fanCount: number;
  customFanCurves: boolean;
  turboFan: boolean;
  slowFan: boolean;
  maxQFan: boolean;
  silentFan: boolean;
  antiDust: boolean;
  dtt: boolean;
  fanOffsetSupported: boolean;
  lowRefreshRate: boolean;
  blueLightFilter: boolean;
  gpuSwitch: boolean;
  keyboardSettings: boolean;
  flexiCharger: boolean;
  batteryUtility: boolean;
  energySave: boolean;
  flexiAccess: boolean;
  pageUpDownKey: boolean;
  controlCenterFunction: boolean;
  amdManualMode: boolean;
}

export interface FeatureFlag {
  id: string;
  label: string;
  supported: boolean;
  access: "writable" | "readOnly" | "notImplemented" | "blocked" | "unsupported";
  detail: string;
}

export interface GpuModeOption {
  value: number;
  label: string;
}

export interface GpuModeState {
  supported: boolean;
  current: number;
  currentLabel: string;
  options: GpuModeOption[];
  restartRequired: boolean;
}

export interface KeyboardSettings {
  supported: boolean;
  controllerType: number;
  fnLock: boolean;
  fnLockSupported: boolean;
  windowsKeyEnabled: boolean;
  windowsKeySupported: boolean;
  winFnSwap: boolean;
  winFnSupported: boolean;
}

export interface ChargingSettings {
  supported: boolean;
  enabled: boolean;
  startPercent: number;
  stopPercent: number;
  startOptions: number[];
  stopOptions: number[];
}

export interface BatteryRuntime {
  present: boolean;
  powerOnline: boolean;
  charging: boolean;
  discharging: boolean;
  chargePercent: number;
}

export interface BatteryFirmware {
  supported: boolean;
  version: number;
  manufacturedOn?: string;
  cycleCount: number;
  fullChargeCapacityMwh: number;
  designCapacityMwh: number;
  healthPercent: number;
  batteryStatus: number;
  pfStatus: number;
  operationStatus: number;
  stopChargingThreshold: number;
  refreshChargePhase: boolean;
  refreshDischargePhase: boolean;
  permanentFailure: boolean;
}

export interface AntiDustState {
  supported: boolean;
  running: boolean;
  schedulerEnabled: boolean;
  dayMask: number;
  hour: number;
  resumeAfterBoot: boolean;
}

export interface SystemPolicyState {
  lowRefreshRateSupported: boolean;
  lowRefreshRateEnabled: boolean;
  blueLightSupported: boolean;
  blueLightFactor: number;
  pageUpDownSupported: boolean;
  pageUpDownEnabled: boolean;
  turboFanActive: boolean;
  slowFanActive: boolean;
  dttEnabled: boolean;
  powerModeLocked: boolean;
}

export interface DeviceState {
  ecVersion?: string;
  gpuMode: GpuModeState;
  keyboard: KeyboardSettings;
  charging: ChargingSettings;
  batteryRuntime: BatteryRuntime;
  batteryFirmware: BatteryFirmware;
  antiDust: AntiDustState;
  systemPolicy: SystemPolicyState;
}

export interface Snapshot {
  connected: boolean;
  source: "native" | "simulation";
  powerMode: number;
  powerModeLabel: string;
  fan: FanSnapshot;
  capabilities: Capabilities;
  device: DeviceState;
  capturedAt: string;
  notice?: string;
}

export interface MemoryModule {
  locator: string;
  manufacturer: string;
  partNumber: string;
  speedMhz: number;
  sizeGb: number;
}

export interface SystemInfo {
  manufacturer: string;
  productName: string;
  serialNumber: string;
  family: string;
  biosVendor: string;
  biosVersion: string;
  biosDate: string;
  boardManufacturer: string;
  boardProduct: string;
  boardVersion: string;
  cpuName: string;
  cpuCores: number;
  cpuThreads: number;
  memoryTotalGb: number;
  memoryModules: MemoryModule[];
  osName: string;
  osBuild: string;
  hostname: string;
  gpuNames: string[];
  battery: BatteryInfo;
}

export interface BatteryInfo {
  present: boolean;
  model: string;
  manufacturer: string;
  designedCapacityMwh: number;
  fullChargedCapacityMwh: number;
  cycleCount: number;
  healthPercent: number;
}

export interface DiagnosticCheck {
  id: string;
  label: string;
  status: "ok" | "warn" | "error";
  detail: string;
}

export interface Diagnostics {
  dllPath?: string;
  dllPresent: boolean;
  apiAvailable: boolean;
  outLogAvailable: boolean;
  powerBiosLogAvailable: boolean;
  featureRaw: number;
  wmi16: number;
  wmi96: number;
  wmi122: number;
  nativeApis: NativeApi[];
  checks: DiagnosticCheck[];
}

export interface NativeApi {
  name: string;
  direction: string;
  purpose: string;
  loaded: boolean;
}

export interface AppSettings {
  pollingIntervalMs: number;
  reduceMotion: boolean;
  showRawDiagnostics: boolean;
}

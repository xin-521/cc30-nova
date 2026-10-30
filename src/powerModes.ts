export interface PowerMode {
  id: number;
  code: string;
  name: string;
  summary: string;
  detail: string;
  icon: "moon" | "leaf" | "zap" | "sparkles";
  color: string;
  accent: string;
}

export const powerModes: PowerMode[] = [
  {
    id: 0,
    code: "静音优先",
    name: "静音",
    summary: "降低风扇转速与性能上限。",
    detail: "适用于轻量办公、阅读和其他低负载场景。",
    icon: "moon",
    color: "#8da8ff",
    accent: "#d2dcff",
  },
  {
    id: 1,
    code: "续航优先",
    name: "节能",
    summary: "限制处理器功耗与温度。",
    detail: "在电池供电时优先延长续航时间。",
    icon: "leaf",
    color: "#56d6c2",
    accent: "#c6fff4",
  },
  {
    id: 2,
    code: "平衡模式",
    name: "性能",
    summary: "平衡性能、温度与噪声。",
    detail: "适用于日常办公和混合负载。",
    icon: "zap",
    color: "#ff4055",
    accent: "#ffabb5",
  },
  {
    id: 3,
    code: "高性能",
    name: "娱乐",
    summary: "提高处理器与显卡性能上限。",
    detail: "在高负载时采用更积极的功耗与散热策略。",
    icon: "sparkles",
    color: "#ff7d91",
    accent: "#ffd0d7",
  },
];

export function powerModeById(id: number): PowerMode {
  return powerModes[id] ?? powerModes[2];
}

export function powerModesFor(options: number[] | undefined): PowerMode[] {
  if (!options?.length) return [];
  return options
    .map((id) => powerModes.find((mode) => mode.id === id))
    .filter((mode): mode is PowerMode => mode !== undefined);
}

export function powerModePosition(id: number, options: number[] | undefined): number {
  if (!options) return 0;
  return options.indexOf(id) + 1;
}

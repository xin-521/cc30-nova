<script setup lang="ts">
import {
  Activity,
  Check,
  CircleAlert,
  Clipboard,
  FileWarning,
  Link2,
  RefreshCw,
  ShieldCheck,
  TerminalSquare,
  X,
} from "lucide-vue-next";
import type { Diagnostics, Snapshot } from "../types";

defineProps<{
  diagnostics: Diagnostics;
  snapshot: Snapshot;
  loading: boolean;
}>();

const emit = defineEmits<{
  refresh: [];
  copy: [text: string];
  toast: [message: string];
}>();

function copyPath(path?: string) {
  if (!path) return;
  void navigator.clipboard.writeText(path).then(
    () => emit("toast", "原生桥接路径已复制"),
    () => emit("toast", "剪贴板不可用"),
  );
}

function statusLabel(status: string): string {
  if (status === "ok") return "正常";
  if (status === "warn") return "警告";
  return "错误";
}
</script>

<template>
  <div class="view-stack diagnostics-view">
    <section class="view-intro reveal">
      <div>
        <span class="eyebrow">系统检查</span>
        <h2>诊断工具</h2>
        <p>检查系统服务、原生桥接与固件能力状态。</p>
      </div>
      <div class="intro-aside">
        <button class="primary-action" type="button" :disabled="loading" @click="emit('refresh')">
          <RefreshCw :size="16" :class="{ spinning: loading }" />
          重新运行检查
        </button>
        <p>只读检查硬件桥接、固件能力标志与事件通道。</p>
      </div>
    </section>

    <div class="diagnostic-overview">
      <section
        v-for="check in diagnostics.checks"
        :key="check.id"
        class="diagnostic-card reveal"
        :class="`diagnostic-${check.status}`"
      >
        <span class="diagnostic-icon">
          <Check v-if="check.status === 'ok'" :size="18" />
          <CircleAlert v-else-if="check.status === 'warn'" :size="18" />
          <X v-else :size="18" />
        </span>
        <div>
          <span>{{ check.label }}</span>
          <strong>{{ statusLabel(check.status) }}</strong>
          <p>{{ check.detail }}</p>
        </div>
      </section>
    </div>

    <div class="diagnostic-grid">
      <section class="panel bridge-panel reveal reveal-delay-2">
        <div class="panel-heading">
          <div>
            <span class="eyebrow">原生桥接</span>
            <h3>Insyde DCHU</h3>
          </div>
          <Link2 :size="20" class="muted-icon" />
        </div>
        <div class="path-box">
          <TerminalSquare :size="18" />
          <code>{{ diagnostics.dllPath || "未找到 InsydeDCHU.dll" }}</code>
          <button type="button" aria-label="复制 DLL 路径" @click="copyPath(diagnostics.dllPath)">
            <Clipboard :size="15" />
          </button>
        </div>
        <dl class="diagnostic-key-values">
          <div><dt>DLL 已部署</dt><dd>{{ diagnostics.dllPresent ? "是" : "否" }}</dd></div>
          <div><dt>API 握手</dt><dd>{{ diagnostics.apiAvailable ? "正常" : "失败" }}</dd></div>
          <div><dt>OutLog</dt><dd>{{ diagnostics.outLogAvailable ? "已开放" : "缺失" }}</dd></div>
          <div><dt>PowerBiosServerLog</dt><dd>{{ diagnostics.powerBiosLogAvailable ? "已开放" : "缺失" }}</dd></div>
        </dl>
      </section>

      <section class="panel raw-panel reveal reveal-delay-3">
        <div class="panel-heading">
          <div>
            <span class="eyebrow">固件应答集合</span>
            <h3>原始能力字</h3>
          </div>
          <Activity :size="20" class="muted-icon" />
        </div>
        <div class="hex-grid">
          <div>
            <span>WMI 16</span>
            <code>0x{{ diagnostics.wmi16.toString(16).toUpperCase().padStart(4, "0") }}</code>
          </div>
          <div>
            <span>WMI 96</span>
            <code>0x{{ diagnostics.wmi96.toString(16).toUpperCase().padStart(4, "0") }}</code>
          </div>
          <div>
            <span>WMI 122</span>
            <code>0x{{ diagnostics.wmi122.toString(16).toUpperCase().padStart(8, "0") }}</code>
          </div>
          <div>
            <span>特性</span>
            <code>0x{{ diagnostics.featureRaw.toString(16).toUpperCase().padStart(4, "0") }}</code>
          </div>
        </div>
        <div class="capability-flags">
          <span :class="{ on: snapshot.capabilities.powerModes }">电源</span>
          <span :class="{ on: snapshot.capabilities.fanControl }">风扇</span>
          <span :class="{ on: snapshot.capabilities.keyboardRgb }">键盘 RGB</span>
          <span :class="{ on: snapshot.capabilities.gpuOverclock }">GPU 超频</span>
          <span :class="{ on: snapshot.capabilities.cpuOverclock }">CPU 超频</span>
          <span :class="{ on: snapshot.capabilities.flexikey }">FLEXIKEY</span>
        </div>
      </section>
    </div>

    <section class="panel native-api-panel reveal reveal-delay-4">
      <div class="panel-heading">
        <div>
          <span class="eyebrow">DCHU EXPORTS</span>
          <h3>原生导出接口</h3>
        </div>
        <span class="native-api-count">
          {{ diagnostics.nativeApis.filter((api) => api.loaded).length }} /
          {{ diagnostics.nativeApis.length }} 已加载
        </span>
      </div>
      <div class="native-api-list">
        <article
          v-for="api in diagnostics.nativeApis"
          :key="api.name"
          class="native-api-row"
          :class="{ loaded: api.loaded }"
        >
          <span class="native-api-state" aria-hidden="true"></span>
          <code>{{ api.name }}</code>
          <span class="native-api-direction">{{ api.direction }}</span>
          <p>{{ api.purpose }}</p>
          <strong>{{ api.loaded ? "已加载" : "未加载" }}</strong>
        </article>
      </div>
    </section>

    <div class="diagnostic-bottom">
      <section class="panel whitelist-panel reveal reveal-delay-4">
        <div class="icon-box"><ShieldCheck :size="21" /></div>
        <div>
          <span class="eyebrow">控制范围</span>
          <h3>仅开放十项固定设备命令。</h3>
          <p>
            支持电源与风扇、功能键、FlexiCharger、低刷新率及蓝光过滤。全部写入均经过能力检查、
            范围约束和回读确认，不提供任意命令接口、固件刷写或超频写入。
          </p>
          <div class="safety-tags">
            <span>固定命令 ID</span>
            <span>范围校验</span>
            <span>回读验证</span>
          </div>
        </div>
      </section>

      <section class="panel warning-panel reveal reveal-delay-5">
        <FileWarning :size="22" />
        <div>
          <span class="eyebrow">运行依赖</span>
          <h3>CLEVO 系统服务</h3>
          <p>
            电源配置依赖 CLEVO <code>PowerBiosServer</code> 事件日志服务；风扇命令依赖已签名的
            <code>InsydeDCHU.dll</code> ACPI 桥接。
          </p>
        </div>
      </section>
    </div>
  </div>
</template>

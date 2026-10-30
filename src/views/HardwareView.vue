<script setup lang="ts">
import {
  Binary,
  Box,
  CircuitBoard,
  Cpu,
  Gauge,
  HardDrive,
  MemoryStick,
  MonitorCog,
  Server,
} from "lucide-vue-next";
import type { SystemInfo } from "../types";

defineProps<{
  system: SystemInfo;
}>();
</script>

<template>
  <div class="view-stack hardware-view">
    <section class="view-intro reveal">
      <div>
        <span class="eyebrow">设备信息</span>
        <h2>硬件信息</h2>
        <p>读取处理器、显卡、内存与固件的设备信息。</p>
      </div>
      <div class="intro-aside">
        <span class="serial-label">序列号</span>
        <strong class="serial-value">{{ system.serialNumber || "已保护" }}</strong>
        <p>信息读取自本机 SMBIOS 固件表。</p>
      </div>
    </section>

    <section class="hardware-banner reveal reveal-delay-1">
      <div class="hardware-banner-mark">
        <CircuitBoard :size="40" stroke-width="1.2" />
      </div>
      <div>
        <span class="eyebrow">{{ system.family || "CLEVO 平台" }}</span>
        <h3>{{ system.productName }}</h3>
        <p>{{ system.manufacturer }} · {{ system.boardProduct }} · {{ system.boardVersion }}</p>
      </div>
      <div class="bios-stamp">
        <span>固件</span>
        <strong>{{ system.biosVersion }}</strong>
        <small>{{ system.biosDate }}</small>
      </div>
    </section>

    <div class="hardware-grid">
      <section class="panel hardware-card reveal reveal-delay-2">
        <div class="hardware-card-icon"><Cpu :size="22" /></div>
        <span class="eyebrow">处理器</span>
        <h3>{{ system.cpuName }}</h3>
        <div class="hardware-stat-row">
          <span><i>核心</i><strong>{{ system.cpuCores || "—" }}</strong></span>
          <span><i>线程</i><strong>{{ system.cpuThreads || "—" }}</strong></span>
          <span><i>平台</i><strong>x64</strong></span>
        </div>
      </section>

      <section class="panel hardware-card reveal reveal-delay-2">
        <div class="hardware-card-icon"><MemoryStick :size="22" /></div>
        <span class="eyebrow">内存</span>
        <h3>已安装 {{ system.memoryTotalGb }} GB</h3>
        <div class="hardware-stat-row">
          <span><i>模组</i><strong>{{ system.memoryModules.length }}</strong></span>
          <span><i>最高速率</i><strong>{{ Math.max(0, ...system.memoryModules.map((m) => m.speedMhz)) }} MT/s</strong></span>
          <span><i>类型</i><strong>DDR</strong></span>
        </div>
      </section>

      <section class="panel hardware-card reveal reveal-delay-3">
        <div class="hardware-card-icon"><MonitorCog :size="22" /></div>
        <span class="eyebrow">图形处理器</span>
        <h3>{{ system.gpuNames[0] || "图形适配器" }}</h3>
        <ul class="graphics-list">
          <li v-for="gpu in system.gpuNames" :key="gpu">{{ gpu }}</li>
        </ul>
      </section>

      <section class="panel hardware-card reveal reveal-delay-3">
        <div class="hardware-card-icon"><Binary :size="22" /></div>
        <span class="eyebrow">操作系统</span>
        <h3>{{ system.osName }}</h3>
        <div class="hardware-stat-row">
          <span><i>版本</i><strong>{{ system.osBuild }}</strong></span>
          <span><i>主机</i><strong>{{ system.hostname }}</strong></span>
          <span><i>架构</i><strong>x64</strong></span>
        </div>
      </section>
    </div>

    <section class="panel memory-table-panel reveal reveal-delay-4">
      <div class="panel-heading">
        <div>
          <span class="eyebrow">SMBIOS 类型 17</span>
          <h3>内存拓扑</h3>
        </div>
        <MemoryStick :size="20" class="muted-icon" />
      </div>
      <div class="data-table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>插槽</th>
              <th>制造商</th>
              <th>部件号</th>
              <th>速率</th>
              <th>容量</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="module in system.memoryModules" :key="module.locator">
              <td>{{ module.locator }}</td>
              <td>{{ module.manufacturer || "—" }}</td>
              <td>{{ module.partNumber || "—" }}</td>
              <td>{{ module.speedMhz || "—" }} MT/s</td>
              <td>{{ module.sizeGb }} GB</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <div class="hardware-footnotes">
      <span><HardDrive :size="16" /> 主板 {{ system.boardProduct }}</span>
      <span><Server :size="16" /> BIOS {{ system.biosVendor }} {{ system.biosVersion }}</span>
      <span><Box :size="16" /> {{ system.serialNumber }}</span>
      <span><Gauge :size="16" /> SMBIOS 固件表</span>
    </div>
  </div>
</template>

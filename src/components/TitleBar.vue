<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-vue-next";

defineProps<{
  pageTitle: string;
}>();

const isTauri = "__TAURI_INTERNALS__" in window;

function currentWindow() {
  return getCurrentWindow();
}

function minimize() {
  if (isTauri) void currentWindow().minimize();
}

function toggleMaximize() {
  if (isTauri) void currentWindow().toggleMaximize();
}

function close() {
  if (isTauri) void currentWindow().close();
}
</script>

<template>
  <header class="titlebar" data-tauri-drag-region @dblclick="toggleMaximize">
    <div class="titlebar-brand" data-tauri-drag-region>
      <span class="brand-mark" aria-hidden="true">
        <span></span>
        <span></span>
        <span></span>
      </span>
      <span class="brand-word">CC30</span>
      <span class="brand-version">NOVA / 01</span>
    </div>

    <div class="titlebar-context" data-tauri-drag-region>
      <span class="titlebar-separator"></span>
      <span>{{ pageTitle }}</span>
    </div>

    <div class="window-actions">
      <button class="window-button" type="button" aria-label="最小化" @click.stop="minimize">
        <Minus :size="15" />
      </button>
      <button class="window-button" type="button" aria-label="最大化" @click.stop="toggleMaximize">
        <Square :size="12" />
      </button>
      <button
        class="window-button window-button-close"
        type="button"
        aria-label="关闭"
        @click.stop="close"
      >
        <X :size="15" />
      </button>
    </div>
  </header>
</template>

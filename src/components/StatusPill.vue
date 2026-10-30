<script setup lang="ts">
defineProps<{
  tone?: "live" | "warn" | "muted";
  label: string;
}>();
</script>

<template>
  <span class="status-pill" :class="`status-${tone ?? 'live'}`">
    <span class="status-dot"></span>
    {{ label }}
  </span>
</template>

<script setup lang="ts">
defineProps<{
  label: string
  value: string | number
  note?: string
}>()
</script>

<template>
  <div class="card p-5">
    <div class="text-sm text-slate-400">{{ label }}</div>
    <div class="mt-2 text-2xl font-black text-white">{{ value }}</div>
    <div v-if="note" class="mt-1 text-xs text-slate-500">{{ note }}</div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  total: number
  page: number
  totalPages: number
}>()

const emit = defineEmits<{
  prev: []
  next: []
}>()
</script>

<template>
  <div class="mt-5 flex flex-col gap-3 rounded-2xl border border-white/10 bg-white/[0.03] px-4 py-3 text-sm text-slate-400 sm:flex-row sm:items-center sm:justify-between">
    <span>共 {{ total }} 条</span>
    <div class="flex items-center justify-between gap-3 sm:justify-end">
      <button class="btn-secondary" :disabled="page <= 1" type="button" @click="emit('prev')">上一页</button>
      <span class="font-mono text-slate-300">{{ page }} / {{ totalPages }}</span>
      <button class="btn-secondary" :disabled="page >= totalPages" type="button" @click="emit('next')">下一页</button>
    </div>
  </div>
</template>

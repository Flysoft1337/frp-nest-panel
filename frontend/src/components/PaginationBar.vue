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
  <div class="mt-4 flex items-center justify-between text-sm text-slate-400">
    <span>共 {{ total }} 条</span>
    <div class="flex items-center gap-2">
      <button class="btn-secondary" :disabled="page <= 1" type="button" @click="emit('prev')">上一页</button>
      <span>{{ page }} / {{ totalPages }}</span>
      <button class="btn-secondary" :disabled="page >= totalPages" type="button" @click="emit('next')">下一页</button>
    </div>
  </div>
</template>

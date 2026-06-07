<script setup lang="ts">
withDefaults(
  defineProps<{
    message: string
    tone?: 'success' | 'danger' | 'default'
  }>(),
  { tone: 'default' },
)
</script>

<template>
  <p
    class="rounded-2xl border px-4 py-3 text-sm"
    :class="{
      'border-emerald-300/20 bg-emerald-400/10 text-emerald-100': tone === 'success',
      'border-red-300/20 bg-red-400/10 text-red-100': tone === 'danger',
      'border-white/10 bg-white/5 text-slate-200': tone === 'default',
    }"
  >
    {{ message }}
  </p>
</template>

<script setup lang="ts">
defineProps<{
  tone?: 'default' | 'success' | 'danger' | 'warning' | 'info' | 'muted' | 'pending'
}>()
</script>

<template>
  <span
    class="status-pill"
    :class="{
      'status-success': tone === 'success',
      'status-danger': tone === 'danger',
      'border-amber-300/20 bg-amber-400/10 text-amber-100': tone === 'warning',
      'border-cyan-300/20 bg-cyan-400/10 text-cyan-100': tone === 'info',
      'border-slate-500/20 bg-slate-500/10 text-slate-300': tone === 'muted',
      'border-violet-300/20 bg-violet-400/10 text-violet-100': tone === 'pending',
    }"
  >
    <slot />
  </span>
</template>

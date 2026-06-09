<script setup lang="ts">
withDefaults(
  defineProps<{
    message: string
    tone?: 'success' | 'danger' | 'warning' | 'info' | 'default'
  }>(),
  { tone: 'default' },
)
</script>

<template>
  <div
    class="rounded-2xl border px-4 py-3 text-sm"
    role="alert"
    :class="{
      'border-emerald-300/20 bg-emerald-400/10 text-emerald-100': tone === 'success',
      'border-red-300/20 bg-red-400/10 text-red-100': tone === 'danger',
      'border-amber-300/20 bg-amber-400/10 text-amber-100': tone === 'warning',
      'border-cyan-300/20 bg-cyan-400/10 text-cyan-100': tone === 'info',
      'border-white/10 bg-white/5 text-slate-200': tone === 'default',
    }"
  >
    <p>{{ message }}</p>
    <div v-if="$slots.default" class="mt-3">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  eyebrow?: string
  title: string
  description?: string
}>()
</script>

<template>
  <div class="mb-6 flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
    <div>
      <p v-if="eyebrow" class="mb-2 text-xs font-bold uppercase tracking-[0.25em] text-cyan-200/80">{{ eyebrow }}</p>
      <h1 class="text-3xl font-black tracking-tight text-white md:text-4xl">{{ title }}</h1>
      <p v-if="description" class="mt-2 max-w-2xl text-sm text-slate-400">{{ description }}</p>
    </div>
    <slot />
  </div>
</template>

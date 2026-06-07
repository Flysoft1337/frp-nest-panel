<script setup lang="ts">
defineProps<{
  tone?: 'default' | 'success' | 'danger'
}>()
</script>

<template>
  <span class="status-pill" :class="{ 'status-success': tone === 'success', 'status-danger': tone === 'danger' }">
    <slot />
  </span>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string
    note?: string
  }>(),
  { note: '' },
)
</script>

<template>
  <label>
    <span>{{ label }}</span>
    <slot />
    <span v-if="note" class="field-note">{{ note }}</span>
  </label>
</template>

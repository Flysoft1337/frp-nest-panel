<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    message: string
    busy?: boolean
    className?: string
  }>(),
  { className: 'btn-danger' },
)

const emit = defineEmits<{ confirm: [] }>()

function onClick() {
  if (window.confirm(props.message)) {
    emit('confirm')
  }
}
</script>

<template>
  <button :class="className" :disabled="busy" type="button" @click="onClick">
    <slot />
  </button>
</template>

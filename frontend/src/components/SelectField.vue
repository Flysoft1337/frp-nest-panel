<script setup lang="ts">
import { computed, ref } from 'vue'

const props = defineProps<{
  modelValue: string | boolean | null
  options: { label: string; value: string | boolean | null; disabled?: boolean }[]
  placeholder?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string | boolean | null]
}>()

const open = ref(false)
const selected = computed(() => props.options.find((option) => option.value === props.modelValue))

function choose(value: string | boolean | null, disabled?: boolean) {
  if (disabled) return
  emit('update:modelValue', value)
  open.value = false
}
</script>

<template>
  <div class="relative">
    <button class="select-trigger" type="button" @blur="open = false" @click="open = !open">
      <span :class="selected ? 'text-slate-100' : 'text-slate-500'">{{ selected?.label ?? placeholder ?? '请选择' }}</span>
      <span class="text-slate-400 transition" :class="open ? 'rotate-180' : ''">⌄</span>
    </button>
    <div v-if="open" class="select-menu">
      <button
        v-for="option in options"
        :key="String(option.value)"
        class="select-option"
        :class="{ 'select-option-active': option.value === modelValue, 'cursor-not-allowed opacity-40': option.disabled }"
        type="button"
        @mousedown.prevent="choose(option.value, option.disabled)"
      >
        {{ option.label }}
      </button>
    </div>
  </div>
</template>

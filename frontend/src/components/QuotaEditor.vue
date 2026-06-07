<script setup lang="ts">
const value = defineModel<number | null>({ required: true })

defineProps<{
  effective: number
}>()

const emit = defineEmits<{
  save: []
}>()

function increment() {
  value.value = (value.value || 0) + 1
}

function decrement() {
  if (!value.value || value.value <= 1) {
    value.value = null
    return
  }
  value.value -= 1
}
</script>

<template>
  <div class="grid gap-1">
    <div class="inline-flex w-fit items-center rounded-2xl border border-white/10 bg-white/5 p-1">
      <button class="rounded-xl px-3 py-2 text-slate-300 hover:bg-white/10" type="button" @click="decrement">−</button>
      <input v-model="value" class="quota-input" min="1" placeholder="默认" type="number" />
      <button class="rounded-xl px-3 py-2 text-slate-300 hover:bg-white/10" type="button" @click="increment">+</button>
      <button class="ml-1 rounded-xl bg-cyan-300 px-3 py-2 text-sm font-bold text-slate-950 hover:bg-cyan-200" type="button" @click="emit('save')">保存</button>
    </div>
    <span class="text-xs text-slate-500">当前有效：{{ effective }}</span>
  </div>
</template>

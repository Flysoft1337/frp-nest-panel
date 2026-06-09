<script setup lang="ts">
import { computed } from 'vue'

import type { TrafficHistoryPoint } from '../api/types'

const props = defineProps<{
  points: TrafficHistoryPoint[]
  loading?: boolean
}>()

const width = 720
const height = 220
const padding = 24

const deltas = computed(() => {
  return props.points.slice(1).map((point, index) => {
    const previous = props.points[index]
    return {
      sampled_at: point.sampled_at,
      traffic_in: Math.max(0, point.traffic_in - previous.traffic_in),
      traffic_out: Math.max(0, point.traffic_out - previous.traffic_out),
    }
  })
})

const maxValue = computed(() => Math.max(1, ...deltas.value.flatMap((point) => [point.traffic_in, point.traffic_out])))

function linePath(key: 'traffic_in' | 'traffic_out') {
  if (deltas.value.length === 0) return ''
  const innerWidth = width - padding * 2
  const innerHeight = height - padding * 2
  return deltas.value
    .map((point, index) => {
      const x = padding + (innerWidth * index) / Math.max(1, deltas.value.length - 1)
      const y = padding + innerHeight - (innerHeight * point[key]) / maxValue.value
      return `${index === 0 ? 'M' : 'L'} ${x.toFixed(2)} ${y.toFixed(2)}`
    })
    .join(' ')
}

function formatBytes(value: number) {
  if (value >= 1024 ** 3) return `${(value / 1024 ** 3).toFixed(2)} GiB`
  if (value >= 1024 ** 2) return `${(value / 1024 ** 2).toFixed(2)} MiB`
  if (value >= 1024) return `${(value / 1024).toFixed(2)} KiB`
  return `${value} B`
}

const totalIn = computed(() => deltas.value.reduce((sum, point) => sum + point.traffic_in, 0))
const totalOut = computed(() => deltas.value.reduce((sum, point) => sum + point.traffic_out, 0))
</script>

<template>
  <div class="rounded-3xl border border-white/10 bg-slate-950/40 p-4">
    <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
      <div>
        <p class="text-sm font-semibold text-white">流量趋势</p>
        <p class="text-xs text-slate-400">按相邻采样累计值计算增量</p>
      </div>
      <div class="flex gap-3 text-xs text-slate-300">
        <span class="inline-flex items-center gap-1"><i class="h-2 w-2 rounded-full bg-cyan-300" />入站 {{ formatBytes(totalIn) }}</span>
        <span class="inline-flex items-center gap-1"><i class="h-2 w-2 rounded-full bg-fuchsia-300" />出站 {{ formatBytes(totalOut) }}</span>
      </div>
    </div>

    <div v-if="loading" class="flex h-48 items-center justify-center text-sm text-slate-400">正在加载流量历史</div>
    <div v-else-if="deltas.length === 0" class="flex h-48 items-center justify-center rounded-2xl border border-dashed border-white/10 text-sm text-slate-500">
      暂无足够采样数据
    </div>
    <svg v-else class="h-56 w-full overflow-visible" :viewBox="`0 0 ${width} ${height}`" role="img" aria-label="流量趋势图">
      <defs>
        <linearGradient id="traffic-grid" x1="0" x2="0" y1="0" y2="1">
          <stop offset="0%" stop-color="rgba(255,255,255,0.12)" />
          <stop offset="100%" stop-color="rgba(255,255,255,0.03)" />
        </linearGradient>
      </defs>
      <line v-for="i in 5" :key="i" :x1="padding" :x2="width - padding" :y1="padding + ((height - padding * 2) * (i - 1)) / 4" :y2="padding + ((height - padding * 2) * (i - 1)) / 4" stroke="rgba(255,255,255,0.08)" />
      <path :d="linePath('traffic_in')" fill="none" stroke="#67e8f9" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" />
      <path :d="linePath('traffic_out')" fill="none" stroke="#f0abfc" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" />
      <text :x="padding" y="16" fill="rgba(226,232,240,0.7)" font-size="12">峰值 {{ formatBytes(maxValue) }}</text>
    </svg>
  </div>
</template>

<script setup lang="ts">
import type { Tunnel } from '../api/types'

const props = defineProps<{
  tunnel?: Tunnel | null
  compact?: boolean
}>()

function formatTime(value: string | null) {
  if (!value) return '尚未应用'
  return new Date(value).toLocaleString()
}
</script>

<template>
  <div
    v-if="props.tunnel"
    class="rounded-3xl border p-4"
    :class="props.tunnel.config_stale
      ? 'border-amber-300/30 bg-amber-300/10 text-amber-100'
      : 'border-cyan-300/20 bg-cyan-300/10 text-cyan-100'"
  >
    <div class="flex flex-wrap items-start justify-between gap-3">
      <div>
        <p class="text-sm font-semibold">
          {{ props.tunnel.config_stale ? '配置待更新' : '配置已预览或下载' }}
        </p>
        <p class="mt-1 text-sm text-slate-300">
          {{ props.tunnel.config_stale ? '隧道配置已变更，请重新复制或下载 frpc 配置，并重启或 reload frpc。' : '把当前配置应用到 frpc 后，需要重启或 reload frpc 才会生效。' }}
        </p>
      </div>
      <span class="rounded-full border border-white/10 bg-white/10 px-3 py-1 text-xs font-semibold text-slate-100">
        v{{ props.tunnel.config_version }}
      </span>
    </div>
    <div v-if="!props.compact" class="mt-3 grid gap-2 text-xs text-slate-400 sm:grid-cols-3">
      <span>变更：{{ formatTime(props.tunnel.config_changed_at) }}</span>
      <span>预览：{{ formatTime(props.tunnel.last_config_viewed_at) }}</span>
      <span>下载：{{ formatTime(props.tunnel.last_config_downloaded_at) }}</span>
    </div>
  </div>
</template>

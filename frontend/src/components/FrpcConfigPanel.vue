<script setup lang="ts">
import { computed, ref } from 'vue'

import type { FrpcResponse } from '../api/types'
import StatusPill from './StatusPill.vue'

const props = defineProps<{
  data: FrpcResponse | null
  title?: string
  downloadHref?: string
  bundleHref?: string | null
  loading?: boolean
}>()

const copyStatus = ref('')

const entry = computed(() => {
  if (!props.data) return '未加载'
  if (props.data.tunnel.remote_port) return String(props.data.tunnel.remote_port)
  if (props.data.tunnel.custom_domain) {
    return props.data.tunnel.custom_domain
      .split(',')
      .map((domain) => `${props.data?.tunnel.protocol}://${domain.trim()}`)
      .join('\n')
  }
  return '未配置'
})

async function copy() {
  if (!props.data) return
  try {
    await navigator.clipboard.writeText(props.data.frpc_toml)
    copyStatus.value = '已复制'
  } catch {
    copyStatus.value = '复制失败，请手动选择文本复制'
  }
}
</script>

<template>
  <div class="grid gap-5">
    <div v-if="loading" class="grid gap-3">
      <div class="h-24 animate-pulse rounded-3xl bg-white/[0.04]" />
      <div class="h-96 animate-pulse rounded-3xl bg-white/[0.04]" />
    </div>

    <template v-else-if="data">
      <div class="grid gap-3 md:grid-cols-3">
        <div class="rounded-2xl border border-white/10 bg-white/[0.04] px-4 py-3">
          <div class="text-xs text-slate-500">协议</div>
          <div class="mt-2"><StatusPill>{{ data.tunnel.protocol.toUpperCase() }}</StatusPill></div>
        </div>
        <div class="rounded-2xl border border-white/10 bg-white/[0.04] px-4 py-3">
          <div class="text-xs text-slate-500">本地服务</div>
          <code class="mt-2 block truncate text-slate-200">{{ data.tunnel.local_host }}:{{ data.tunnel.local_port }}</code>
        </div>
        <div class="rounded-2xl border border-cyan-300/10 bg-cyan-300/[0.04] px-4 py-3">
          <div class="text-xs text-slate-500">入口</div>
          <code class="mt-2 block whitespace-pre-wrap break-all text-cyan-100">{{ entry }}</code>
        </div>
      </div>

      <div class="rounded-3xl border border-cyan-300/20 bg-cyan-300/[0.06] p-4 text-sm text-cyan-50">
        <div class="font-bold">在用户自己的机器上运行 frpc。</div>
        <div class="mt-2 text-cyan-100/80">保存下面的 frpc.toml 到 frpc 同目录，然后执行 frpc -c frpc.toml。上传证书模式请下载配置包。</div>
      </div>

      <div class="flex flex-wrap items-center gap-3">
        <button class="btn-primary" type="button" @click="copy">复制配置</button>
        <a v-if="downloadHref" class="btn-secondary" role="button" :href="downloadHref">下载 frpc.toml</a>
        <a v-if="bundleHref" class="btn-secondary" role="button" :href="bundleHref">下载配置包</a>
        <span class="text-sm text-slate-400">{{ copyStatus }}</span>
      </div>

      <div class="overflow-hidden rounded-3xl border border-white/10 bg-slate-950/60">
        <div class="flex items-center justify-between border-b border-white/10 px-4 py-3 text-xs text-slate-400">
          <span>{{ title || 'frpc.toml' }}</span>
          <span>{{ data.frpc_toml.length }} chars</span>
        </div>
        <textarea class="min-h-80 max-h-[60vh] rounded-none border-0 bg-transparent font-mono text-sm leading-7 focus:ring-0" readonly :value="data.frpc_toml" />
      </div>
    </template>
  </div>
</template>

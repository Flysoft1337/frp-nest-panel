<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'

import { getFrpc } from '../api/tunnels'
import type { FrpcResponse } from '../api/types'
import PageHeader from '../components/PageHeader.vue'
import StatusPill from '../components/StatusPill.vue'

const route = useRoute()
const data = ref<FrpcResponse | null>(null)
const error = ref('')
const copyStatus = ref('')

function tunnelEntry(value: FrpcResponse) {
  if (value.tunnel.remote_port) return String(value.tunnel.remote_port)
  if (value.tunnel.custom_domain) {
    return value.tunnel.custom_domain
      .split(',')
      .map((domain) => `${value.tunnel.protocol}://${domain.trim()}`)
      .join('\n')
  }
  return '未配置'
}

async function load() {
  try {
    data.value = await getFrpc(route.params.id as string)
  } catch (err) {
    error.value = err instanceof Error ? err.message : '加载失败'
  }
}

async function copy() {
  if (!data.value) return
  try {
    await navigator.clipboard.writeText(data.value.frpc_toml)
    copyStatus.value = '已复制'
  } catch {
    copyStatus.value = '复制失败，请手动选择文本复制'
  }
}

onMounted(load)
</script>

<template>
  <PageHeader eyebrow="frpc.toml" :title="data?.tunnel.name || '配置预览'" description="复制或下载这个隧道的 frpc 配置。">
    <a v-if="data" class="btn-primary" role="button" :href="`/tunnels/${data.tunnel.id}/frpc.toml`">下载</a>
    <a v-if="data?.tunnel.tls_mode === 'uploaded_cert'" class="btn-secondary" role="button" :href="`/tunnels/${data.tunnel.id}/frpc.zip`">下载配置包</a>
  </PageHeader>

  <section class="card p-6">
    <p v-if="error" class="rounded-2xl border border-red-300/20 bg-red-400/10 px-4 py-3 text-sm text-red-100">{{ error }}</p>
    <div v-if="data" class="grid gap-5">
      <div class="grid gap-3 md:grid-cols-3">
        <div class="rounded-2xl border border-white/10 bg-white/[0.04] px-4 py-3">
          <div class="text-xs text-slate-500">协议</div>
          <div class="mt-2"><StatusPill>{{ data.tunnel.protocol }}</StatusPill></div>
        </div>
        <div class="rounded-2xl border border-white/10 bg-white/[0.04] px-4 py-3">
          <div class="text-xs text-slate-500">本地服务</div>
          <code class="mt-2 block truncate text-slate-200">{{ data.tunnel.local_host }}:{{ data.tunnel.local_port }}</code>
        </div>
        <div class="rounded-2xl border border-cyan-300/10 bg-cyan-300/[0.04] px-4 py-3">
          <div class="text-xs text-slate-500">入口</div>
          <code class="mt-2 block whitespace-pre-wrap break-all text-cyan-100">{{ tunnelEntry(data) }}</code>
        </div>
      </div>
      <div class="flex flex-wrap items-center gap-3">
        <button class="btn-primary" type="button" @click="copy">复制配置</button>
        <span class="text-sm text-slate-400">{{ copyStatus }}</span>
      </div>
      <div class="overflow-hidden rounded-3xl border border-white/10 bg-slate-950/60">
        <div class="flex items-center justify-between border-b border-white/10 px-4 py-3 text-xs text-slate-400">
          <span>frpc.toml</span>
          <span>{{ data.frpc_toml.length }} chars</span>
        </div>
        <textarea class="min-h-96 rounded-none border-0 bg-transparent font-mono text-sm leading-7 focus:ring-0" readonly :value="data.frpc_toml" />
      </div>
    </div>
  </section>
</template>

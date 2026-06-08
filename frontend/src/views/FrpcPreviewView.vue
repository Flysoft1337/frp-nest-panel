<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'

import { getFrpc } from '../api/tunnels'
import type { FrpcResponse } from '../api/types'
import PageHeader from '../components/PageHeader.vue'

const route = useRoute()
const data = ref<FrpcResponse | null>(null)
const error = ref('')
const copyStatus = ref('')

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
    <div v-if="data" class="grid gap-4">
      <div class="flex flex-wrap items-center gap-3">
        <button class="btn-primary" type="button" @click="copy">复制配置</button>
        <span class="text-sm text-slate-400">{{ copyStatus }}</span>
      </div>
      <textarea class="min-h-96 font-mono text-sm leading-7" readonly :value="data.frpc_toml" />
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'

import { getFrpc } from '../api/tunnels'
import type { FrpcResponse } from '../api/types'
import AlertBox from '../components/AlertBox.vue'
import ConfigChangeAlert from '../components/ConfigChangeAlert.vue'
import FrpcConfigPanel from '../components/FrpcConfigPanel.vue'
import PageHeader from '../components/PageHeader.vue'

const route = useRoute()
const data = ref<FrpcResponse | null>(null)
const error = ref('')
const loading = ref(true)

async function load() {
  loading.value = true
  error.value = ''
  try {
    data.value = await getFrpc(route.params.id as string)
  } catch (err) {
    error.value = err instanceof Error ? err.message : '加载失败'
  } finally {
    loading.value = false
  }
}

onMounted(load)
</script>

<template>
  <PageHeader eyebrow="frpc.toml" :title="data?.tunnel.name || '配置预览'" description="复制、下载并在用户自己的机器上运行这个隧道的 frpc 配置。" />

  <section class="card p-6">
    <AlertBox v-if="error" tone="danger" :message="error" />
    <ConfigChangeAlert v-if="data" class="mb-4" :tunnel="data.tunnel" />
    <FrpcConfigPanel
      :data="data"
      :loading="loading"
      :download-href="data ? `/tunnels/${data.tunnel.id}/frpc.toml` : undefined"
      :bundle-href="data?.tunnel.tls_mode === 'uploaded_cert' ? `/tunnels/${data.tunnel.id}/frpc.zip` : null"
    />
  </section>
</template>

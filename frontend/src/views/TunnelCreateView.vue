<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { createTunnel, getTunnel, updateTunnel } from '../api/tunnels'
import PageHeader from '../components/PageHeader.vue'

const route = useRoute()
const router = useRouter()
const tunnelId = computed(() => route.params.id as string | undefined)
const isEdit = computed(() => Boolean(tunnelId.value))

const name = ref('')
const protocol = ref('tcp')
const localHost = ref('127.0.0.1')
const localPort = ref<number | null>(null)
const remotePort = ref<number | null>(null)
const error = ref('')
const loading = ref(false)
const loadingTunnel = ref(false)

async function loadTunnel() {
  if (!tunnelId.value) return
  loadingTunnel.value = true
  error.value = ''
  try {
    const tunnel = await getTunnel(tunnelId.value)
    name.value = tunnel.name
    protocol.value = tunnel.protocol
    localHost.value = tunnel.local_host
    localPort.value = tunnel.local_port
    remotePort.value = tunnel.remote_port
  } catch (err) {
    error.value = err instanceof Error ? err.message : '加载失败'
  } finally {
    loadingTunnel.value = false
  }
}

async function submit() {
  if (!localPort.value) return
  error.value = ''
  loading.value = true
  try {
    const input = {
      name: name.value,
      protocol: protocol.value,
      local_host: localHost.value,
      local_port: localPort.value,
    }
    if (tunnelId.value) {
      await updateTunnel(tunnelId.value, input)
    } else {
      await createTunnel(input)
    }
    await router.push('/dashboard')
  } catch (err) {
    error.value = err instanceof Error ? err.message : isEdit.value ? '保存失败' : '创建失败'
  } finally {
    loading.value = false
  }
}

onMounted(loadTunnel)
</script>

<template>
  <PageHeader
    eyebrow="Tunnel"
    :title="isEdit ? '编辑隧道' : '创建隧道'"
    :description="isEdit ? '修改隧道名称、协议和本地服务地址。远程端口保持不变。' : '选择 TCP 或 UDP，并填写本地服务地址。'"
  />
  <section class="card max-w-2xl p-6">
    <p v-if="loadingTunnel" class="mb-4 text-sm text-slate-400">加载中</p>
    <form class="grid gap-5" @submit.prevent="submit">
      <label>隧道名称<input v-model="name" placeholder="mc-server" required /></label>
      <label>协议
        <select v-model="protocol" required>
          <option value="tcp">TCP</option>
          <option value="udp">UDP</option>
        </select>
      </label>
      <div class="grid gap-5 md:grid-cols-2">
        <label>本地地址<input v-model="localHost" required /></label>
        <label>本地端口<input v-model="localPort" max="65535" min="1" required type="number" /></label>
      </div>
      <div v-if="isEdit && remotePort" class="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-slate-300">
        远程端口保持不变：<code class="text-cyan-100">{{ remotePort }}</code>
      </div>
      <p v-if="error" class="rounded-2xl border border-red-300/20 bg-red-400/10 px-4 py-3 text-sm text-red-100">{{ error }}</p>
      <div class="flex flex-wrap gap-3">
        <button class="btn-primary" :disabled="loading || loadingTunnel" type="submit">{{ loading ? '保存中' : isEdit ? '保存修改' : '创建' }}</button>
        <RouterLink class="btn-secondary" role="button" to="/dashboard">返回</RouterLink>
      </div>
    </form>
  </section>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'

import { createTunnel } from '../api/tunnels'
import PageHeader from '../components/PageHeader.vue'

const router = useRouter()
const name = ref('')
const protocol = ref('tcp')
const localHost = ref('127.0.0.1')
const localPort = ref<number | null>(null)
const error = ref('')
const loading = ref(false)

async function submit() {
  if (!localPort.value) return
  error.value = ''
  loading.value = true
  try {
    await createTunnel({
      name: name.value,
      protocol: protocol.value,
      local_host: localHost.value,
      local_port: localPort.value,
    })
    await router.push('/dashboard')
  } catch (err) {
    error.value = err instanceof Error ? err.message : '创建失败'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <PageHeader eyebrow="Tunnel" title="创建隧道" description="选择 TCP 或 UDP，并填写本地服务地址。" />
  <section class="card max-w-2xl p-6">
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
      <p v-if="error" class="rounded-2xl border border-red-300/20 bg-red-400/10 px-4 py-3 text-sm text-red-100">{{ error }}</p>
      <div class="flex flex-wrap gap-3">
        <button class="btn-primary" :disabled="loading" type="submit">{{ loading ? '创建中' : '创建' }}</button>
        <RouterLink class="btn-secondary" role="button" to="/dashboard">返回</RouterLink>
      </div>
    </form>
  </section>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'

import { getFrps, restartFrps, updateFrps } from '../api/admin'
import type { FrpsStatus } from '../api/types'
import ConfirmButton from '../components/ConfirmButton.vue'
import PageHeader from '../components/PageHeader.vue'
import StatusPill from '../components/StatusPill.vue'

const status = ref<FrpsStatus | null>(null)
const form = reactive({
  server_addr: '',
  bind_port: 7000,
  auth_token: '',
  remote_port_min: 20000,
  remote_port_max: 30000,
})
const error = ref('')
const message = ref('')

async function load() {
  const data = await getFrps()
  status.value = data
  form.server_addr = data.server_addr
  form.bind_port = data.bind_port
  form.auth_token = ''
  form.remote_port_min = data.remote_port_min
  form.remote_port_max = data.remote_port_max
}

async function save() {
  error.value = ''
  message.value = ''
  try {
    await updateFrps({ ...form })
    await load()
    message.value = '配置已保存，需重启 frps 生效'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '保存失败'
  }
}

async function restart() {
  error.value = ''
  message.value = ''
  try {
    await restartFrps()
    await load()
    message.value = 'frps 已提交重启'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '重启失败'
  }
}

onMounted(async () => {
  try {
    await load()
  } catch (err) {
    error.value = err instanceof Error ? err.message : '加载失败'
  }
})
</script>

<template>
  <PageHeader eyebrow="Admin" title="frps 管理" description="编辑本机 frps 配置。保存不会自动重启。" />

  <section v-if="status" class="card p-6">
    <div class="grid gap-4 md:grid-cols-4">
      <div>
        <div class="text-sm text-slate-400">状态</div>
        <div class="mt-2"><StatusPill>{{ status.status }}</StatusPill></div>
      </div>
      <div>
        <div class="text-sm text-slate-400">配置文件</div>
        <code class="mt-2 block text-cyan-100">{{ status.config_path }}</code>
      </div>
      <div>
        <div class="text-sm text-slate-400">Token</div>
        <div class="mt-2 text-white">{{ status.token_set ? '已设置' : '未设置' }}</div>
      </div>
      <div>
        <div class="text-sm text-slate-400">重启命令</div>
        <div class="mt-2 text-white">{{ status.restart_command_configured ? '已配置' : '未配置' }}</div>
      </div>
    </div>
  </section>

  <section class="card mt-6 p-6">
    <p v-if="error" class="mb-4 rounded-2xl border border-red-300/20 bg-red-400/10 px-4 py-3 text-sm text-red-100">{{ error }}</p>
    <p v-if="message" class="mb-4 rounded-2xl border border-emerald-300/20 bg-emerald-400/10 px-4 py-3 text-sm text-emerald-100">{{ message }}</p>

    <form class="grid gap-4" @submit.prevent="save">
      <label>frps 地址<input v-model="form.server_addr" required placeholder="example.com" /></label>
      <label>bindPort<input v-model="form.bind_port" max="65535" min="1" required type="number" /></label>
      <label>Token<input v-model="form.auth_token" autocomplete="new-password" placeholder="留空表示不修改" type="password" /></label>
      <div class="grid gap-4 md:grid-cols-2">
        <label>远程端口最小值<input v-model="form.remote_port_min" max="65535" min="1" required type="number" /></label>
        <label>远程端口最大值<input v-model="form.remote_port_max" max="65535" min="1" required type="number" /></label>
      </div>
      <div class="flex flex-wrap gap-3">
        <button class="btn-primary" type="submit">保存配置</button>
        <ConfirmButton class-name="btn-danger" message="重启 frps 会影响所有隧道连接，确定继续吗？" @confirm="restart">重启 frps</ConfirmButton>
      </div>
    </form>
  </section>
</template>

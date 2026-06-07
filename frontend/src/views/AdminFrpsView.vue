<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'

import { getFrps, restartFrps, updateFrps } from '../api/admin'
import type { FrpsStatus } from '../api/types'
import AdminNav from '../components/AdminNav.vue'
import AlertBox from '../components/AlertBox.vue'
import ConfirmButton from '../components/ConfirmButton.vue'
import FormField from '../components/FormField.vue'
import PageHeader from '../components/PageHeader.vue'
import StatCard from '../components/StatCard.vue'
import StatusPill from '../components/StatusPill.vue'

const status = ref<FrpsStatus | null>(null)
const form = reactive({
  server_addr: '',
  bind_port: 7000,
  auth_token: '',
  remote_port_min: 20000,
  remote_port_max: 30000,
  dashboard_addr: '127.0.0.1',
  dashboard_port: null as number | null,
  dashboard_user: '',
  dashboard_password: '',
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
  form.dashboard_addr = data.dashboard_addr
  form.dashboard_port = data.dashboard_port
  form.dashboard_user = data.dashboard_user
  form.dashboard_password = ''
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
    await pollStatus()
    message.value = 'frps 已重启'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '重启失败'
  }
}

async function pollStatus() {
  for (let index = 0; index < 10; index += 1) {
    await load()
    if (!status.value?.restarting) return
    await new Promise((resolve) => window.setTimeout(resolve, 1000))
  }
}

function statusTone(value: FrpsStatus) {
  if (value.state === 'running') return 'success'
  if (value.state === 'stopped') return 'danger'
  return 'default'
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
  <AdminNav />

  <section v-if="status" class="grid gap-4 md:grid-cols-2 xl:grid-cols-6">
    <div class="card p-5">
      <div class="text-sm text-slate-400">状态</div>
      <div class="mt-2"><StatusPill :tone="statusTone(status)">{{ status.display_status }}</StatusPill></div>
    </div>
    <StatCard label="版本" :value="status.version" />
    <StatCard label="配置文件" :value="status.config_path" />
    <StatCard label="Token" :value="status.token_set ? '已设置' : '未设置'" />
    <div class="card p-5 md:col-span-2 xl:col-span-2">
      <div class="text-sm text-slate-400">Dashboard</div>
      <div class="mt-2"><StatusPill :tone="status.dashboard_available ? 'success' : 'default'">{{ status.dashboard_available ? '可用' : status.dashboard_configured ? '不可用' : '未配置' }}</StatusPill></div>
    </div>
  </section>

  <section class="card mt-6 p-6">
    <div class="mb-4 grid gap-3">
      <AlertBox v-if="error" tone="danger" :message="error" />
      <AlertBox v-if="message" tone="success" :message="message" />
    </div>

    <form class="grid gap-5" @submit.prevent="save">
      <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-4">
        <div class="mb-4 flex flex-col gap-1">
          <h2 class="text-lg font-bold text-white">基础配置</h2>
          <p class="text-sm text-slate-400">面板生成 frpc 配置时会使用这里的地址和 bindPort。</p>
        </div>
        <div class="grid gap-4 md:grid-cols-2">
          <FormField label="frps 地址"><input v-model="form.server_addr" required placeholder="example.com" /></FormField>
          <FormField label="bindPort"><input v-model="form.bind_port" max="65535" min="1" required type="number" /></FormField>
          <FormField label="Token" note="留空表示不修改"><input v-model="form.auth_token" autocomplete="new-password" placeholder="留空表示不修改" type="password" /></FormField>
        </div>
      </section>

      <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-4">
        <div class="mb-4 flex flex-col gap-1">
          <h2 class="text-lg font-bold text-white">远程端口范围</h2>
          <p class="text-sm text-slate-400">创建隧道时只能使用这个范围内的远程端口。</p>
        </div>
        <div class="grid gap-4 md:grid-cols-2">
          <FormField label="最小值"><input v-model="form.remote_port_min" max="65535" min="1" required type="number" /></FormField>
          <FormField label="最大值"><input v-model="form.remote_port_max" max="65535" min="1" required type="number" /></FormField>
        </div>
      </section>

      <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-4">
        <div class="mb-4 flex flex-col gap-1">
          <h2 class="text-lg font-bold text-white">Dashboard 流量数据源</h2>
          <p class="text-sm text-slate-400">启用后 frps 会写入 webServer 配置，重启 frps 后流量统计才会可用。</p>
        </div>
        <div class="grid gap-4 md:grid-cols-2">
          <FormField label="Dashboard 地址"><input v-model="form.dashboard_addr" placeholder="127.0.0.1" /></FormField>
          <FormField label="Dashboard 端口"><input v-model="form.dashboard_port" max="65535" min="1" placeholder="留空关闭" type="number" /></FormField>
          <FormField label="Dashboard 用户"><input v-model="form.dashboard_user" autocomplete="username" placeholder="admin" /></FormField>
          <FormField label="Dashboard 密码" note="留空表示不修改"><input v-model="form.dashboard_password" autocomplete="new-password" placeholder="留空表示不修改" type="password" /></FormField>
        </div>
      </section>

      <div class="flex flex-wrap gap-3">
        <button class="btn-primary" type="submit">保存配置</button>
        <ConfirmButton class-name="btn-danger" :busy="status?.restarting" :message="status?.restarting ? 'frps 正在重启' : '重启 frps 会影响所有隧道连接，确定继续吗？'" @confirm="restart">{{ status?.restarting ? '重启中' : '重启 frps' }}</ConfirmButton>
      </div>
    </form>
  </section>
</template>

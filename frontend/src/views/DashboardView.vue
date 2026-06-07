<script setup lang="ts">
import { onMounted, ref } from 'vue'

import { getDashboardSummary } from '../api/dashboard'
import { deleteTunnel, listTunnels } from '../api/tunnels'
import type { DashboardSummary, Tunnel } from '../api/types'
import ConfirmButton from '../components/ConfirmButton.vue'
import PageHeader from '../components/PageHeader.vue'
import StatusPill from '../components/StatusPill.vue'

const tunnels = ref<Tunnel[]>([])
const summary = ref<DashboardSummary | null>(null)
const loading = ref(true)
const error = ref('')

async function load() {
  loading.value = true
  error.value = ''
  try {
    const [tunnelList, dashboardSummary] = await Promise.all([
      listTunnels(),
      getDashboardSummary(),
    ])
    tunnels.value = tunnelList
    summary.value = dashboardSummary
  } catch (err) {
    error.value = err instanceof Error ? err.message : '加载失败'
  } finally {
    loading.value = false
  }
}

async function remove(id: string) {
  await deleteTunnel(id)
  await load()
}

onMounted(load)
</script>

<template>
  <PageHeader eyebrow="Dashboard" title="我的隧道" description="创建和管理你的 TCP/UDP frp 隧道。">
    <RouterLink class="btn-primary" role="button" to="/tunnels/new">创建隧道</RouterLink>
  </PageHeader>

  <section v-if="summary" class="mb-6 grid gap-4 md:grid-cols-2 xl:grid-cols-4">
    <div class="card p-5">
      <p class="text-sm text-slate-400">隧道配额</p>
      <div class="mt-2 text-2xl font-black text-white">{{ summary.tunnel_count }} / {{ summary.user_max_tunnels }}</div>
    </div>
    <div class="card p-5">
      <p class="text-sm text-slate-400">剩余可创建</p>
      <div class="mt-2 text-2xl font-black" :class="summary.remaining_tunnels > 0 ? 'text-emerald-200' : 'text-red-200'">{{ summary.remaining_tunnels }}</div>
    </div>
    <div class="card p-5">
      <p class="text-sm text-slate-400">frps 地址</p>
      <div class="mt-2 break-all font-mono text-lg font-bold text-cyan-100">{{ summary.frps_server_addr }}:{{ summary.frps_bind_port }}</div>
    </div>
    <div class="card p-5">
      <p class="text-sm text-slate-400">远程端口范围</p>
      <div class="mt-2 font-mono text-lg font-bold text-cyan-100">{{ summary.remote_port_min }}-{{ summary.remote_port_max }}</div>
    </div>
  </section>

  <section class="card p-6">
    <div class="mb-5 flex flex-wrap items-center gap-3">
      <span class="rounded-2xl border border-white/10 bg-white/5 px-4 py-2 text-sm text-slate-300">{{ tunnels.length }} 个隧道</span>
      <span v-if="loading" class="text-sm text-slate-500">加载中</span>
      <span v-if="summary?.remaining_tunnels === 0" class="rounded-2xl border border-amber-300/20 bg-amber-400/10 px-4 py-2 text-sm text-amber-100">隧道数量已达上限</span>
      <span v-if="error" class="text-sm text-red-200">{{ error }}</span>
    </div>

    <div v-if="!loading && tunnels.length === 0" class="grid place-items-center rounded-3xl border border-dashed border-white/10 py-16 text-center">
      <h2 class="text-xl font-bold text-white">还没有隧道</h2>
      <p class="mt-2 text-sm text-slate-400">创建一个 TCP 或 UDP 隧道后，这里会显示配置入口。</p>
      <RouterLink class="btn-primary mt-6" role="button" to="/tunnels/new">创建第一个隧道</RouterLink>
    </div>

    <div v-else class="table-wrap">
      <table class="data-table">
        <thead><tr><th>名称</th><th>协议</th><th>本地</th><th>远程端口</th><th>操作</th></tr></thead>
        <tbody>
          <tr v-for="tunnel in tunnels" :key="tunnel.id">
            <td class="font-semibold text-white">{{ tunnel.name }}</td>
            <td><StatusPill>{{ tunnel.protocol }}</StatusPill></td>
            <td><code class="text-slate-300">{{ tunnel.local_host }}:{{ tunnel.local_port }}</code></td>
            <td><code class="text-cyan-100">{{ tunnel.remote_port }}</code></td>
            <td>
              <div class="flex flex-wrap items-center gap-2">
                <RouterLink class="btn-secondary" role="button" :to="`/tunnels/${tunnel.id}/edit`">编辑</RouterLink>
                <RouterLink class="btn-secondary" role="button" :to="`/tunnels/${tunnel.id}/frpc`">预览</RouterLink>
                <a class="btn-secondary" role="button" :href="`/tunnels/${tunnel.id}/frpc.toml`">下载</a>
                <ConfirmButton message="确定删除这个隧道吗？" @confirm="remove(tunnel.id)">删除</ConfirmButton>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

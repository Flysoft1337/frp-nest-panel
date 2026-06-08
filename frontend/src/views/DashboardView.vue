<script setup lang="ts">
import { onMounted, ref } from 'vue'

import { getDashboardSummary } from '../api/dashboard'
import { deleteTunnel, listTunnels } from '../api/tunnels'
import type { DashboardSummary, TunnelWithTraffic } from '../api/types'
import ConfirmButton from '../components/ConfirmButton.vue'
import PageHeader from '../components/PageHeader.vue'
import StatusPill from '../components/StatusPill.vue'

const tunnels = ref<TunnelWithTraffic[]>([])
const summary = ref<DashboardSummary | null>(null)
const loading = ref(true)
const error = ref('')

function formatBytes(value: number) {
  if (value < 1024) return `${value} B`
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KiB`
  if (value < 1024 * 1024 * 1024) return `${(value / 1024 / 1024).toFixed(1)} MiB`
  return `${(value / 1024 / 1024 / 1024).toFixed(1)} GiB`
}

function customDomains(value: string | null) {
  return value?.split(',').map((domain) => domain.trim()).filter(Boolean) || []
}

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
  <PageHeader eyebrow="Dashboard" title="我的隧道" description="创建和管理 TCP、UDP、HTTP 和 HTTPS frp 隧道。">
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

    <div v-if="!loading && tunnels.length === 0" class="empty-state">
      <h2 class="text-xl font-bold text-white">还没有隧道</h2>
      <p class="mt-2 text-sm text-slate-400">创建一个 TCP 或 UDP 隧道后，这里会显示配置入口。</p>
      <RouterLink class="btn-primary mt-6" role="button" to="/tunnels/new">创建第一个隧道</RouterLink>
    </div>

    <div v-else class="grid gap-3">
      <article v-for="row in tunnels" :key="row.tunnel.id" class="rounded-3xl border border-white/10 bg-white/[0.04] p-4 transition hover:border-cyan-300/20 hover:bg-white/[0.06]">
        <div class="grid gap-4">
          <div class="flex flex-wrap items-start justify-between gap-3">
            <div class="min-w-0">
              <div class="flex flex-wrap items-center gap-2">
                <h2 class="truncate text-lg font-black text-white">{{ row.tunnel.name }}</h2>
                <StatusPill>{{ row.tunnel.protocol }}</StatusPill>
              </div>
              <div class="mt-2 text-sm text-slate-400">
                <code v-if="row.traffic_available" class="text-cyan-100">↓ {{ formatBytes(row.traffic_in) }} / ↑ {{ formatBytes(row.traffic_out) }}</code>
                <span v-else>流量暂无数据</span>
              </div>
            </div>
            <div class="flex flex-wrap gap-2">
              <RouterLink class="btn-secondary" role="button" :to="`/tunnels/${row.tunnel.id}/edit`">编辑</RouterLink>
              <RouterLink class="btn-secondary" role="button" :to="`/tunnels/${row.tunnel.id}/frpc`">预览</RouterLink>
              <a class="btn-secondary" role="button" :href="`/tunnels/${row.tunnel.id}/frpc.toml`">下载</a>
              <a v-if="row.tunnel.tls_mode === 'uploaded_cert'" class="btn-secondary" role="button" :href="`/tunnels/${row.tunnel.id}/frpc.zip`">配置包</a>
              <ConfirmButton message="确定删除这个隧道吗？" @confirm="remove(row.tunnel.id)">删除</ConfirmButton>
            </div>
          </div>

          <div class="grid gap-3 border-t border-white/10 pt-4 md:grid-cols-2">
            <div class="rounded-2xl border border-white/10 bg-slate-950/30 px-4 py-3">
              <div class="text-xs font-bold uppercase tracking-[0.18em] text-slate-500">Local</div>
              <code class="mt-2 block truncate text-slate-200">{{ row.tunnel.local_host }}:{{ row.tunnel.local_port }}</code>
            </div>
            <div class="rounded-2xl border border-cyan-300/10 bg-cyan-300/[0.04] px-4 py-3">
              <div class="text-xs font-bold uppercase tracking-[0.18em] text-slate-500">Entry</div>
              <code v-if="row.tunnel.remote_port" class="mt-2 block text-cyan-100">:{{ row.tunnel.remote_port }}</code>
              <div v-else-if="row.tunnel.custom_domain" class="mt-2 grid gap-1">
                <a v-for="domain in customDomains(row.tunnel.custom_domain)" :key="domain" class="block break-all text-cyan-100" :href="`${row.tunnel.protocol}://${domain}`" target="_blank">{{ row.tunnel.protocol }}://{{ domain }}</a>
              </div>
              <span v-else class="mt-2 block text-sm text-slate-500">未配置</span>
            </div>
          </div>
        </div>
      </article>
    </div>
  </section>
</template>

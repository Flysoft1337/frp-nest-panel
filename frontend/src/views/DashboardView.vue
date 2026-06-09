<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { getDashboardSummary } from '../api/dashboard'
import { deleteTunnel, getTunnelTrafficHistory, listTunnels } from '../api/tunnels'
import type { DashboardSummary, TrafficHistoryPoint, TunnelWithTraffic } from '../api/types'
import AlertBox from '../components/AlertBox.vue'
import ConfigChangeAlert from '../components/ConfigChangeAlert.vue'
import ConfirmButton from '../components/ConfirmButton.vue'
import PageHeader from '../components/PageHeader.vue'
import StatusPill from '../components/StatusPill.vue'
import TrafficChart from '../components/TrafficChart.vue'

const route = useRoute()
const router = useRouter()
const tunnels = ref<TunnelWithTraffic[]>([])
const summary = ref<DashboardSummary | null>(null)
const loading = ref(true)
const error = ref('')
const message = ref('')
const historyTunnelId = ref<string | null>(null)
const historyLoading = ref(false)
const historyError = ref('')
const historyPoints = ref<TrafficHistoryPoint[]>([])
const onlineCount = computed(() => tunnels.value.filter((row) => row.runtime_status !== 'offline').length)

function formatBytes(value: number) {
  if (value < 1024) return `${value} B`
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KiB`
  if (value < 1024 * 1024 * 1024) return `${(value / 1024 / 1024).toFixed(1)} MiB`
  return `${(value / 1024 / 1024 / 1024).toFixed(1)} GiB`
}

function customDomains(value: string | null) {
  return value?.split(',').map((domain) => domain.trim()).filter(Boolean) || []
}

function formatDateTime(value: string | null) {
  if (!value) return ''
  return new Date(value).toLocaleString()
}

function entryValue(row: TunnelWithTraffic) {
  if (row.tunnel.remote_port && summary.value) return `${summary.value.frps_server_addr}:${row.tunnel.remote_port}`
  const domains = customDomains(row.tunnel.custom_domain)
  if (domains.length > 0) return domains.map((domain) => `${row.tunnel.protocol}://${domain}`).join(' · ')
  return '未配置入口'
}

function advancedItems(row: TunnelWithTraffic) {
  const items: string[] = []
  if (row.tunnel.use_encryption) items.push('加密')
  if (row.tunnel.use_compression) items.push('压缩')
  if (row.tunnel.bandwidth_limit) items.push(`限速 ${row.tunnel.bandwidth_limit}${row.tunnel.bandwidth_limit_mode ? ` · ${row.tunnel.bandwidth_limit_mode}` : ''}`)
  if (row.tunnel.proxy_protocol_version) items.push(`Proxy Protocol ${row.tunnel.proxy_protocol_version}`)
  if (row.tunnel.locations) items.push(`路径 ${row.tunnel.locations.split(',').length}`)
  if (row.tunnel.host_header_rewrite) items.push(`Host ${row.tunnel.host_header_rewrite}`)
  return items
}

function statusTone(status: string): 'default' | 'success' | 'danger' {
  if (status === 'online' || status === 'running') return 'success'
  if (status === 'offline') return 'danger'
  return 'default'
}

function statusText(status: string) {
  if (status === 'online' || status === 'running') return '在线'
  if (status === 'offline') return '离线'
  return '未知'
}

async function toggleHistory(id: string) {
  if (historyTunnelId.value === id) {
    historyTunnelId.value = null
    historyPoints.value = []
    historyError.value = ''
    return
  }
  historyTunnelId.value = id
  historyLoading.value = true
  historyError.value = ''
  historyPoints.value = []
  try {
    historyPoints.value = (await getTunnelTrafficHistory(id)).points
  } catch (err) {
    historyError.value = err instanceof Error ? err.message : '历史流量加载失败'
  } finally {
    historyLoading.value = false
  }
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

onMounted(async () => {
  if (route.query.config === 'updated') {
    message.value = '隧道已保存。请重新复制或下载 frpc 配置，并重启或 reload frpc。'
    await router.replace('/dashboard')
  }
  await load()
})
</script>

<template>
  <PageHeader eyebrow="Dashboard" title="我的隧道" description="创建和管理 TCP、UDP、HTTP 和 HTTPS frp 隧道。">
    <RouterLink class="btn-primary" role="button" to="/tunnels/new">创建隧道</RouterLink>
  </PageHeader>

  <AlertBox v-if="message" class="mb-4" tone="success" :message="message" />

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
      <span class="rounded-2xl border border-emerald-300/20 bg-emerald-400/10 px-4 py-2 text-sm text-emerald-100">{{ onlineCount }} 个在线</span>
      <span v-if="loading" class="text-sm text-slate-500">加载中</span>
      <span v-if="summary?.remaining_tunnels === 0" class="rounded-2xl border border-amber-300/20 bg-amber-400/10 px-4 py-2 text-sm text-amber-100">隧道数量已达上限</span>
      <span v-if="error" class="rounded-2xl border border-red-300/20 bg-red-400/10 px-4 py-2 text-sm text-red-100">{{ error }}</span>
    </div>

    <div v-if="!loading && tunnels.length === 0" class="empty-state">
      <h2 class="text-xl font-bold text-white">还没有隧道</h2>
      <p class="mt-2 text-sm text-slate-400">创建一个 TCP 或 UDP 隧道后，这里会显示配置入口。</p>
      <RouterLink class="btn-primary mt-6" role="button" to="/tunnels/new">创建第一个隧道</RouterLink>
    </div>

    <div v-else class="grid gap-4">
      <article v-for="row in tunnels" :key="row.tunnel.id" class="rounded-3xl border border-white/10 bg-white/[0.04] p-4 transition hover:border-cyan-300/20 hover:bg-white/[0.06]">
        <div class="grid gap-4">
          <div class="flex flex-wrap items-start justify-between gap-3">
            <div class="min-w-0">
              <div class="flex flex-wrap items-center gap-2">
                <h2 class="truncate text-lg font-black text-white">{{ row.tunnel.name }}</h2>
                <StatusPill tone="default">{{ row.tunnel.protocol.toUpperCase() }}</StatusPill>
                <StatusPill :tone="statusTone(row.runtime_status)">{{ statusText(row.runtime_status) }}</StatusPill>
                <StatusPill v-if="row.tunnel.config_stale" tone="danger">配置待更新</StatusPill>
              </div>
              <div class="mt-2 grid gap-1 text-sm text-slate-400">
                <code class="break-all text-cyan-100">{{ entryValue(row) }}</code>
                <span>本地 {{ row.tunnel.local_host }}:{{ row.tunnel.local_port }}</span>
                <span v-if="row.matched_proxy_name" class="text-xs text-slate-500">frps proxy：{{ row.matched_proxy_name }}</span>
                <div v-if="advancedItems(row).length" class="mt-2 flex flex-wrap gap-2">
                  <span v-for="item in advancedItems(row)" :key="item" class="rounded-full border border-cyan-300/15 bg-cyan-300/[0.06] px-2.5 py-1 text-xs font-semibold text-cyan-100">{{ item }}</span>
                </div>
              </div>
            </div>
            <div class="flex flex-wrap gap-2">
              <RouterLink class="btn-secondary" role="button" :to="`/tunnels/${row.tunnel.id}/edit`">编辑</RouterLink>
              <RouterLink class="btn-secondary" role="button" :to="`/tunnels/${row.tunnel.id}/frpc`">复制配置</RouterLink>
              <button class="btn-secondary" type="button" @click="toggleHistory(row.tunnel.id)">{{ historyTunnelId === row.tunnel.id ? '收起流量' : '历史流量' }}</button>
              <a class="btn-secondary" role="button" :href="`/tunnels/${row.tunnel.id}/frpc.toml`">下载</a>
              <a v-if="row.tunnel.tls_mode === 'uploaded_cert'" class="btn-secondary" role="button" :href="`/tunnels/${row.tunnel.id}/frpc.zip`">配置包</a>
              <ConfirmButton message="确定删除这个隧道吗？" @confirm="remove(row.tunnel.id)">删除</ConfirmButton>
            </div>
          </div>

          <ConfigChangeAlert v-if="row.tunnel.config_stale" :tunnel="row.tunnel" compact />

          <div class="grid gap-3 border-t border-white/10 pt-4 md:grid-cols-3">
            <div class="rounded-2xl border border-white/10 bg-slate-950/30 px-4 py-3">
              <div class="text-xs font-bold uppercase tracking-[0.18em] text-slate-500">Status</div>
              <div class="mt-2 text-lg font-bold text-white">{{ row.current_connections }} 连接</div>
              <div class="mt-1 text-xs text-slate-500">{{ row.traffic_available ? 'Dashboard 实时数据可用' : '实时数据源未接入' }}</div>
            </div>
            <div class="rounded-2xl border border-cyan-300/10 bg-cyan-300/[0.04] px-4 py-3">
              <div class="text-xs font-bold uppercase tracking-[0.18em] text-slate-500">Persistent</div>
              <code v-if="row.persistent_traffic_available" class="mt-2 block text-cyan-100">↓ {{ formatBytes(row.persistent_traffic_in) }} / ↑ {{ formatBytes(row.persistent_traffic_out) }}</code>
              <span v-else class="mt-2 block text-sm text-slate-500">等待 Prometheus 采样</span>
              <span v-if="row.last_sampled_at" class="mt-1 block text-xs text-slate-500">{{ formatDateTime(row.last_sampled_at) }}</span>
            </div>
            <div class="rounded-2xl border border-white/10 bg-slate-950/30 px-4 py-3">
              <div class="text-xs font-bold uppercase tracking-[0.18em] text-slate-500">Realtime</div>
              <code v-if="row.traffic_available" class="mt-2 block text-slate-200">↓ {{ formatBytes(row.traffic_in) }} / ↑ {{ formatBytes(row.traffic_out) }}</code>
              <span v-else class="mt-2 block text-sm text-slate-500">未接入</span>
            </div>
          </div>

          <div v-if="historyTunnelId === row.tunnel.id" class="grid gap-3">
            <p v-if="historyError" class="rounded-2xl border border-red-300/20 bg-red-400/10 p-4 text-sm text-red-100">{{ historyError }}</p>
            <TrafficChart v-else :points="historyPoints" :loading="historyLoading" />
          </div>
        </div>
      </article>
    </div>
  </section>
</template>

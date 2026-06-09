<script setup lang="ts">
import { onMounted, ref } from 'vue'

import { getAdminSummary, getAdminTrafficSummary, getConfig } from '../api/admin'
import type { AdminSummary, AdminTrafficSummary, ConfigResponse } from '../api/types'
import AdminNav from '../components/AdminNav.vue'
import AlertBox from '../components/AlertBox.vue'
import PageHeader from '../components/PageHeader.vue'
import StatCard from '../components/StatCard.vue'
import StatusPill from '../components/StatusPill.vue'

const config = ref<ConfigResponse | null>(null)
const summary = ref<AdminSummary | null>(null)
const traffic = ref<AdminTrafficSummary | null>(null)
const error = ref('')

function formatBytes(value: number) {
  if (value < 1024) return `${value} B`
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KiB`
  if (value < 1024 * 1024 * 1024) return `${(value / 1024 / 1024).toFixed(1)} MiB`
  return `${(value / 1024 / 1024 / 1024).toFixed(1)} GiB`
}

function formatDateTime(value: string | null) {
  if (!value) return ''
  return new Date(value).toLocaleString()
}

onMounted(async () => {
  try {
    const [configData, summaryData, trafficData] = await Promise.all([getConfig(), getAdminSummary(), getAdminTrafficSummary()])
    config.value = configData
    summary.value = summaryData
    traffic.value = trafficData
  } catch (err) {
    error.value = err instanceof Error ? err.message : '加载失败'
  }
})
</script>

<template>
  <PageHeader eyebrow="Admin" title="管理后台" description="管理邀请码、用户、隧道和本机 frps。" />
  <AdminNav />

  <AlertBox v-if="error" class="mb-4" tone="danger" :message="error" />

  <section v-if="summary" class="grid gap-4 md:grid-cols-3 xl:grid-cols-6">
    <StatCard label="用户" :value="summary.user_count" />
    <StatCard label="禁用用户" :value="summary.disabled_user_count" />
    <StatCard label="隧道" :value="summary.tunnel_count" />
    <StatCard label="邀请码" :value="summary.invite_count" />
    <StatCard label="未用邀请码" :value="summary.unused_invite_count" />
    <StatCard label="端口占用" :value="`${summary.used_remote_port_count} / ${summary.remote_port_capacity}`" />
  </section>

  <section v-if="traffic" class="card mt-6 overflow-hidden p-0">
    <div class="border-b border-white/10 bg-cyan-300/[0.04] px-6 py-5">
      <div class="flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
        <div>
          <h2 class="text-xl font-bold text-white">总流量</h2>
          <p class="text-sm text-slate-400">
            <template v-if="traffic.persistent_available">来自 Prometheus 长期统计。最近采样 {{ formatDateTime(traffic.last_sampled_at) }}。</template>
            <template v-else-if="traffic.available">仅 frps dashboard 实时数据可用，长期统计等待采样。</template>
            <template v-else>frps dashboard / Prometheus 数据源未配置或不可用。</template>
          </p>
        </div>
        <StatusPill :tone="traffic.persistent_available ? 'success' : traffic.available ? 'success' : 'default'">{{ traffic.persistent_available ? '长期统计可用' : traffic.available ? '实时可用' : '未接入' }}</StatusPill>
      </div>
    </div>
    <div class="grid gap-3 p-6 text-sm md:grid-cols-2">
      <div class="rounded-3xl border border-white/10 bg-slate-950/30 px-5 py-4">
        <span class="text-slate-400">长期入站</span>
        <div class="mt-2 font-mono text-2xl font-black text-cyan-100">{{ formatBytes(traffic.persistent_total_traffic_in) }}</div>
        <div class="mt-2 text-xs text-slate-500">实时 {{ formatBytes(traffic.total_traffic_in) }}</div>
      </div>
      <div class="rounded-3xl border border-white/10 bg-slate-950/30 px-5 py-4">
        <span class="text-slate-400">长期出站</span>
        <div class="mt-2 font-mono text-2xl font-black text-cyan-100">{{ formatBytes(traffic.persistent_total_traffic_out) }}</div>
        <div class="mt-2 text-xs text-slate-500">实时 {{ formatBytes(traffic.total_traffic_out) }}</div>
      </div>
    </div>
  </section>

  <div class="mt-6 grid gap-4 md:grid-cols-4">
    <RouterLink class="card group block p-6 no-underline transition hover:-translate-y-1 hover:border-cyan-300/30 hover:bg-white/[0.08]" to="/admin/invites">
      <div class="flex h-10 w-10 items-center justify-center rounded-2xl bg-cyan-300/10 text-sm font-black text-cyan-100 transition group-hover:bg-cyan-300 group-hover:text-slate-950">邀</div>
      <div class="mt-4 text-lg font-bold text-white">邀请码</div>
      <p class="mt-2 text-sm text-slate-400">生成、筛选、复制和作废邀请码。</p>
    </RouterLink>
    <RouterLink class="card group block p-6 no-underline transition hover:-translate-y-1 hover:border-cyan-300/30 hover:bg-white/[0.08]" to="/admin/users">
      <div class="flex h-10 w-10 items-center justify-center rounded-2xl bg-cyan-300/10 text-sm font-black text-cyan-100 transition group-hover:bg-cyan-300 group-hover:text-slate-950">用</div>
      <div class="mt-4 text-lg font-bold text-white">用户</div>
      <p class="mt-2 text-sm text-slate-400">搜索、筛选、启禁用和重置密码。</p>
    </RouterLink>
    <RouterLink class="card group block p-6 no-underline transition hover:-translate-y-1 hover:border-cyan-300/30 hover:bg-white/[0.08]" to="/admin/tunnels">
      <div class="flex h-10 w-10 items-center justify-center rounded-2xl bg-cyan-300/10 text-sm font-black text-cyan-100 transition group-hover:bg-cyan-300 group-hover:text-slate-950">隧</div>
      <div class="mt-4 text-lg font-bold text-white">全部隧道</div>
      <p class="mt-2 text-sm text-slate-400">查看用户名、协议和远程端口。</p>
    </RouterLink>
    <RouterLink class="card group block p-6 no-underline transition hover:-translate-y-1 hover:border-cyan-300/30 hover:bg-white/[0.08]" to="/admin/frps">
      <div class="flex h-10 w-10 items-center justify-center rounded-2xl bg-cyan-300/10 text-sm font-black text-cyan-100 transition group-hover:bg-cyan-300 group-hover:text-slate-950">F</div>
      <div class="mt-4 text-lg font-bold text-white">frps 管理</div>
      <p class="mt-2 text-sm text-slate-400">编辑本机 frps 配置并手动重启。</p>
    </RouterLink>
  </div>

  <section v-if="config" class="card mt-6 p-6">
    <div class="mb-5 flex flex-wrap items-center justify-between gap-3">
      <div>
        <h2 class="text-xl font-bold text-white">当前配置</h2>
        <p class="mt-1 text-sm text-slate-400">这些值会影响用户创建隧道和下载 frpc 配置。</p>
      </div>
      <RouterLink class="btn-secondary" role="button" to="/admin/frps">编辑配置</RouterLink>
    </div>
    <dl class="grid gap-3 text-sm md:grid-cols-3">
      <div class="rounded-3xl border border-white/10 bg-white/[0.03] px-4 py-3">
        <dt class="text-slate-400">frps 地址</dt>
        <dd class="mt-2 break-all font-mono text-cyan-100">{{ config.frps_server_addr }}:{{ config.frps_bind_port }}</dd>
      </div>
      <div class="rounded-3xl border border-white/10 bg-white/[0.03] px-4 py-3">
        <dt class="text-slate-400">远程端口范围</dt>
        <dd class="mt-2 font-mono text-cyan-100">{{ config.remote_port_min }}-{{ config.remote_port_max }}</dd>
      </div>
      <div class="rounded-3xl border border-white/10 bg-white/[0.03] px-4 py-3">
        <dt class="text-slate-400">每个用户最多隧道数</dt>
        <dd class="mt-2 font-mono text-cyan-100">{{ config.user_max_tunnels }}</dd>
      </div>
    </dl>
  </section>
</template>

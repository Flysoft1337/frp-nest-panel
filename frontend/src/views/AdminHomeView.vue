<script setup lang="ts">
import { onMounted, ref } from 'vue'

import { getAdminSummary, getConfig } from '../api/admin'
import type { AdminSummary, ConfigResponse } from '../api/types'
import PageHeader from '../components/PageHeader.vue'

const config = ref<ConfigResponse | null>(null)
const summary = ref<AdminSummary | null>(null)
const error = ref('')

onMounted(async () => {
  try {
    const [configData, summaryData] = await Promise.all([getConfig(), getAdminSummary()])
    config.value = configData
    summary.value = summaryData
  } catch (err) {
    error.value = err instanceof Error ? err.message : '加载失败'
  }
})
</script>

<template>
  <PageHeader eyebrow="Admin" title="管理后台" description="管理邀请码、用户、隧道和本机 frps。" />

  <p v-if="error" class="mb-4 rounded-2xl border border-red-300/20 bg-red-400/10 px-4 py-3 text-sm text-red-100">{{ error }}</p>

  <section v-if="summary" class="grid gap-4 md:grid-cols-3 xl:grid-cols-6">
    <div class="card p-5"><div class="text-sm text-slate-400">用户</div><div class="mt-2 text-3xl font-black text-white">{{ summary.user_count }}</div></div>
    <div class="card p-5"><div class="text-sm text-slate-400">禁用用户</div><div class="mt-2 text-3xl font-black text-white">{{ summary.disabled_user_count }}</div></div>
    <div class="card p-5"><div class="text-sm text-slate-400">隧道</div><div class="mt-2 text-3xl font-black text-white">{{ summary.tunnel_count }}</div></div>
    <div class="card p-5"><div class="text-sm text-slate-400">邀请码</div><div class="mt-2 text-3xl font-black text-white">{{ summary.invite_count }}</div></div>
    <div class="card p-5"><div class="text-sm text-slate-400">未用邀请码</div><div class="mt-2 text-3xl font-black text-white">{{ summary.unused_invite_count }}</div></div>
    <div class="card p-5"><div class="text-sm text-slate-400">端口占用</div><div class="mt-2 text-xl font-black text-white">{{ summary.used_remote_port_count }} / {{ summary.remote_port_capacity }}</div></div>
  </section>

  <div class="mt-6 grid gap-4 md:grid-cols-4">
    <RouterLink class="card block p-6 no-underline transition hover:-translate-y-1 hover:bg-white/[0.08]" to="/admin/invites">
      <div class="text-lg font-bold text-white">邀请码</div>
      <p class="mt-2 text-sm text-slate-400">生成、筛选、复制和作废邀请码。</p>
    </RouterLink>
    <RouterLink class="card block p-6 no-underline transition hover:-translate-y-1 hover:bg-white/[0.08]" to="/admin/users">
      <div class="text-lg font-bold text-white">用户</div>
      <p class="mt-2 text-sm text-slate-400">搜索、筛选、启禁用和重置密码。</p>
    </RouterLink>
    <RouterLink class="card block p-6 no-underline transition hover:-translate-y-1 hover:bg-white/[0.08]" to="/admin/tunnels">
      <div class="text-lg font-bold text-white">全部隧道</div>
      <p class="mt-2 text-sm text-slate-400">查看用户名、协议和远程端口。</p>
    </RouterLink>
    <RouterLink class="card block p-6 no-underline transition hover:-translate-y-1 hover:bg-white/[0.08]" to="/admin/frps">
      <div class="text-lg font-bold text-white">frps 管理</div>
      <p class="mt-2 text-sm text-slate-400">编辑本机 frps 配置并手动重启。</p>
    </RouterLink>
  </div>

  <section v-if="config" class="card mt-6 p-6">
    <h2 class="text-xl font-bold text-white">当前配置</h2>
    <dl class="mt-4 grid gap-3 text-sm">
      <div class="flex justify-between gap-4 border-b border-white/10 pb-3"><dt class="text-slate-400">frps 地址</dt><dd class="font-mono text-cyan-100">{{ config.frps_server_addr }}:{{ config.frps_bind_port }}</dd></div>
      <div class="flex justify-between gap-4 border-b border-white/10 pb-3"><dt class="text-slate-400">远程端口范围</dt><dd class="font-mono text-cyan-100">{{ config.remote_port_min }}-{{ config.remote_port_max }}</dd></div>
      <div class="flex justify-between gap-4"><dt class="text-slate-400">每个用户最多隧道数</dt><dd class="font-mono text-cyan-100">{{ config.user_max_tunnels }}</dd></div>
    </dl>
  </section>
</template>

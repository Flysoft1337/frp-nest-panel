<script setup lang="ts">
import { onMounted, ref } from 'vue'

import { getDashboardSummary } from '../api/dashboard'
import type { DashboardSummary } from '../api/types'
import PageHeader from '../components/PageHeader.vue'
import StatusPill from '../components/StatusPill.vue'

const summary = ref<DashboardSummary | null>(null)
const error = ref('')

onMounted(async () => {
  try {
    summary.value = await getDashboardSummary()
  } catch (err) {
    error.value = err instanceof Error ? err.message : '加载失败'
  }
})
</script>

<template>
  <PageHeader eyebrow="Account" title="用户信息" description="查看账号状态、隧道配额和使用情况。" />

  <p v-if="error" class="mb-4 rounded-2xl border border-red-300/20 bg-red-400/10 px-4 py-3 text-sm text-red-100">{{ error }}</p>

  <section v-if="summary" class="grid gap-4 md:grid-cols-3">
    <div class="card p-6">
      <div class="text-sm text-slate-400">用户名</div>
      <div class="mt-2 text-2xl font-black text-white">{{ summary.username }}</div>
    </div>
    <div class="card p-6">
      <div class="text-sm text-slate-400">角色</div>
      <div class="mt-2"><StatusPill>{{ summary.role }}</StatusPill></div>
    </div>
    <div class="card p-6">
      <div class="text-sm text-slate-400">状态</div>
      <div class="mt-2"><StatusPill :tone="summary.disabled ? 'danger' : 'success'">{{ summary.disabled ? '已禁用' : '正常' }}</StatusPill></div>
    </div>
    <div class="card p-6">
      <div class="text-sm text-slate-400">创建时间</div>
      <div class="mt-2 font-mono text-cyan-100">{{ summary.created_at }}</div>
    </div>
    <div class="card p-6">
      <div class="text-sm text-slate-400">隧道数量</div>
      <div class="mt-2 text-2xl font-black text-white">{{ summary.tunnel_count }} / {{ summary.effective_max_tunnels }}</div>
    </div>
    <div class="card p-6">
      <div class="text-sm text-slate-400">剩余额度</div>
      <div class="mt-2 text-2xl font-black text-white">{{ summary.remaining_tunnels }}</div>
    </div>
  </section>
</template>

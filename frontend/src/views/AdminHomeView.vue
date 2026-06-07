<script setup lang="ts">
import { onMounted, ref } from 'vue'

import { getConfig } from '../api/admin'
import type { ConfigResponse } from '../api/types'
import PageHeader from '../components/PageHeader.vue'

const config = ref<ConfigResponse | null>(null)

onMounted(async () => {
  config.value = await getConfig()
})
</script>

<template>
  <PageHeader eyebrow="Admin" title="管理后台" description="管理邀请码、用户和全部隧道。" />

  <div class="grid gap-4 md:grid-cols-3">
    <RouterLink class="card block p-6 no-underline transition hover:-translate-y-1 hover:bg-white/[0.08]" to="/admin/invites">
      <div class="text-lg font-bold text-white">邀请码</div>
      <p class="mt-2 text-sm text-slate-400">生成和查看注册邀请码。</p>
    </RouterLink>
    <RouterLink class="card block p-6 no-underline transition hover:-translate-y-1 hover:bg-white/[0.08]" to="/admin/users">
      <div class="text-lg font-bold text-white">用户</div>
      <p class="mt-2 text-sm text-slate-400">启用、禁用和重置密码。</p>
    </RouterLink>
    <RouterLink class="card block p-6 no-underline transition hover:-translate-y-1 hover:bg-white/[0.08]" to="/admin/tunnels">
      <div class="text-lg font-bold text-white">全部隧道</div>
      <p class="mt-2 text-sm text-slate-400">查看和删除所有用户隧道。</p>
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

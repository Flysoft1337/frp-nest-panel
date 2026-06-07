<script setup lang="ts">
import { onMounted, ref } from 'vue'

import { deleteTunnel, listAllTunnels } from '../api/admin'
import type { Tunnel } from '../api/types'
import ConfirmButton from '../components/ConfirmButton.vue'
import PageHeader from '../components/PageHeader.vue'
import StatusPill from '../components/StatusPill.vue'

const tunnels = ref<Tunnel[]>([])
const error = ref('')

async function load() {
  tunnels.value = await listAllTunnels()
}

async function remove(id: string) {
  await deleteTunnel(id)
  await load()
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
  <PageHeader eyebrow="Admin" title="全部隧道" description="查看所有用户创建的 TCP/UDP 隧道。" />
  <section class="card p-6">
    <p v-if="error" class="mb-4 rounded-2xl border border-red-300/20 bg-red-400/10 px-4 py-3 text-sm text-red-100">{{ error }}</p>
    <div class="table-wrap">
      <table class="data-table">
        <thead><tr><th>用户 ID</th><th>名称</th><th>协议</th><th>本地</th><th>远程端口</th><th>创建时间</th><th>操作</th></tr></thead>
        <tbody>
          <tr v-for="tunnel in tunnels" :key="tunnel.id">
            <td><code class="text-slate-400">{{ tunnel.user_id }}</code></td>
            <td class="font-semibold text-white">{{ tunnel.name }}</td>
            <td><StatusPill>{{ tunnel.protocol }}</StatusPill></td>
            <td><code class="text-slate-300">{{ tunnel.local_host }}:{{ tunnel.local_port }}</code></td>
            <td><code class="text-cyan-100">{{ tunnel.remote_port }}</code></td>
            <td class="text-slate-400">{{ tunnel.created_at }}</td>
            <td><ConfirmButton message="确定删除这个隧道吗？" @confirm="remove(tunnel.id)">删除</ConfirmButton></td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

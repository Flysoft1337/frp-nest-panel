<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'

import { deleteTunnel, listAllTunnels } from '../api/admin'
import type { AdminTunnelRow, PageResponse } from '../api/types'
import ConfirmButton from '../components/ConfirmButton.vue'
import PageHeader from '../components/PageHeader.vue'
import StatusPill from '../components/StatusPill.vue'

const page = ref<PageResponse<AdminTunnelRow> | null>(null)
const q = ref('')
const status = ref('')
const currentPage = ref(1)
const error = ref('')
const message = ref('')

const totalPages = computed(() => {
  if (!page.value) return 1
  return Math.max(1, Math.ceil(page.value.total / page.value.page_size))
})

async function load() {
  page.value = await listAllTunnels({ q: q.value, status: status.value, page: currentPage.value })
}

async function remove(id: string) {
  error.value = ''
  message.value = ''
  try {
    await deleteTunnel(id)
    await load()
    message.value = '隧道已删除'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '删除失败'
  }
}

watch([q, status], () => {
  currentPage.value = 1
  load().catch((err) => { error.value = err instanceof Error ? err.message : '加载失败' })
})

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
    <p v-if="message" class="mb-4 rounded-2xl border border-emerald-300/20 bg-emerald-400/10 px-4 py-3 text-sm text-emerald-100">{{ message }}</p>
    <div class="mb-4 grid gap-3 md:grid-cols-[1fr_180px]">
      <input v-model="q" placeholder="搜索名称、本地地址、远程端口或用户名" />
      <select v-model="status">
        <option value="">全部协议</option>
        <option value="tcp">TCP</option>
        <option value="udp">UDP</option>
      </select>
    </div>
    <div class="table-wrap">
      <table class="data-table">
        <thead><tr><th>用户</th><th>名称</th><th>协议</th><th>本地</th><th>远程端口</th><th>创建时间</th><th>操作</th></tr></thead>
        <tbody>
          <tr v-for="row in page?.items || []" :key="row.tunnel.id">
            <td class="font-semibold text-white">{{ row.username }}</td>
            <td class="font-semibold text-white">{{ row.tunnel.name }}</td>
            <td><StatusPill>{{ row.tunnel.protocol }}</StatusPill></td>
            <td><code class="text-slate-300">{{ row.tunnel.local_host }}:{{ row.tunnel.local_port }}</code></td>
            <td><code class="text-cyan-100">{{ row.tunnel.remote_port }}</code></td>
            <td class="text-slate-400">{{ row.tunnel.created_at }}</td>
            <td><ConfirmButton message="确定删除这个隧道吗？" @confirm="remove(row.tunnel.id)">删除</ConfirmButton></td>
          </tr>
        </tbody>
      </table>
    </div>
    <div class="mt-4 flex items-center justify-between text-sm text-slate-400">
      <span>共 {{ page?.total || 0 }} 条</span>
      <div class="flex items-center gap-2">
        <button class="btn-secondary" :disabled="currentPage <= 1" @click="currentPage--; load()">上一页</button>
        <span>{{ currentPage }} / {{ totalPages }}</span>
        <button class="btn-secondary" :disabled="currentPage >= totalPages" @click="currentPage++; load()">下一页</button>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'

import { deleteTunnel, listAllTunnels } from '../api/admin'
import type { AdminTunnelRow, PageResponse } from '../api/types'
import AdminNav from '../components/AdminNav.vue'
import AlertBox from '../components/AlertBox.vue'
import ConfirmButton from '../components/ConfirmButton.vue'
import PageHeader from '../components/PageHeader.vue'
import PaginationBar from '../components/PaginationBar.vue'
import SelectField from '../components/SelectField.vue'
import StatusPill from '../components/StatusPill.vue'
import Toolbar from '../components/Toolbar.vue'

const page = ref<PageResponse<AdminTunnelRow> | null>(null)
const q = ref('')
const status = ref('')
const currentPage = ref(1)
const error = ref('')
const message = ref('')
const protocolOptions = [
  { label: '全部协议', value: '' },
  { label: 'TCP', value: 'tcp' },
  { label: 'UDP', value: 'udp' },
  { label: 'HTTP', value: 'http' },
  { label: 'HTTPS', value: 'https' },
]

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
  <PageHeader eyebrow="Admin" title="全部隧道" description="查看所有用户创建的端口和域名隧道。" />
  <AdminNav />

  <section class="card p-6">
    <div class="mb-4 grid gap-3">
      <AlertBox v-if="error" tone="danger" :message="error" />
      <AlertBox v-if="message" tone="success" :message="message" />
    </div>

    <Toolbar>
      <input v-model="q" placeholder="搜索名称、本地地址、远程端口、域名或用户名" />
      <SelectField v-model="status" :options="protocolOptions" />
    </Toolbar>

    <div class="grid gap-3">
      <article v-for="row in page?.items || []" :key="row.tunnel.id" class="rounded-3xl border border-white/10 bg-white/[0.04] p-4">
        <div class="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
          <div class="min-w-0">
            <div class="flex flex-wrap items-center gap-2">
              <h2 class="truncate text-lg font-black text-white">{{ row.tunnel.name }}</h2>
              <StatusPill>{{ row.tunnel.protocol }}</StatusPill>
            </div>
            <div class="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-sm text-slate-400">
              <span>用户 <strong class="text-slate-200">{{ row.username }}</strong></span>
              <span>创建 {{ row.tunnel.created_at }}</span>
            </div>
          </div>

          <div class="grid gap-3 md:grid-cols-2 xl:min-w-[28rem]">
            <div class="rounded-2xl border border-white/10 bg-white/[0.03] px-4 py-3">
              <div class="text-xs text-slate-500">本地服务</div>
              <code class="mt-1 block truncate text-slate-200">{{ row.tunnel.local_host }}:{{ row.tunnel.local_port }}</code>
            </div>
            <div class="rounded-2xl border border-cyan-300/10 bg-cyan-300/[0.04] px-4 py-3">
              <div class="text-xs text-slate-500">入口</div>
              <code v-if="row.tunnel.remote_port" class="mt-1 block text-cyan-100">{{ row.tunnel.remote_port }}</code>
              <code v-else class="mt-1 block truncate text-cyan-100">{{ row.tunnel.custom_domain || '未配置' }}</code>
            </div>
          </div>

          <div class="flex justify-start xl:justify-end">
            <ConfirmButton message="确定删除这个隧道吗？" @confirm="remove(row.tunnel.id)">删除</ConfirmButton>
          </div>
        </div>
      </article>
    </div>

    <PaginationBar :total="page?.total || 0" :page="currentPage" :total-pages="totalPages" @prev="currentPage--; load()" @next="currentPage++; load()" />
  </section>
</template>

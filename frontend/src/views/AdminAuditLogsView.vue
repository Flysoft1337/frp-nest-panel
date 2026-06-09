<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'

import { listAuditLogs } from '../api/admin'
import type { AuditLog, PageResponse } from '../api/types'
import AdminNav from '../components/AdminNav.vue'
import AlertBox from '../components/AlertBox.vue'
import PageHeader from '../components/PageHeader.vue'
import PaginationBar from '../components/PaginationBar.vue'
import SelectField from '../components/SelectField.vue'
import StatusPill from '../components/StatusPill.vue'
import Toolbar from '../components/Toolbar.vue'

const page = ref<PageResponse<AuditLog> | null>(null)
const q = ref('')
const action = ref('')
const outcome = ref('')
const currentPage = ref(1)
const error = ref('')

const actionOptions = [
  { label: '全部动作', value: '' },
  { label: '认证', value: 'auth.login' },
  { label: '隧道创建', value: 'tunnel.create' },
  { label: '隧道更新', value: 'tunnel.update' },
  { label: '隧道删除', value: 'tunnel.delete' },
  { label: 'frpc 预览', value: 'tunnel.frpc_preview' },
  { label: 'frpc 下载', value: 'tunnel.frpc_download' },
  { label: '管理员 frps 更新', value: 'admin.frps.update' },
]

const outcomeOptions = [
  { label: '全部结果', value: '' },
  { label: '成功', value: 'success' },
  { label: '失败', value: 'failure' },
]

const totalPages = computed(() => {
  if (!page.value) return 1
  return Math.max(1, Math.ceil(page.value.total / page.value.page_size))
})

function formatTime(value: string) {
  return new Date(value).toLocaleString()
}

function metadataText(value: string | null) {
  if (!value) return ''
  try {
    return JSON.stringify(JSON.parse(value), null, 2)
  } catch {
    return value
  }
}

async function load() {
  page.value = await listAuditLogs({
    q: q.value,
    action: action.value,
    outcome: outcome.value,
    page: currentPage.value,
  })
}

function previousPage() {
  currentPage.value -= 1
  load().catch((err) => { error.value = err instanceof Error ? err.message : '加载失败' })
}

function nextPage() {
  currentPage.value += 1
  load().catch((err) => { error.value = err instanceof Error ? err.message : '加载失败' })
}

watch([q, action, outcome], () => {
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
  <PageHeader eyebrow="Admin" title="审计日志" description="查看登录、隧道、证书和管理员操作记录。" />
  <AdminNav />

  <section class="card p-6">
    <AlertBox v-if="error" class="mb-4" tone="danger" :message="error" />

    <Toolbar>
      <input v-model="q" placeholder="搜索用户、动作、资源" />
      <SelectField v-model="action" :options="actionOptions" />
      <SelectField v-model="outcome" :options="outcomeOptions" />
    </Toolbar>

    <div class="grid gap-3">
      <article v-for="item in page?.items || []" :key="item.id" class="rounded-3xl border border-white/10 bg-white/[0.04] p-4">
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div class="min-w-0">
            <div class="flex flex-wrap items-center gap-2">
              <h2 class="break-all text-base font-black text-white">{{ item.action }}</h2>
              <StatusPill :tone="item.outcome === 'success' ? 'success' : 'danger'">{{ item.outcome }}</StatusPill>
              <StatusPill>{{ item.resource_type }}</StatusPill>
            </div>
            <div class="mt-2 flex flex-wrap gap-4 text-sm text-slate-400">
              <span>操作者 <code class="text-cyan-100">{{ item.actor_username || 'system' }}</code></span>
              <span v-if="item.resource_name">资源 <code class="text-cyan-100">{{ item.resource_name }}</code></span>
              <span>{{ formatTime(item.created_at) }}</span>
            </div>
            <p v-if="item.message" class="mt-2 text-sm text-slate-300">{{ item.message }}</p>
          </div>
        </div>
        <pre v-if="metadataText(item.metadata_json)" class="mt-3 max-h-40 overflow-auto rounded-2xl border border-white/10 bg-slate-950/50 p-3 text-xs text-slate-300">{{ metadataText(item.metadata_json) }}</pre>
      </article>
    </div>

    <div v-if="page && page.items.length === 0" class="empty-state">
      <h2 class="text-xl font-bold text-white">没有审计记录</h2>
      <p class="mt-2 text-sm text-slate-400">调整筛选条件后再查看。</p>
    </div>

    <PaginationBar :total="page?.total || 0" :page="currentPage" :total-pages="totalPages" @prev="previousPage" @next="nextPage" />
  </section>
</template>

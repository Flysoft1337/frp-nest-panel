<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'

import { disableUser, enableUser, listUsers, resetUserPassword, updateUserQuota } from '../api/admin'
import type { PageResponse, UserRow } from '../api/types'
import AdminNav from '../components/AdminNav.vue'
import AlertBox from '../components/AlertBox.vue'
import ConfirmButton from '../components/ConfirmButton.vue'
import PageHeader from '../components/PageHeader.vue'
import PaginationBar from '../components/PaginationBar.vue'
import QuotaEditor from '../components/QuotaEditor.vue'
import SelectField from '../components/SelectField.vue'
import StatusPill from '../components/StatusPill.vue'
import Toolbar from '../components/Toolbar.vue'
import { useSessionStore } from '../stores/session'

const session = useSessionStore()
const page = ref<PageResponse<UserRow> | null>(null)
const passwords = reactive<Record<string, string>>({})
const quotas = reactive<Record<string, number | null>>({})
const q = ref('')
const status = ref('')
const currentPage = ref(1)
const error = ref('')
const message = ref('')
const statusOptions = [
  { label: '全部状态', value: '' },
  { label: '正常', value: 'enabled' },
  { label: '已禁用', value: 'disabled' },
]

const totalPages = computed(() => {
  if (!page.value) return 1
  return Math.max(1, Math.ceil(page.value.total / page.value.page_size))
})

async function load() {
  page.value = await listUsers({ q: q.value, status: status.value, page: currentPage.value })
  for (const row of page.value.items) {
    quotas[row.user.id] = row.user.max_tunnels
  }
}

async function toggle(id: string, disabled: boolean) {
  error.value = ''
  message.value = ''
  try {
    if (disabled) await enableUser(id)
    else await disableUser(id)
    await load()
    message.value = disabled ? '用户已启用' : '用户已禁用'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '操作失败'
  }
}

async function reset(id: string) {
  const password = passwords[id]
  if (!password) return
  error.value = ''
  message.value = ''
  try {
    await resetUserPassword(id, password)
    passwords[id] = ''
    message.value = '密码已重置'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '重置失败'
  }
}

async function saveQuota(id: string) {
  error.value = ''
  message.value = ''
  try {
    await updateUserQuota(id, quotas[id] || null)
    await load()
    message.value = '隧道配额已更新'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '更新配额失败'
  }
}

function previousPage() {
  currentPage.value -= 1
  load()
}

function nextPage() {
  currentPage.value += 1
  load()
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
  <PageHeader eyebrow="Admin" title="用户" description="查看用户状态、隧道数量、配额和重置密码。" />
  <AdminNav />
  <section class="card p-6">
    <div class="mb-4 grid gap-3">
      <AlertBox v-if="error" tone="danger" :message="error" />
      <AlertBox v-if="message" tone="success" :message="message" />
    </div>

    <Toolbar>
      <input v-model="q" placeholder="搜索用户名" />
      <SelectField v-model="status" :options="statusOptions" />
    </Toolbar>

    <div class="grid gap-3">
      <article v-for="row in page?.items || []" :key="row.user.id" class="rounded-3xl border border-white/10 bg-white/[0.04] p-4">
        <div class="grid gap-4">
          <div class="flex flex-wrap items-start justify-between gap-3">
            <div class="min-w-0">
              <div class="flex flex-wrap items-center gap-2">
                <h2 class="text-lg font-black text-white">{{ row.user.username }}</h2>
                <StatusPill>{{ row.user.role }}</StatusPill>
                <StatusPill :tone="row.user.disabled ? 'danger' : 'success'">{{ row.user.disabled ? '已禁用' : '正常' }}</StatusPill>
                <span v-if="row.user.id === session.user?.id" class="rounded-full border border-white/10 bg-white/5 px-2.5 py-1 text-xs text-slate-400">当前用户</span>
              </div>
              <div class="mt-2 flex flex-wrap gap-4 text-sm text-slate-400">
                <span>隧道 <code class="text-cyan-100">{{ row.tunnel_count }}</code></span>
                <span>创建 {{ row.user.created_at }}</span>
              </div>
            </div>
            <ConfirmButton v-if="row.user.id !== session.user?.id" :message="row.user.disabled ? '确定启用这个用户吗？' : '确定禁用这个用户吗？'" :class-name="row.user.disabled ? 'btn-secondary' : 'btn-danger'" @confirm="toggle(row.user.id, row.user.disabled)">{{ row.user.disabled ? '启用' : '禁用' }}</ConfirmButton>
          </div>

          <div class="grid gap-3 border-t border-white/10 pt-4 lg:grid-cols-[auto_1fr] lg:items-end">
            <div class="rounded-2xl border border-white/10 bg-slate-950/30 p-3">
              <div class="mb-2 text-xs font-bold uppercase tracking-[0.18em] text-slate-500">Quota</div>
              <QuotaEditor v-model="quotas[row.user.id]" :effective="row.effective_max_tunnels" @save="saveQuota(row.user.id)" />
            </div>

            <form class="grid gap-2 rounded-2xl border border-white/10 bg-slate-950/30 p-3 md:grid-cols-[1fr_auto]" @submit.prevent="reset(row.user.id)">
              <div>
                <div class="mb-2 text-xs font-bold uppercase tracking-[0.18em] text-slate-500">Password</div>
                <input v-model="passwords[row.user.id]" minlength="8" placeholder="输入新密码" required type="password" />
              </div>
              <button class="btn-secondary self-end whitespace-nowrap" type="submit">重置密码</button>
            </form>
          </div>
        </div>
      </article>
    </div>

    <PaginationBar :total="page?.total || 0" :page="currentPage" :total-pages="totalPages" @prev="previousPage" @next="nextPage" />
  </section>
</template>

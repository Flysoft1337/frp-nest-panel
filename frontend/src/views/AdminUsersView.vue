<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'

import { disableUser, enableUser, listUsers, resetUserPassword, updateUserQuota } from '../api/admin'
import type { PageResponse, UserRow } from '../api/types'
import AdminNav from '../components/AdminNav.vue'
import ConfirmButton from '../components/ConfirmButton.vue'
import PageHeader from '../components/PageHeader.vue'
import StatusPill from '../components/StatusPill.vue'
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
    <p v-if="error" class="mb-4 rounded-2xl border border-red-300/20 bg-red-400/10 px-4 py-3 text-sm text-red-100">{{ error }}</p>
    <p v-if="message" class="mb-4 rounded-2xl border border-emerald-300/20 bg-emerald-400/10 px-4 py-3 text-sm text-emerald-100">{{ message }}</p>
    <div class="mb-4 grid gap-3 md:grid-cols-[1fr_180px]">
      <input v-model="q" placeholder="搜索用户名" />
      <select v-model="status">
        <option value="">全部状态</option>
        <option value="enabled">正常</option>
        <option value="disabled">已禁用</option>
      </select>
    </div>
    <div class="table-wrap">
      <table class="data-table">
        <thead><tr><th>用户名</th><th>角色</th><th>状态</th><th>隧道数</th><th>配额</th><th>创建时间</th><th>操作</th><th>重置密码</th></tr></thead>
        <tbody>
          <tr v-for="row in page?.items || []" :key="row.user.id">
            <td class="font-semibold text-white">{{ row.user.username }}</td>
            <td><StatusPill>{{ row.user.role }}</StatusPill></td>
            <td><StatusPill :tone="row.user.disabled ? 'danger' : 'success'">{{ row.user.disabled ? '已禁用' : '正常' }}</StatusPill></td>
            <td><code class="text-cyan-100">{{ row.tunnel_count }}</code></td>
            <td>
              <form class="flex min-w-52 gap-2" @submit.prevent="saveQuota(row.user.id)">
                <input v-model="quotas[row.user.id]" min="1" placeholder="默认" type="number" />
                <button class="btn-secondary" type="submit">保存</button>
              </form>
              <div class="mt-1 text-xs text-slate-500">当前有效：{{ row.effective_max_tunnels }}</div>
            </td>
            <td class="text-slate-400">{{ row.user.created_at }}</td>
            <td>
              <ConfirmButton v-if="row.user.id !== session.user?.id" :message="row.user.disabled ? '确定启用这个用户吗？' : '确定禁用这个用户吗？'" :class-name="row.user.disabled ? 'btn-secondary' : 'btn-danger'" @confirm="toggle(row.user.id, row.user.disabled)">{{ row.user.disabled ? '启用' : '禁用' }}</ConfirmButton>
              <span v-else class="text-sm text-slate-500">当前用户</span>
            </td>
            <td>
              <form class="flex min-w-64 gap-2" @submit.prevent="reset(row.user.id)">
                <input v-model="passwords[row.user.id]" minlength="8" placeholder="新密码" required type="password" />
                <button class="btn-secondary" type="submit">重置</button>
              </form>
            </td>
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

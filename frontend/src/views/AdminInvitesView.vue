<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'

import { createInvites, deleteInvite, listInvites } from '../api/admin'
import type { Invite, PageResponse } from '../api/types'
import ConfirmButton from '../components/ConfirmButton.vue'
import PageHeader from '../components/PageHeader.vue'
import StatusPill from '../components/StatusPill.vue'

const page = ref<PageResponse<Invite> | null>(null)
const count = ref(1)
const expiresDays = ref<number | null>(null)
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
  page.value = await listInvites({ q: q.value, status: status.value, page: currentPage.value })
}

async function submit() {
  error.value = ''
  message.value = ''
  try {
    await createInvites(count.value, expiresDays.value)
    currentPage.value = 1
    await load()
    message.value = '邀请码已生成'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '生成失败'
  }
}

async function copy(code: string) {
  await navigator.clipboard.writeText(code)
  message.value = '邀请码已复制'
}

async function remove(id: string) {
  await deleteInvite(id)
  await load()
  message.value = '邀请码已作废'
}

function inviteStatus(invite: Invite) {
  if (invite.used_by) return '已使用'
  if (invite.expires_at && new Date(invite.expires_at).getTime() <= Date.now()) return '已过期'
  return '未使用'
}

function statusTone(invite: Invite) {
  if (invite.used_by) return 'default'
  if (invite.expires_at && new Date(invite.expires_at).getTime() <= Date.now()) return 'danger'
  return 'success'
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
  <PageHeader eyebrow="Admin" title="邀请码" description="生成注册邀请码，并查看使用状态。" />

  <section class="card p-6">
    <form class="grid gap-4 md:grid-cols-[1fr_1fr_auto] md:items-end" @submit.prevent="submit">
      <label>生成数量<input v-model="count" max="100" min="1" required type="number" /></label>
      <label>过期天数<input v-model="expiresDays" min="0" placeholder="0 表示永不过期" type="number" /></label>
      <button class="btn-primary" type="submit">生成邀请码</button>
    </form>
    <p v-if="error" class="mt-4 rounded-2xl border border-red-300/20 bg-red-400/10 px-4 py-3 text-sm text-red-100">{{ error }}</p>
    <p v-if="message" class="mt-4 rounded-2xl border border-emerald-300/20 bg-emerald-400/10 px-4 py-3 text-sm text-emerald-100">{{ message }}</p>
  </section>

  <section class="card mt-6 p-6">
    <div class="mb-4 grid gap-3 md:grid-cols-[1fr_180px]">
      <input v-model="q" placeholder="搜索邀请码" />
      <select v-model="status">
        <option value="">全部状态</option>
        <option value="unused">未使用</option>
        <option value="used">已使用</option>
        <option value="expired">已过期</option>
      </select>
    </div>
    <div class="table-wrap">
      <table class="data-table">
        <thead><tr><th>邀请码</th><th>状态</th><th>过期时间</th><th>创建时间</th><th>操作</th></tr></thead>
        <tbody>
          <tr v-for="invite in page?.items || []" :key="invite.id">
            <td><code class="text-cyan-100">{{ invite.code }}</code></td>
            <td><StatusPill :tone="statusTone(invite)">{{ inviteStatus(invite) }}</StatusPill></td>
            <td class="text-slate-300">{{ invite.expires_at || '永不过期' }}</td>
            <td class="text-slate-400">{{ invite.created_at }}</td>
            <td class="flex gap-2">
              <button class="btn-secondary" type="button" @click="copy(invite.code)">复制</button>
              <ConfirmButton v-if="!invite.used_by" message="确定作废这个邀请码吗？" @confirm="remove(invite.id)">作废</ConfirmButton>
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

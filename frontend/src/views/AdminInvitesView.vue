<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'

import { createInvites, deleteInvite, listInvites } from '../api/admin'
import type { Invite, PageResponse } from '../api/types'
import AdminNav from '../components/AdminNav.vue'
import AlertBox from '../components/AlertBox.vue'
import ConfirmButton from '../components/ConfirmButton.vue'
import FormField from '../components/FormField.vue'
import PageHeader from '../components/PageHeader.vue'
import PaginationBar from '../components/PaginationBar.vue'
import SelectField from '../components/SelectField.vue'
import StatusPill from '../components/StatusPill.vue'
import Toolbar from '../components/Toolbar.vue'

const page = ref<PageResponse<Invite> | null>(null)
const count = ref(1)
const expiresDays = ref<number | null>(null)
const q = ref('')
const status = ref('')
const currentPage = ref(1)
const error = ref('')
const message = ref('')
const statusOptions = [
  { label: '全部状态', value: '' },
  { label: '未使用', value: 'unused' },
  { label: '已使用', value: 'used' },
  { label: '已过期', value: 'expired' },
]

const totalPages = computed(() => {
  if (!page.value) return 1
  return Math.max(1, Math.ceil(page.value.total / page.value.page_size))
})
const inviteCountLabel = computed(() => `${page.value?.total || 0} 个邀请码`)

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
  <AdminNav />

  <section class="card p-6">
    <div class="mb-5 grid gap-3">
      <AlertBox v-if="error" tone="danger" :message="error" />
      <AlertBox v-if="message" tone="success" :message="message" />
      <div>
        <h2 class="text-lg font-bold text-white">生成邀请码</h2>
        <p class="mt-1 text-sm text-slate-400">邀请码用于开放一次注册入口，可设置数量和过期时间。</p>
      </div>
    </div>

    <form class="grid gap-4 rounded-3xl border border-white/10 bg-white/[0.03] p-4 md:grid-cols-[1fr_1fr_auto] md:items-end" @submit.prevent="submit">
      <FormField label="生成数量"><input v-model="count" max="100" min="1" required type="number" /></FormField>
      <FormField label="过期天数" note="0 或留空表示永不过期"><input v-model="expiresDays" min="0" placeholder="永不过期" type="number" /></FormField>
      <button class="btn-primary" type="submit">生成邀请码</button>
    </form>
  </section>

  <section class="card mt-6 p-6">
    <div class="mb-5 flex flex-wrap items-center justify-between gap-3">
      <div>
        <h2 class="text-lg font-bold text-white">邀请码列表</h2>
        <p class="mt-1 text-sm text-slate-400">复制未使用的邀请码，或作废不再需要的入口。</p>
      </div>
      <span class="rounded-full border border-cyan-300/20 bg-cyan-300/10 px-3 py-1 text-xs font-bold text-cyan-100">{{ inviteCountLabel }}</span>
    </div>

    <Toolbar>
      <input v-model="q" placeholder="搜索邀请码" />
      <SelectField v-model="status" :options="statusOptions" />
    </Toolbar>

    <div v-if="(page?.items || []).length === 0" class="empty-state">
      <div class="text-base font-bold text-white">没有匹配的邀请码</div>
      <p class="mt-2 text-sm text-slate-400">调整筛选条件，或先生成新的邀请码。</p>
    </div>
    <div v-else class="grid gap-3">
      <article v-for="invite in page?.items || []" :key="invite.id" class="rounded-3xl border border-white/10 bg-white/[0.04] p-4 transition hover:border-cyan-300/20 hover:bg-white/[0.06]">
        <div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
          <div class="min-w-0">
            <div class="flex flex-wrap items-center gap-2">
              <code class="break-all rounded-2xl border border-cyan-300/15 bg-cyan-300/[0.06] px-3 py-2 text-base font-bold text-cyan-100">{{ invite.code }}</code>
              <StatusPill :tone="statusTone(invite)">{{ inviteStatus(invite) }}</StatusPill>
            </div>
            <div class="mt-3 grid gap-1 text-sm text-slate-400 md:grid-cols-2">
              <span>过期 {{ invite.expires_at || '永不过期' }}</span>
              <span>创建 {{ invite.created_at }}</span>
            </div>
          </div>

          <div class="flex flex-wrap gap-2">
            <button class="btn-secondary" type="button" @click="copy(invite.code)">复制</button>
            <ConfirmButton v-if="!invite.used_by" message="确定作废这个邀请码吗？" @confirm="remove(invite.id)">作废</ConfirmButton>
          </div>
        </div>
      </article>
    </div>

    <PaginationBar :total="page?.total || 0" :page="currentPage" :total-pages="totalPages" @prev="currentPage--; load()" @next="currentPage++; load()" />
  </section>
</template>

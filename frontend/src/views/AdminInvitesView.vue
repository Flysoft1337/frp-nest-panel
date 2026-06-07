<script setup lang="ts">
import { onMounted, ref } from 'vue'

import { createInvites, listInvites } from '../api/admin'
import type { Invite } from '../api/types'
import PageHeader from '../components/PageHeader.vue'
import StatusPill from '../components/StatusPill.vue'

const invites = ref<Invite[]>([])
const count = ref(1)
const expiresDays = ref<number | null>(null)
const error = ref('')

async function load() {
  invites.value = await listInvites()
}

async function submit() {
  error.value = ''
  try {
    await createInvites(count.value, expiresDays.value)
    await load()
  } catch (err) {
    error.value = err instanceof Error ? err.message : '生成失败'
  }
}

onMounted(load)
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
  </section>

  <section class="card mt-6 p-6">
    <div class="table-wrap">
      <table class="data-table">
        <thead><tr><th>邀请码</th><th>状态</th><th>过期时间</th><th>创建时间</th></tr></thead>
        <tbody>
          <tr v-for="invite in invites" :key="invite.id">
            <td><code class="text-cyan-100">{{ invite.code }}</code></td>
            <td><StatusPill :tone="invite.used_by ? 'default' : 'success'">{{ invite.used_by ? '已使用' : '未使用' }}</StatusPill></td>
            <td class="text-slate-300">{{ invite.expires_at || '永不过期' }}</td>
            <td class="text-slate-400">{{ invite.created_at }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'

import { disableUser, enableUser, listUsers, resetUserPassword } from '../api/admin'
import type { UserRow } from '../api/types'
import ConfirmButton from '../components/ConfirmButton.vue'
import PageHeader from '../components/PageHeader.vue'
import StatusPill from '../components/StatusPill.vue'
import { useSessionStore } from '../stores/session'

const session = useSessionStore()
const rows = ref<UserRow[]>([])
const passwords = reactive<Record<string, string>>({})
const error = ref('')

async function load() {
  rows.value = await listUsers()
}

async function toggle(id: string, disabled: boolean) {
  if (disabled) await enableUser(id)
  else await disableUser(id)
  await load()
}

async function reset(id: string) {
  const password = passwords[id]
  if (!password) return
  await resetUserPassword(id, password)
  passwords[id] = ''
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
  <PageHeader eyebrow="Admin" title="用户" description="查看用户状态、隧道数量和重置密码。" />
  <section class="card p-6">
    <p v-if="error" class="mb-4 rounded-2xl border border-red-300/20 bg-red-400/10 px-4 py-3 text-sm text-red-100">{{ error }}</p>
    <div class="table-wrap">
      <table class="data-table">
        <thead><tr><th>用户名</th><th>角色</th><th>状态</th><th>隧道数</th><th>创建时间</th><th>操作</th><th>重置密码</th></tr></thead>
        <tbody>
          <tr v-for="row in rows" :key="row.user.id">
            <td class="font-semibold text-white">{{ row.user.username }}</td>
            <td><StatusPill>{{ row.user.role }}</StatusPill></td>
            <td><StatusPill :tone="row.user.disabled ? 'danger' : 'success'">{{ row.user.disabled ? '已禁用' : '正常' }}</StatusPill></td>
            <td><code class="text-cyan-100">{{ row.tunnel_count }}</code></td>
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
  </section>
</template>

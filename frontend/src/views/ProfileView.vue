<script setup lang="ts">
import { onMounted, ref } from 'vue'

import { changePassword } from '../api/auth'
import { getDashboardSummary } from '../api/dashboard'
import type { DashboardSummary } from '../api/types'
import AlertBox from '../components/AlertBox.vue'
import FormField from '../components/FormField.vue'
import PageHeader from '../components/PageHeader.vue'
import StatusPill from '../components/StatusPill.vue'

const summary = ref<DashboardSummary | null>(null)
const error = ref('')
const passwordError = ref('')
const passwordMessage = ref('')
const passwordLoading = ref(false)
const currentPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')

async function submitPassword() {
  passwordError.value = ''
  passwordMessage.value = ''
  passwordLoading.value = true
  try {
    await changePassword(currentPassword.value, newPassword.value, confirmPassword.value)
    currentPassword.value = ''
    newPassword.value = ''
    confirmPassword.value = ''
    passwordMessage.value = '密码已更新'
  } catch (err) {
    passwordError.value = err instanceof Error ? err.message : '修改失败'
  } finally {
    passwordLoading.value = false
  }
}

onMounted(async () => {
  try {
    summary.value = await getDashboardSummary()
  } catch (err) {
    error.value = err instanceof Error ? err.message : '加载失败'
  }
})
</script>

<template>
  <PageHeader eyebrow="Account" title="个人资料" description="查看账号状态、隧道配额并修改密码。" />

  <AlertBox v-if="error" class="mb-4" tone="danger" :message="error" />

  <section class="grid gap-6 xl:grid-cols-[1fr_24rem]">
    <div v-if="summary" class="grid gap-4 md:grid-cols-3">
      <div class="card border-cyan-300/20 bg-cyan-300/[0.06] p-6">
        <div class="text-sm text-cyan-100/80">用户名</div>
        <div class="mt-2 text-2xl font-black text-white">{{ summary.username }}</div>
      </div>
      <div class="card p-6">
        <div class="text-sm text-slate-400">角色</div>
        <div class="mt-2"><StatusPill>{{ summary.role }}</StatusPill></div>
      </div>
      <div class="card p-6">
        <div class="text-sm text-slate-400">状态</div>
        <div class="mt-2"><StatusPill :tone="summary.disabled ? 'danger' : 'success'">{{ summary.disabled ? '已禁用' : '正常' }}</StatusPill></div>
      </div>
      <div class="card p-6">
        <div class="text-sm text-slate-400">创建时间</div>
        <div class="mt-2 font-mono text-cyan-100">{{ summary.created_at }}</div>
      </div>
      <div class="card p-6">
        <div class="text-sm text-slate-400">隧道数量</div>
        <div class="mt-2 text-2xl font-black text-white">{{ summary.tunnel_count }} / {{ summary.effective_max_tunnels }}</div>
      </div>
      <div class="card p-6">
        <div class="text-sm text-slate-400">剩余额度</div>
        <div class="mt-2 text-2xl font-black" :class="summary.remaining_tunnels > 0 ? 'text-emerald-200' : 'text-red-200'">{{ summary.remaining_tunnels }}</div>
      </div>
    </div>

    <section class="card p-6">
      <div class="mb-5">
        <h2 class="text-lg font-bold text-white">修改密码</h2>
        <p class="mt-1 text-sm text-slate-400">更新当前登录账号的密码。</p>
      </div>
      <form class="grid gap-4" @submit.prevent="submitPassword">
        <FormField label="当前密码"><input v-model="currentPassword" autocomplete="current-password" required type="password" /></FormField>
        <FormField label="新密码"><input v-model="newPassword" autocomplete="new-password" minlength="8" required type="password" /></FormField>
        <FormField label="确认新密码"><input v-model="confirmPassword" autocomplete="new-password" minlength="8" required type="password" /></FormField>
        <AlertBox v-if="passwordMessage" tone="success" :message="passwordMessage" />
        <AlertBox v-if="passwordError" tone="danger" :message="passwordError" />
        <button class="btn-primary w-fit" :disabled="passwordLoading" type="submit">{{ passwordLoading ? '保存中' : '保存密码' }}</button>
      </form>
    </section>
  </section>
</template>

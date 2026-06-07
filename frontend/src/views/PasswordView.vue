<script setup lang="ts">
import { ref } from 'vue'

import { changePassword } from '../api/auth'
import PageHeader from '../components/PageHeader.vue'

const currentPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const message = ref('')
const error = ref('')
const loading = ref(false)

async function submit() {
  message.value = ''
  error.value = ''
  loading.value = true
  try {
    await changePassword(currentPassword.value, newPassword.value, confirmPassword.value)
    currentPassword.value = ''
    newPassword.value = ''
    confirmPassword.value = ''
    message.value = '密码已更新'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '修改失败'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <PageHeader eyebrow="Account" title="修改密码" description="更新当前登录账号的密码。" />
  <section class="card max-w-xl p-6">
    <form class="grid gap-4" @submit.prevent="submit">
      <label>当前密码<input v-model="currentPassword" autocomplete="current-password" required type="password" /></label>
      <label>新密码<input v-model="newPassword" autocomplete="new-password" minlength="8" required type="password" /></label>
      <label>确认新密码<input v-model="confirmPassword" autocomplete="new-password" minlength="8" required type="password" /></label>
      <p v-if="message" class="rounded-2xl border border-emerald-300/20 bg-emerald-400/10 px-4 py-3 text-sm text-emerald-100">{{ message }}</p>
      <p v-if="error" class="rounded-2xl border border-red-300/20 bg-red-400/10 px-4 py-3 text-sm text-red-100">{{ error }}</p>
      <button class="btn-primary w-fit" :disabled="loading" type="submit">{{ loading ? '保存中' : '保存' }}</button>
    </form>
  </section>
</template>

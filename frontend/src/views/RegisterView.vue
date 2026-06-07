<script setup lang="ts">
import { ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { useSessionStore } from '../stores/session'

const route = useRoute()
const router = useRouter()
const session = useSessionStore()

const inviteCode = ref((route.query.code as string) || '')
const username = ref('')
const password = ref('')
const error = ref('')
const loading = ref(false)

async function submit() {
  error.value = ''
  loading.value = true
  try {
    await session.register(inviteCode.value, username.value, password.value)
    await router.push('/dashboard')
  } catch (err) {
    error.value = err instanceof Error ? err.message : '注册失败'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <section class="mx-auto grid min-h-[60vh] max-w-md place-items-center">
    <div class="card w-full p-7">
      <p class="mb-2 text-xs font-bold uppercase tracking-[0.25em] text-cyan-200/80">Invite</p>
      <h1 class="text-3xl font-black tracking-tight text-white">邀请码注册</h1>
      <p class="mt-2 text-sm text-slate-400">输入管理员提供的邀请码后即可创建自己的隧道。</p>

      <form class="mt-7 grid gap-4" @submit.prevent="submit">
        <label>邀请码<input v-model="inviteCode" required /></label>
        <label>用户名<input v-model="username" autocomplete="username" required /></label>
        <label>密码<input v-model="password" autocomplete="new-password" minlength="8" required type="password" /></label>
        <p v-if="error" class="rounded-2xl border border-red-300/20 bg-red-400/10 px-4 py-3 text-sm text-red-100">{{ error }}</p>
        <button class="btn-primary w-full" :disabled="loading" type="submit">{{ loading ? '注册中' : '注册' }}</button>
      </form>

      <RouterLink class="mt-5 block text-center text-sm text-slate-400 hover:text-cyan-200" to="/login">已有账号，去登录</RouterLink>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import AlertBox from '../components/AlertBox.vue'
import BrandIcon from '../components/BrandIcon.vue'
import FormField from '../components/FormField.vue'
import { useSessionStore } from '../stores/session'

const route = useRoute()
const router = useRouter()
const session = useSessionStore()

const username = ref('')
const password = ref('')
const error = ref('')
const loading = ref(false)

async function submit() {
  error.value = ''
  loading.value = true
  try {
    await session.login(username.value, password.value)
    await router.push((route.query.redirect as string) || '/dashboard')
  } catch (err) {
    error.value = err instanceof Error ? err.message : '登录失败'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <section class="mx-auto grid min-h-[65vh] max-w-md place-items-center">
    <div class="card w-full p-7">
      <div class="mb-6 flex items-center gap-3">
        <BrandIcon class="h-12 w-12 drop-shadow-[0_12px_24px_rgba(34,211,238,0.22)]" />
        <div>
          <p class="text-xs font-bold uppercase tracking-[0.25em] text-cyan-200/80">Sign in</p>
          <h1 class="text-3xl font-black tracking-tight text-white">登录面板</h1>
        </div>
      </div>
      <p class="text-sm text-slate-400">管理你的 TCP、UDP、HTTP 和 HTTPS 隧道。</p>

      <form class="mt-7 grid gap-4" @submit.prevent="submit">
        <FormField label="用户名"><input v-model="username" autocomplete="username" required /></FormField>
        <FormField label="密码"><input v-model="password" autocomplete="current-password" required type="password" /></FormField>
        <AlertBox v-if="error" tone="danger" :message="error" />
        <button class="btn-primary w-full" :disabled="loading" type="submit">{{ loading ? '登录中' : '登录' }}</button>
      </form>

      <RouterLink class="mt-5 block text-center text-sm text-slate-400 hover:text-cyan-200" to="/register">使用邀请码注册</RouterLink>
    </div>
  </section>
</template>

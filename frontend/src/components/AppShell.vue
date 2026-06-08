<script setup lang="ts">
import { useRouter } from 'vue-router'

import { useSessionStore } from '../stores/session'

const session = useSessionStore()
const router = useRouter()

async function signOut() {
  await session.logout()
  await router.push('/login')
}
</script>

<template>
  <div class="relative min-h-screen overflow-hidden">
    <div class="pointer-events-none absolute inset-0">
      <div class="absolute -left-40 top-0 h-96 w-96 rounded-full bg-cyan-500/20 blur-3xl" />
      <div class="absolute right-0 top-20 h-96 w-96 rounded-full bg-violet-500/20 blur-3xl" />
      <div class="absolute bottom-0 left-1/2 h-96 w-96 -translate-x-1/2 rounded-full bg-blue-500/10 blur-3xl" />
    </div>

    <div class="relative mx-auto flex min-h-screen w-full max-w-7xl flex-col px-4 py-5 sm:px-6 lg:px-8">
      <header class="card mb-8 flex flex-col gap-4 px-5 py-4 md:flex-row md:items-center md:justify-between">
        <RouterLink to="/dashboard" class="flex items-center gap-3 text-slate-100 no-underline">
          <div class="grid h-11 w-11 place-items-center rounded-2xl bg-cyan-300 text-lg font-black text-slate-950 shadow-lg shadow-cyan-500/20">N</div>
          <div>
            <div class="text-base font-bold tracking-tight">frp-nest-panel</div>
            <div class="text-xs text-slate-400">Self-service frp tunnels</div>
          </div>
        </RouterLink>

        <nav v-if="session.isAuthenticated" class="flex flex-wrap items-center gap-2 rounded-3xl border border-white/10 bg-slate-950/30 p-1.5">
          <RouterLink class="nav-link" active-class="nav-link-active" to="/dashboard">Dashboard</RouterLink>
          <RouterLink class="nav-link" active-class="nav-link-active" to="/certificates">证书</RouterLink>
          <RouterLink v-if="session.isAdmin" class="nav-link" active-class="nav-link-active" to="/admin">管理</RouterLink>
          <RouterLink class="nav-link" active-class="nav-link-active" to="/profile">个人资料</RouterLink>
          <button class="btn-secondary" type="button" @click="signOut">退出</button>
        </nav>
      </header>

      <main class="flex-1">
        <slot />
      </main>
    </div>
  </div>
</template>

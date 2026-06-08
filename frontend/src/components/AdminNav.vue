<script setup lang="ts">
const links = [
  { to: '/admin', label: '概览' },
  { to: '/admin/invites', label: '邀请码' },
  { to: '/admin/users', label: '用户' },
  { to: '/admin/tunnels', label: '隧道' },
  { to: '/admin/frps', label: 'frps' },
]
</script>

<template>
  <nav class="mb-6 flex w-full gap-2 overflow-x-auto rounded-3xl border border-white/10 bg-white/[0.04] p-2 shadow-2xl shadow-slate-950/30 backdrop-blur-xl">
    <RouterLink
      v-for="link in links"
      :key="link.to"
      :to="link.to"
      class="shrink-0 rounded-2xl px-4 py-2 text-sm font-semibold text-slate-300 no-underline transition hover:bg-white/10 hover:text-white"
      active-class="bg-cyan-300 text-slate-950 shadow-lg shadow-cyan-500/20 hover:bg-cyan-300 hover:text-slate-950"
      :class="$route.path === link.to ? 'bg-cyan-300 text-slate-950 shadow-lg shadow-cyan-500/20 hover:bg-cyan-300 hover:text-slate-950' : ''"
    >
      {{ link.label }}
    </RouterLink>
  </nav>
</template>

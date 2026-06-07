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
  <nav class="card mb-6 flex flex-wrap gap-2 p-2">
    <RouterLink
      v-for="link in links"
      :key="link.to"
      :to="link.to"
      class="rounded-2xl px-4 py-2 text-sm font-semibold text-slate-300 no-underline transition hover:bg-white/10 hover:text-white"
      active-class="bg-cyan-300 text-slate-950 hover:bg-cyan-300 hover:text-slate-950"
      :class="$route.path === link.to ? 'bg-cyan-300 text-slate-950 hover:bg-cyan-300 hover:text-slate-950' : ''"
    >
      {{ link.label }}
    </RouterLink>
  </nav>
</template>

import { createRouter, createWebHistory } from 'vue-router'

import { useSessionStore } from '../stores/session'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', redirect: '/dashboard' },
    { path: '/login', component: () => import('../views/LoginView.vue'), meta: { public: true } },
    { path: '/register', component: () => import('../views/RegisterView.vue'), meta: { public: true } },
    { path: '/dashboard', component: () => import('../views/DashboardView.vue') },
    { path: '/tunnels/new', component: () => import('../views/TunnelCreateView.vue') },
    { path: '/tunnels/:id/edit', component: () => import('../views/TunnelCreateView.vue') },
    { path: '/tunnels/:id/frpc', component: () => import('../views/FrpcPreviewView.vue') },
    { path: '/password', component: () => import('../views/PasswordView.vue') },
    { path: '/admin', component: () => import('../views/AdminHomeView.vue'), meta: { admin: true } },
    { path: '/admin/invites', component: () => import('../views/AdminInvitesView.vue'), meta: { admin: true } },
    { path: '/admin/users', component: () => import('../views/AdminUsersView.vue'), meta: { admin: true } },
    { path: '/admin/tunnels', component: () => import('../views/AdminTunnelsView.vue'), meta: { admin: true } },
    { path: '/:pathMatch(.*)*', redirect: '/dashboard' },
  ],
})

router.beforeEach(async (to) => {
  const session = useSessionStore()
  if (!session.loaded) {
    await session.load()
  }

  if (to.meta.public) {
    if (session.isAuthenticated && (to.path === '/login' || to.path === '/register')) {
      return '/dashboard'
    }
    return true
  }

  if (!session.isAuthenticated) {
    return { path: '/login', query: { redirect: to.fullPath } }
  }

  if (to.meta.admin && !session.isAdmin) {
    return '/dashboard'
  }

  return true
})

export default router

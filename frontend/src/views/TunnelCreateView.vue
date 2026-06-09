<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { listCertificates } from '../api/certificates'
import { getDashboardSummary } from '../api/dashboard'
import { createTunnel, getTunnel, updateTunnel } from '../api/tunnels'
import type { CertificateInfo, DashboardSummary } from '../api/types'
import AlertBox from '../components/AlertBox.vue'
import FormField from '../components/FormField.vue'
import PageHeader from '../components/PageHeader.vue'
import SelectField from '../components/SelectField.vue'

const route = useRoute()
const router = useRouter()
const tunnelId = computed(() => route.params.id as string | undefined)
const isEdit = computed(() => Boolean(tunnelId.value))
const isPortTunnel = computed(() => protocol.value === 'tcp' || protocol.value === 'udp')
const isDomainTunnel = computed(() => protocol.value === 'http' || protocol.value === 'https')

const name = ref('')
const protocol = ref('tcp')
const localHost = ref('127.0.0.1')
const localPort = ref<number | null>(null)
const remotePort = ref<number | null>(null)
const customDomain = ref('')
const tlsMode = ref('https_passthrough')
const certificateId = ref<string | null>(null)
const certificates = ref<CertificateInfo[]>([])
const summary = ref<DashboardSummary | null>(null)
const error = ref('')
const loading = ref(false)
const loadingTunnel = ref(false)
const protocolOptions = [
  { label: 'TCP', value: 'tcp' },
  { label: 'UDP', value: 'udp' },
  { label: 'HTTP 域名', value: 'http' },
  { label: 'HTTPS 域名', value: 'https' },
]
const tlsModeOptions = [
  { label: 'HTTPS 透传，本地服务自己处理 TLS', value: 'https_passthrough' },
  { label: '上传证书，转发到本地 HTTP', value: 'uploaded_cert' },
]
const certificateOptions = computed(() => [
  { label: '选择证书', value: null, disabled: true },
  ...certificates.value.map((cert) => ({
    label: `${cert.name} · ${cert.domains.join(', ')}`,
    value: cert.id,
  })),
])
const protocolLabel = computed(() => protocolOptions.find((option) => option.value === protocol.value)?.label ?? protocol.value.toUpperCase())
const entryLabel = computed(() => {
  if (isPortTunnel.value) return isEdit.value ? '远程端口可修改，留空则保留当前端口或自动分配' : '远程端口可手动填写，也可以留空自动分配'
  if (protocol.value === 'http') return '通过 HTTP vhost 域名访问本地服务'
  return tlsMode.value === 'uploaded_cert' ? 'frpc 使用上传证书提供 HTTPS 入口' : 'TLS 透传到本地服务'
})
const activeVhostPort = computed(() => (protocol.value === 'http' ? summary.value?.vhost_http_port : summary.value?.vhost_https_port))
const vhostEnabled = computed(() => !isDomainTunnel.value || Boolean(activeVhostPort.value))
const vhostStatusLabel = computed(() => {
  if (!isDomainTunnel.value) return ''
  return activeVhostPort.value ? `${protocol.value.toUpperCase()} 域名入口已启用：${activeVhostPort.value}` : `${protocol.value.toUpperCase()} 域名入口未启用`
})
const entryPreview = computed(() => {
  if (!summary.value) return '配置加载后显示入口预览'
  if (isPortTunnel.value) {
    const port = remotePort.value || (isEdit.value ? '当前端口' : '自动分配')
    return `${summary.value.frps_server_addr}:${port}`
  }
  const domains = customDomain.value
    .split(/[,\n]/)
    .map((domain) => domain.trim())
    .filter(Boolean)
  if (domains.length === 0) return '填写绑定域名后显示入口预览'
  return domains.map((domain) => `${protocol.value}://${domain}`).join(' · ')
})
const protocolCards = computed(() => protocolOptions.map((option) => ({
  ...option,
  description: option.value === 'tcp'
    ? '开放一个远程 TCP 端口。'
    : option.value === 'udp'
      ? '开放一个远程 UDP 端口。'
      : option.value === 'http'
        ? '用 HTTP 域名访问本地服务。'
        : '用 HTTPS 域名访问本地服务。',
})))

async function loadTunnel() {
  if (!tunnelId.value) return
  loadingTunnel.value = true
  error.value = ''
  try {
    const tunnel = await getTunnel(tunnelId.value)
    name.value = tunnel.name
    protocol.value = tunnel.protocol
    localHost.value = tunnel.local_host
    localPort.value = tunnel.local_port
    remotePort.value = tunnel.remote_port
    customDomain.value = tunnel.custom_domain ?? ''
    tlsMode.value = tunnel.tls_mode ?? 'https_passthrough'
    certificateId.value = tunnel.certificate_id
  } catch (err) {
    error.value = err instanceof Error ? err.message : '加载失败'
  } finally {
    loadingTunnel.value = false
  }
}

async function submit() {
  if (!localPort.value) return
  if (!vhostEnabled.value) {
    error.value = `${protocol.value.toUpperCase()} 域名入口未启用，请联系管理员配置 frps vhost 端口`
    return
  }
  error.value = ''
  loading.value = true
  try {
    const input = {
      name: name.value,
      protocol: protocol.value,
      local_host: localHost.value,
      local_port: localPort.value,
      remote_port: isPortTunnel.value ? remotePort.value : null,
      custom_domain: isDomainTunnel.value ? customDomain.value : null,
      tls_mode: protocol.value === 'https' ? tlsMode.value : null,
      certificate_id: protocol.value === 'https' && tlsMode.value === 'uploaded_cert' ? certificateId.value : null,
    }
    if (tunnelId.value) {
      await updateTunnel(tunnelId.value, input)
    } else {
      await createTunnel(input)
    }
    await router.push('/dashboard')
  } catch (err) {
    error.value = err instanceof Error ? err.message : isEdit.value ? '保存失败' : '创建失败'
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  try {
    const [certificateList, dashboardSummary] = await Promise.all([listCertificates(), getDashboardSummary()])
    certificates.value = certificateList
    summary.value = dashboardSummary
  } catch {
    certificates.value = []
  }
  await loadTunnel()
})
</script>

<template>
  <PageHeader
    eyebrow="Tunnel"
    :title="isEdit ? '编辑隧道' : '创建隧道'"
    :description="isEdit ? '修改隧道名称、协议、本地服务和远端入口。' : '先选择协议，再填写本地服务和远端入口。'"
  />
  <section class="card max-w-3xl p-6">
    <p v-if="loadingTunnel" class="mb-4 text-sm text-slate-400">加载中</p>
    <form class="grid gap-5" @submit.prevent="submit">
      <div class="grid gap-3 md:grid-cols-4">
        <button
          v-for="item in protocolCards"
          :key="item.value"
          class="rounded-3xl border p-4 text-left transition"
          :class="protocol === item.value ? 'border-cyan-300/40 bg-cyan-300/[0.08]' : 'border-white/10 bg-white/[0.03] hover:border-cyan-300/20'"
          type="button"
          @click="protocol = item.value"
        >
          <div class="font-black text-white">{{ item.label }}</div>
          <div class="mt-2 text-sm text-slate-400">{{ item.description }}</div>
        </button>
      </div>

      <div class="rounded-3xl border border-cyan-300/20 bg-cyan-300/[0.06] p-4">
        <div class="text-xs font-bold uppercase tracking-[0.2em] text-cyan-100/80">{{ protocolLabel }} 入口预览</div>
        <code class="mt-2 block break-all text-sm text-cyan-50">{{ entryPreview }}</code>
        <div class="mt-2 text-sm text-slate-300">{{ entryLabel }}</div>
      </div>

      <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-4">
        <div class="mb-4 flex flex-col gap-1">
          <h2 class="text-lg font-bold text-white">基础信息</h2>
          <p class="text-sm text-slate-400">名称只用于面板和 frpc 配置识别。</p>
        </div>
        <div class="grid gap-5 md:grid-cols-2">
          <FormField label="隧道名称"><input v-model="name" placeholder="mc-server" required /></FormField>
          <FormField label="当前协议"><input :value="protocolLabel" disabled /></FormField>
        </div>
      </section>

      <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-4">
        <div class="mb-4 flex flex-col gap-1">
          <h2 class="text-lg font-bold text-white">本地服务</h2>
          <p class="text-sm text-slate-400">frpc 会把远端入口转发到这里。</p>
        </div>
        <div class="grid gap-5 md:grid-cols-2">
          <FormField label="本地地址"><input v-model="localHost" required /></FormField>
          <FormField label="本地端口"><input v-model="localPort" max="65535" min="1" required type="number" /></FormField>
        </div>
      </section>

      <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-4">
        <div class="mb-4 flex flex-col gap-1">
          <h2 class="text-lg font-bold text-white">远端入口</h2>
          <p class="text-sm text-slate-400">端口隧道使用远程端口，域名隧道使用绑定域名。</p>
        </div>
        <div class="grid gap-5">
          <FormField v-if="isPortTunnel" label="远程端口（可选)"><input v-model="remotePort" max="65535" min="1" placeholder="留空自动分配" type="number" /></FormField>
          <FormField v-if="isDomainTunnel" label="绑定域名"><textarea v-model="customDomain" required rows="3" placeholder="example.com&#10;api.example.com" /></FormField>
          <FormField v-if="protocol === 'https'" label="TLS 模式"><SelectField v-model="tlsMode" :options="tlsModeOptions" /></FormField>
          <FormField v-if="protocol === 'https' && tlsMode === 'uploaded_cert'" label="证书"><SelectField v-model="certificateId" :options="certificateOptions" /></FormField>
        </div>
      </section>

      <div v-if="protocol === 'https' && tlsMode === 'uploaded_cert' && certificates.length === 0" class="rounded-2xl border border-amber-300/20 bg-amber-300/10 px-4 py-3 text-sm text-amber-100">
        还没有可用证书。先到证书页上传证书和私钥，再回来选择。
      </div>
      <div v-if="isDomainTunnel" :class="vhostEnabled ? 'rounded-2xl border border-cyan-300/20 bg-cyan-400/10 px-4 py-3 text-sm text-cyan-100' : 'rounded-2xl border border-amber-300/20 bg-amber-300/10 px-4 py-3 text-sm text-amber-100'">
        <div class="font-semibold">{{ vhostStatusLabel }}</div>
        <div class="mt-1">每行一个域名，最多 8 个。域名需要解析到 frps 服务器。HTTPS 上传证书模式会生成包含证书和私钥的 frpc.zip。</div>
      </div>
      <AlertBox v-if="error" :message="error" tone="danger" />
      <div class="sticky bottom-4 z-20 flex flex-wrap gap-3 rounded-3xl border border-white/10 bg-slate-950/80 p-3 shadow-2xl shadow-slate-950/60 backdrop-blur-xl">
        <button class="btn-primary" :disabled="loading || loadingTunnel || !vhostEnabled" type="submit">{{ loading ? '保存中' : isEdit ? '保存修改' : '创建' }}</button>
        <RouterLink class="btn-secondary" role="button" to="/dashboard">返回</RouterLink>
      </div>
    </form>
  </section>
</template>

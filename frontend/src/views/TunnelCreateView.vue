<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { listCertificates } from '../api/certificates'
import { createTunnel, getTunnel, updateTunnel } from '../api/tunnels'
import type { CertificateInfo } from '../api/types'
import FormField from '../components/FormField.vue'
import PageHeader from '../components/PageHeader.vue'

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
const error = ref('')
const loading = ref(false)
const loadingTunnel = ref(false)

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
  error.value = ''
  loading.value = true
  try {
    const input = {
      name: name.value,
      protocol: protocol.value,
      local_host: localHost.value,
      local_port: localPort.value,
      remote_port: isEdit.value || !isPortTunnel.value ? undefined : remotePort.value,
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
    certificates.value = await listCertificates()
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
    :description="isEdit ? '修改隧道名称、协议和本地服务地址。远程端口保持不变。' : '选择 TCP 或 UDP，并填写本地服务地址。远程端口可选。'"
  />
  <section class="card max-w-2xl p-6">
    <p v-if="loadingTunnel" class="mb-4 text-sm text-slate-400">加载中</p>
    <form class="grid gap-5" @submit.prevent="submit">
      <FormField label="隧道名称"><input v-model="name" placeholder="mc-server" required /></FormField>
      <FormField label="协议">
        <select v-model="protocol" required>
          <option value="tcp">TCP</option>
          <option value="udp">UDP</option>
          <option value="http">HTTP 域名</option>
          <option value="https">HTTPS 域名</option>
        </select>
      </FormField>
      <div class="grid gap-5 md:grid-cols-2">
        <FormField label="本地地址"><input v-model="localHost" required /></FormField>
        <FormField label="本地端口"><input v-model="localPort" max="65535" min="1" required type="number" /></FormField>
      </div>
      <FormField v-if="!isEdit && isPortTunnel" label="远程端口（可选)"><input v-model="remotePort" max="65535" min="1" placeholder="留空自动分配" type="number" /></FormField>
      <FormField v-if="isDomainTunnel" label="绑定域名"><input v-model="customDomain" required placeholder="example.com" /></FormField>
      <FormField v-if="protocol === 'https'" label="TLS 模式">
        <select v-model="tlsMode" required>
          <option value="https_passthrough">HTTPS 透传，本地服务自己处理 TLS</option>
          <option value="uploaded_cert">上传证书，转发到本地 HTTP</option>
        </select>
      </FormField>
      <FormField v-if="protocol === 'https' && tlsMode === 'uploaded_cert'" label="证书">
        <select v-model="certificateId" required>
          <option :value="null" disabled>选择证书</option>
          <option v-for="cert in certificates" :key="cert.id" :value="cert.id">{{ cert.name }} · {{ cert.domains.join(', ') }}</option>
        </select>
      </FormField>
      <div v-if="isDomainTunnel" class="rounded-2xl border border-cyan-300/20 bg-cyan-400/10 px-4 py-3 text-sm text-cyan-100">
        域名需要解析到 frps 服务器。HTTPS 上传证书模式会生成包含证书和私钥的 frpc.zip。
      </div>
      <div v-if="isEdit && remotePort" class="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-slate-300">
        远程端口保持不变：<code class="text-cyan-100">{{ remotePort }}</code>
      </div>
      <p v-if="error" class="rounded-2xl border border-red-300/20 bg-red-400/10 px-4 py-3 text-sm text-red-100">{{ error }}</p>
      <div class="flex flex-wrap gap-3">
        <button class="btn-primary" :disabled="loading || loadingTunnel" type="submit">{{ loading ? '保存中' : isEdit ? '保存修改' : '创建' }}</button>
        <RouterLink class="btn-secondary" role="button" to="/dashboard">返回</RouterLink>
      </div>
    </form>
  </section>
</template>

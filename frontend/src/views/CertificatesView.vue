<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

import { createCertificate, deleteCertificate, listCertificates } from '../api/certificates'
import type { CertificateInfo } from '../api/types'
import AlertBox from '../components/AlertBox.vue'
import ConfirmButton from '../components/ConfirmButton.vue'
import FormField from '../components/FormField.vue'
import PageHeader from '../components/PageHeader.vue'

const certificates = ref<CertificateInfo[]>([])
const name = ref('')
const certificatePem = ref('')
const privateKeyPem = ref('')
const error = ref('')
const message = ref('')
const loading = ref(false)
const certificateCountLabel = computed(() => `${certificates.value.length} 张证书`)

async function load() {
  certificates.value = await listCertificates()
}

async function submit() {
  error.value = ''
  message.value = ''
  loading.value = true
  try {
    await createCertificate(name.value, certificatePem.value, privateKeyPem.value)
    name.value = ''
    certificatePem.value = ''
    privateKeyPem.value = ''
    message.value = '证书已上传'
    await load()
  } catch (err) {
    error.value = err instanceof Error ? err.message : '上传失败'
  } finally {
    loading.value = false
  }
}

async function remove(id: string) {
  error.value = ''
  message.value = ''
  try {
    await deleteCertificate(id)
    message.value = '证书已删除'
    await load()
  } catch (err) {
    error.value = err instanceof Error ? err.message : '删除失败'
  }
}

onMounted(async () => {
  try {
    await load()
  } catch (err) {
    error.value = err instanceof Error ? err.message : '加载失败'
  }
})
</script>

<template>
  <PageHeader eyebrow="Certificates" title="SSL 证书" description="上传用于 HTTPS 域名隧道的证书和私钥。私钥不会在页面中回显。" />

  <div v-if="error || message" class="fixed right-6 top-6 z-50 grid w-[min(24rem,calc(100vw-3rem))] gap-3 shadow-2xl">
    <AlertBox v-if="error" tone="danger" :message="error" />
    <AlertBox v-if="message" tone="success" :message="message" />
  </div>

  <section class="grid gap-6 xl:grid-cols-[1fr_28rem]">
    <div class="card p-6">
      <div class="mb-5 flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 class="text-lg font-bold text-white">已上传证书</h2>
          <p class="mt-1 text-sm text-slate-400">HTTPS 上传证书模式会从这里选择证书。</p>
        </div>
        <span class="rounded-full border border-cyan-300/20 bg-cyan-300/10 px-3 py-1 text-xs font-bold text-cyan-100">{{ certificateCountLabel }}</span>
      </div>
      <div v-if="certificates.length === 0" class="empty-state">
        <div class="text-base font-bold text-white">还没有证书</div>
        <p class="mt-2 max-w-md text-sm text-slate-400">上传证书和私钥后，创建 HTTPS 域名隧道时可以直接选择。</p>
      </div>
      <div v-else class="grid gap-3">
        <article v-for="cert in certificates" :key="cert.id" class="rounded-3xl border border-white/10 bg-white/[0.03] p-4 transition hover:border-cyan-300/20 hover:bg-white/[0.05]">
          <div class="flex flex-wrap items-start justify-between gap-3">
            <div class="min-w-0 flex-1">
              <div class="flex flex-wrap items-center gap-2">
                <h3 class="font-bold text-white">{{ cert.name }}</h3>
                <span class="rounded-full bg-white/10 px-2.5 py-1 text-xs text-slate-300">{{ cert.domains.length }} 个域名</span>
              </div>
              <div class="mt-3 flex flex-wrap gap-2">
                <code v-for="domain in cert.domains" :key="domain" class="rounded-full border border-cyan-300/15 bg-cyan-300/[0.06] px-2.5 py-1 text-xs text-cyan-100">{{ domain }}</code>
              </div>
              <div class="mt-4 grid gap-2 text-xs text-slate-500 md:grid-cols-2">
                <div>过期时间：<span class="font-mono text-slate-300">{{ cert.not_after }}</span></div>
                <div class="break-all font-mono">SHA256 {{ cert.fingerprint_sha256 }}</div>
              </div>
            </div>
            <ConfirmButton message="确定删除这个证书吗？使用中的证书不能删除。" @confirm="remove(cert.id)">删除</ConfirmButton>
          </div>
        </article>
      </div>
    </div>

    <section class="card p-6">
      <div class="mb-5">
        <h2 class="text-lg font-bold text-white">上传证书</h2>
        <p class="mt-1 text-sm text-slate-400">粘贴完整 PEM 内容。私钥只保存到后端，不会在页面回显。</p>
      </div>
      <form class="grid gap-4" @submit.prevent="submit">
        <FormField label="证书名称"><input v-model="name" required placeholder="example.com" /></FormField>
        <FormField label="证书 PEM"><textarea v-model="certificatePem" required rows="8" placeholder="-----BEGIN CERTIFICATE-----" /></FormField>
        <FormField label="私钥 PEM"><textarea v-model="privateKeyPem" required rows="8" placeholder="-----BEGIN PRIVATE KEY-----" /></FormField>
        <div class="rounded-2xl border border-amber-300/20 bg-amber-300/10 px-4 py-3 text-sm text-amber-100">证书需要包含绑定域名。通配符证书可以用于对应子域名。</div>
        <button class="btn-primary w-fit" :disabled="loading" type="submit">{{ loading ? '上传中' : '上传证书' }}</button>
      </form>
    </section>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'

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

  <div class="mb-4 grid gap-3">
    <AlertBox v-if="error" tone="danger" :message="error" />
    <AlertBox v-if="message" tone="success" :message="message" />
  </div>

  <section class="grid gap-6 xl:grid-cols-[1fr_28rem]">
    <div class="card p-6">
      <h2 class="mb-4 text-lg font-bold text-white">已上传证书</h2>
      <div v-if="certificates.length === 0" class="rounded-3xl border border-dashed border-white/10 py-12 text-center text-sm text-slate-400">还没有证书</div>
      <div v-else class="grid gap-3">
        <article v-for="cert in certificates" :key="cert.id" class="rounded-3xl border border-white/10 bg-white/[0.03] p-4">
          <div class="flex flex-wrap items-start justify-between gap-3">
            <div>
              <h3 class="font-bold text-white">{{ cert.name }}</h3>
              <p class="mt-1 break-all text-sm text-cyan-100">{{ cert.domains.join(', ') }}</p>
              <p class="mt-2 text-xs text-slate-500">过期时间：{{ cert.not_after }}</p>
              <p class="mt-1 break-all font-mono text-xs text-slate-500">SHA256 {{ cert.fingerprint_sha256 }}</p>
            </div>
            <ConfirmButton message="确定删除这个证书吗？使用中的证书不能删除。" @confirm="remove(cert.id)">删除</ConfirmButton>
          </div>
        </article>
      </div>
    </div>

    <section class="card p-6">
      <h2 class="mb-4 text-lg font-bold text-white">上传证书</h2>
      <form class="grid gap-4" @submit.prevent="submit">
        <FormField label="证书名称"><input v-model="name" required placeholder="example.com" /></FormField>
        <FormField label="证书 PEM"><textarea v-model="certificatePem" required rows="8" placeholder="-----BEGIN CERTIFICATE-----" /></FormField>
        <FormField label="私钥 PEM"><textarea v-model="privateKeyPem" required rows="8" placeholder="-----BEGIN PRIVATE KEY-----" /></FormField>
        <button class="btn-primary w-fit" :disabled="loading" type="submit">{{ loading ? '上传中' : '上传证书' }}</button>
      </form>
    </section>
  </section>
</template>

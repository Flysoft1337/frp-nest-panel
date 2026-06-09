<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'

import { getCaddy, getFrps, getPanelTls, reloadCaddy, restartFrps, restartPanel, updateCaddy, updateFrps, updatePanelTls } from '../api/admin'
import type { CaddyStatus, FrpsStatus, PanelTlsStatus } from '../api/types'
import AdminNav from '../components/AdminNav.vue'
import AlertBox from '../components/AlertBox.vue'
import ConfirmButton from '../components/ConfirmButton.vue'
import FormField from '../components/FormField.vue'
import PageHeader from '../components/PageHeader.vue'
import SelectField from '../components/SelectField.vue'
import StatCard from '../components/StatCard.vue'
import StatusPill from '../components/StatusPill.vue'

const status = ref<FrpsStatus | null>(null)
const panelTls = ref<PanelTlsStatus | null>(null)
const caddy = ref<CaddyStatus | null>(null)
const form = reactive({
  server_addr: '',
  bind_port: 7000,
  auth_token: '',
  remote_port_min: 20000,
  remote_port_max: 30000,
  dashboard_addr: '127.0.0.1',
  dashboard_port: null as number | null,
  dashboard_user: '',
  dashboard_password: '',
  enable_prometheus: false,
  vhost_http_port: null as number | null,
  vhost_https_port: null as number | null,
})
const panelTlsForm = reactive({
  enabled: false,
  bind: '0.0.0.0:8443',
  domain: '',
  certificate_pem: '',
  private_key_pem: '',
})
const caddyForm = reactive({
  enabled: false,
  domain: '',
})
const error = ref('')
const message = ref('')
const panelTlsEnabledOptions = [
  { label: '关闭', value: false },
  { label: '启用', value: true },
]
const caddyEnabledOptions = [
  { label: '关闭', value: false },
  { label: '启用', value: true },
]

async function load() {
  const data = await getFrps()
  status.value = data
  form.server_addr = data.server_addr
  form.bind_port = data.bind_port
  form.auth_token = ''
  form.remote_port_min = data.remote_port_min
  form.remote_port_max = data.remote_port_max
  form.dashboard_addr = data.dashboard_addr
  form.dashboard_port = data.dashboard_port
  form.dashboard_user = data.dashboard_user
  form.dashboard_password = ''
  form.enable_prometheus = data.enable_prometheus
  form.vhost_http_port = data.vhost_http_port
  form.vhost_https_port = data.vhost_https_port
  const tls = await getPanelTls()
  panelTls.value = tls
  panelTlsForm.enabled = tls.enabled
  panelTlsForm.bind = tls.bind
  panelTlsForm.domain = tls.domain
  panelTlsForm.certificate_pem = ''
  panelTlsForm.private_key_pem = ''
  const caddyData = await getCaddy()
  caddy.value = caddyData
  caddyForm.enabled = caddyData.enabled
  caddyForm.domain = caddyData.domain
}

async function save() {
  error.value = ''
  message.value = ''
  try {
    await updateFrps({ ...form })
    await load()
    message.value = '配置已保存，需重启 frps 生效'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '保存失败'
  }
}

async function savePanelTls() {
  error.value = ''
  message.value = ''
  try {
    panelTls.value = await updatePanelTls({ ...panelTlsForm })
    panelTlsForm.certificate_pem = ''
    panelTlsForm.private_key_pem = ''
    message.value = '面板 HTTPS 配置已保存，需重启面板生效'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '保存失败'
  }
}

async function saveCaddy() {
  error.value = ''
  message.value = ''
  try {
    caddy.value = await updateCaddy({ ...caddyForm })
    message.value = caddyForm.enabled ? 'Caddy 配置已保存并重载' : 'Caddy 面板入口已关闭'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '保存失败'
  }
}

async function reloadCaddyConfig() {
  error.value = ''
  message.value = ''
  try {
    caddy.value = await reloadCaddy()
    message.value = 'Caddy 已重载'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '重载失败'
  }
}

async function restart() {
  error.value = ''
  message.value = ''
  try {
    await restartFrps()
    await pollStatus()
    message.value = 'frps 已重启'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '重启失败'
  }
}

async function restartPanelService() {
  error.value = ''
  message.value = ''
  try {
    await restartPanel()
    message.value = '面板正在重启，请稍后刷新页面'
  } catch (err) {
    error.value = err instanceof Error ? err.message : '重启失败'
  }
}

async function pollStatus() {
  for (let index = 0; index < 10; index += 1) {
    await load()
    if (!status.value?.restarting) return
    await new Promise((resolve) => window.setTimeout(resolve, 1000))
  }
}

function statusTone(value: FrpsStatus) {
  if (value.state === 'running') return 'success'
  if (value.state === 'stopped') return 'danger'
  return 'default'
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
  <PageHeader eyebrow="Admin" title="frps 管理" description="编辑本机 frps 配置。保存不会自动重启。" />
  <AdminNav />

  <section v-if="status" class="grid gap-4 md:grid-cols-2 xl:grid-cols-6">
    <div class="card p-5">
      <div class="text-sm text-slate-400">状态</div>
      <div class="mt-2"><StatusPill :tone="statusTone(status)">{{ status.display_status }}</StatusPill></div>
    </div>
    <StatCard label="版本" :value="status.version" />
    <StatCard label="配置文件" :value="status.config_path" />
    <StatCard label="Token" :value="status.token_set ? '已设置' : '未设置'" />
    <div class="card p-5 md:col-span-2 xl:col-span-2">
      <div class="text-sm text-slate-400">Dashboard</div>
      <div class="mt-2"><StatusPill :tone="status.dashboard_available ? 'success' : 'default'">{{ status.dashboard_available ? '可用' : status.dashboard_configured ? '不可用' : '未配置' }}</StatusPill></div>
    </div>
  </section>

  <div v-if="error || message" class="fixed right-6 top-6 z-50 grid w-[min(24rem,calc(100vw-3rem))] gap-3 shadow-2xl">
    <AlertBox v-if="error" tone="danger" :message="error" />
    <AlertBox v-if="message" tone="success" :message="message" />
  </div>

  <section class="card mt-6 p-6">
    <form class="grid gap-5" @submit.prevent="save">
      <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-4">
        <div class="mb-4 flex flex-col gap-1">
          <h2 class="text-lg font-bold text-white">基础配置</h2>
          <p class="text-sm text-slate-400">面板生成 frpc 配置时会使用这里的地址和 bindPort。</p>
        </div>
        <div class="grid items-start gap-4 md:grid-cols-2">
          <FormField label="frps 地址"><input v-model="form.server_addr" required placeholder="example.com" /></FormField>
          <FormField label="bindPort"><input v-model="form.bind_port" max="65535" min="1" required type="number" /></FormField>
          <FormField label="Token" note="留空表示不修改"><input v-model="form.auth_token" autocomplete="new-password" placeholder="留空表示不修改" type="password" /></FormField>
        </div>
      </section>

      <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-4">
        <div class="mb-4 flex flex-col gap-1">
          <h2 class="text-lg font-bold text-white">远程端口范围</h2>
          <p class="text-sm text-slate-400">创建隧道时只能使用这个范围内的远程端口。</p>
        </div>
        <div class="grid items-start gap-4 md:grid-cols-2">
          <FormField label="最小值"><input v-model="form.remote_port_min" max="65535" min="1" required type="number" /></FormField>
          <FormField label="最大值"><input v-model="form.remote_port_max" max="65535" min="1" required type="number" /></FormField>
        </div>
      </section>

      <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-4">
        <div class="mb-4 flex flex-col gap-1">
          <h2 class="text-lg font-bold text-white">域名入口</h2>
          <p class="text-sm text-slate-400">启用 HTTP/HTTPS 域名隧道时，frps 会监听这些 vhost 端口。</p>
        </div>
        <div class="grid items-start gap-4 md:grid-cols-2">
          <FormField label="HTTP vhost 端口"><input v-model="form.vhost_http_port" max="65535" min="1" placeholder="留空关闭" type="number" /></FormField>
          <FormField label="HTTPS vhost 端口"><input v-model="form.vhost_https_port" max="65535" min="1" placeholder="留空关闭" type="number" /></FormField>
        </div>
        <p v-if="caddyForm.enabled" class="mt-4 rounded-2xl border border-amber-300/20 bg-amber-300/10 px-4 py-3 text-sm text-amber-100">
          Caddy 已接管公网 80/443。HTTPS vhost 不要填写 443；如果要让 frp HTTPS 域名隧道共用 443，需要另外做 Caddy 分流。
        </p>
      </section>

      <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-4">
        <div class="mb-4 flex flex-col gap-1">
          <h2 class="text-lg font-bold text-white">Dashboard / Prometheus 数据源</h2>
          <p class="text-sm text-slate-400">Dashboard 提供实时状态；Prometheus 用于长期流量采样。保存并重启 frps 后生效。</p>
        </div>
        <div v-if="status" class="mb-4 rounded-2xl border border-white/10 bg-slate-950/40 px-4 py-3 text-sm text-slate-300">
          当前 Prometheus：<span :class="status.prometheus_configured ? 'text-emerald-100' : 'text-slate-400'">{{ status.prometheus_configured ? '已配置' : '未配置' }}</span>
        </div>
        <div class="grid items-start gap-4 md:grid-cols-2">
          <FormField label="Dashboard 地址"><input v-model="form.dashboard_addr" placeholder="127.0.0.1" /></FormField>
          <FormField label="Dashboard 端口"><input v-model="form.dashboard_port" max="65535" min="1" placeholder="留空关闭" type="number" /></FormField>
          <FormField label="Dashboard 用户"><input v-model="form.dashboard_user" autocomplete="username" placeholder="admin" /></FormField>
          <FormField label="Dashboard 密码" note="留空表示不修改"><input v-model="form.dashboard_password" autocomplete="new-password" placeholder="留空表示不修改" type="password" /></FormField>
          <FormField label="启用 Prometheus 长期统计"><SelectField v-model="form.enable_prometheus" :options="panelTlsEnabledOptions" /></FormField>
        </div>
      </section>

      <section class="rounded-3xl border border-white/10 bg-white/[0.03] p-4">
        <div class="mb-4 flex flex-col gap-1">
          <h2 class="text-lg font-bold text-white">frps 版本差异</h2>
          <p class="text-sm text-slate-400">当前面板固定使用 0.69.1，初始框架使用 0.62.1。</p>
        </div>
        <div class="grid gap-3 text-sm text-slate-300 md:grid-cols-2 xl:grid-cols-3">
          <div class="rounded-2xl border border-white/10 bg-slate-950/40 p-3"><span class="text-cyan-100">0.65.0</span> 新增更细的 Prometheus proxy 指标。</div>
          <div class="rounded-2xl border border-white/10 bg-slate-950/40 p-3"><span class="text-cyan-100">0.67.0</span> 新增 clientID 和客户端连接状态。</div>
          <div class="rounded-2xl border border-white/10 bg-slate-950/40 p-3"><span class="text-cyan-100">0.68.0</span> 新增 frpc built-in store 和管理 API。</div>
          <div class="rounded-2xl border border-white/10 bg-slate-950/40 p-3"><span class="text-cyan-100">0.68.1</span> 修复 HTTP proxy auth bypass。</div>
          <div class="rounded-2xl border border-white/10 bg-slate-950/40 p-3"><span class="text-cyan-100">0.69.0</span> 新增 transport.wireProtocol v1/v2。</div>
          <div class="rounded-2xl border border-white/10 bg-slate-950/40 p-3"><span class="text-cyan-100">0.69.1</span> v2 扩展到 UDP/SUDP payload。</div>
        </div>
      </section>

      <div class="flex flex-wrap gap-3">
        <button class="btn-primary" type="submit">保存配置</button>
        <ConfirmButton class-name="btn-danger" :busy="status?.restarting" :message="status?.restarting ? 'frps 正在重启' : '重启 frps 会影响所有隧道连接，确定继续吗？'" @confirm="restart">{{ status?.restarting ? '重启中' : '重启 frps' }}</ConfirmButton>
      </div>
    </form>
  </section>

  <section class="card mt-6 p-6">
    <form class="grid gap-5" @submit.prevent="saveCaddy">
      <div>
        <h2 class="text-lg font-bold text-white">Caddy 域名入口</h2>
        <p class="mt-1 text-sm text-slate-400">启用后公网 80/443 由 Caddy 监听，只通过这里填写的域名转发到面板。</p>
      </div>
      <div v-if="caddy" class="rounded-3xl border border-white/10 bg-white/[0.03] p-4 text-sm text-slate-300">
        <div>当前状态：{{ caddy.enabled ? '启用' : '关闭' }}</div>
        <div>访问域名：<code class="text-cyan-100">{{ caddy.domain || '未配置' }}</code></div>
        <div>反代后端：<code class="text-cyan-100">{{ caddy.upstream }}</code></div>
        <div>面板监听：<code class="text-cyan-100">{{ caddy.app_bind }}</code></div>
        <div>配置路径：<code class="text-cyan-100">{{ caddy.config_path }}</code></div>
        <div>命令状态：<span :class="caddy.available ? 'text-emerald-100' : 'text-red-100'">{{ caddy.available ? 'Caddy 可用' : 'Caddy 未安装或不可用' }}</span></div>
        <div>暴露状态：<span :class="caddy.app_bind_local ? 'text-emerald-100' : 'text-red-100'">{{ caddy.app_bind_local ? '面板已仅监听本机' : '面板仍可能公网暴露，请把 APP_BIND 改为 127.0.0.1:8080 后重启面板' }}</span></div>
      </div>
      <div class="grid items-start gap-4 md:grid-cols-2">
        <FormField label="启用 Caddy 接管"><SelectField v-model="caddyForm.enabled" :options="caddyEnabledOptions" /></FormField>
        <FormField label="面板访问域名" note="例如 frp.let2.dev，不允许带协议、端口或路径"><input v-model="caddyForm.domain" placeholder="frp.let2.dev" /></FormField>
      </div>
      <div class="rounded-2xl border border-cyan-300/20 bg-cyan-400/10 px-4 py-3 text-sm text-cyan-100">
        启用后请把 APP_BIND 改为 127.0.0.1:8080 并重启面板，避免公网 8080 暴露。Caddy 接管 443 后，frps 的 HTTPS vhost 端口不能同时使用 443。
      </div>
      <div class="flex flex-wrap gap-3">
        <button class="btn-primary" type="submit">保存并重载 Caddy</button>
        <button class="btn-secondary" type="button" @click="reloadCaddyConfig">重新重载 Caddy</button>
      </div>
    </form>
  </section>

  <section class="card mt-6 p-6">
    <form class="grid gap-5" @submit.prevent="savePanelTls">
      <div>
        <h2 class="text-lg font-bold text-white">直接 HTTPS（不推荐）</h2>
        <p class="mt-1 text-sm text-slate-400">由面板进程直接监听 HTTPS。启用 Caddy 接管后，不建议继续让面板占用公网 443。</p>
      </div>
      <div v-if="panelTls" class="rounded-3xl border border-white/10 bg-white/[0.03] p-4 text-sm text-slate-300">
        <div>当前状态：{{ panelTls.enabled ? '启用' : '关闭' }}</div>
        <div>访问域名：<code class="text-cyan-100">{{ panelTls.domain || '未配置' }}</code></div>
        <div>监听地址：<code class="text-cyan-100">{{ panelTls.bind }}</code></div>
        <div>证书域名：<span class="text-cyan-100">{{ panelTls.domains.length ? panelTls.domains.join(', ') : '未上传' }}</span></div>
        <div v-if="panelTls.not_after">过期时间：{{ panelTls.not_after }}</div>
      </div>
      <div v-if="caddyForm.enabled" class="rounded-2xl border border-amber-300/20 bg-amber-300/10 px-4 py-3 text-sm text-amber-100">
        Caddy 已启用，直接 HTTPS 已收起。需要切回面板直接监听 HTTPS 时，先关闭 Caddy 接管。
      </div>
      <template v-else>
        <div class="grid items-start gap-4 md:grid-cols-2">
          <FormField label="启用 HTTPS"><SelectField v-model="panelTlsForm.enabled" :options="panelTlsEnabledOptions" /></FormField>
          <FormField label="访问域名" note="例如 frp.let2.dev，证书必须覆盖这个域名"><input v-model="panelTlsForm.domain" placeholder="frp.let2.dev" /></FormField>
          <FormField label="HTTPS 监听地址" note="必须是 IP:端口，例如 0.0.0.0:443"><input v-model="panelTlsForm.bind" required placeholder="0.0.0.0:443" /></FormField>
          <FormField label="证书 PEM"><textarea v-model="panelTlsForm.certificate_pem" rows="6" placeholder="留空表示不修改" /></FormField>
          <FormField label="私钥 PEM"><textarea v-model="panelTlsForm.private_key_pem" rows="6" placeholder="留空表示不修改" /></FormField>
        </div>
        <div class="flex flex-wrap gap-3">
          <button class="btn-primary" type="submit">保存面板 HTTPS</button>
          <ConfirmButton class-name="btn-danger" message="重启面板会短暂中断管理页面，确定继续吗？" @confirm="restartPanelService">重启面板</ConfirmButton>
        </div>
      </template>
    </form>
  </section>
</template>

export interface PublicUser {
  id: string
  username: string
  role: 'admin' | 'user' | string
  disabled: boolean
  created_at: string
  max_tunnels: number | null
}

export interface SessionResponse {
  user: PublicUser
}

export interface Tunnel {
  id: string
  user_id: string
  name: string
  protocol: 'tcp' | 'udp' | 'http' | 'https' | string
  local_host: string
  local_port: number
  remote_port: number | null
  custom_domain: string | null
  tls_mode: string | null
  certificate_id: string | null
  use_encryption: boolean
  use_compression: boolean
  bandwidth_limit: string | null
  bandwidth_limit_mode: string | null
  proxy_protocol_version: string | null
  locations: string | null
  host_header_rewrite: string | null
  updated_at: string
  config_changed_at: string
  last_config_viewed_at: string | null
  last_config_downloaded_at: string | null
  config_version: number
  config_stale: boolean
  created_at: string
}

export interface TunnelWithTraffic {
  tunnel: Tunnel
  traffic_available: boolean
  traffic_in: number
  traffic_out: number
  runtime_status: string
  current_connections: number
  matched_proxy_name: string | null
  persistent_traffic_available: boolean
  persistent_traffic_in: number
  persistent_traffic_out: number
  last_sampled_at: string | null
}

export interface TrafficHistoryPoint {
  traffic_in: number
  traffic_out: number
  sampled_at: string
}

export interface TrafficHistoryResponse {
  points: TrafficHistoryPoint[]
}

export interface FrpcResponse {
  tunnel: Tunnel
  frpc_toml: string
}

export interface Invite {
  id: string
  code: string
  created_by: string
  used_by: string | null
  used_at: string | null
  expires_at: string | null
  created_at: string
}

export interface UserRow {
  user: PublicUser
  tunnel_count: number
  effective_max_tunnels: number
}

export interface AdminTunnelRow {
  tunnel: Tunnel
  username: string
}

export interface AdminTrafficSummary {
  available: boolean
  persistent_available: boolean
  total_traffic_in: number
  total_traffic_out: number
  persistent_total_traffic_in: number
  persistent_total_traffic_out: number
  last_sampled_at: string | null
  tunnels: AdminTunnelTraffic[]
}

export interface AdminTunnelTraffic {
  tunnel: Tunnel
  username: string
  traffic_in: number
  traffic_out: number
  runtime_status: string
  current_connections: number
  matched_proxy_name: string | null
  persistent_traffic_available: boolean
  persistent_traffic_in: number
  persistent_traffic_out: number
  last_sampled_at: string | null
}

export interface AuditLog {
  id: string
  actor_user_id: string | null
  actor_username: string | null
  actor_role: string | null
  action: string
  resource_type: string
  resource_id: string | null
  resource_name: string | null
  outcome: string
  message: string | null
  metadata_json: string | null
  ip_address: string | null
  user_agent: string | null
  created_at: string
}

export interface PageResponse<T> {
  items: T[]
  total: number
  page: number
  page_size: number
}

export interface ListParams {
  q?: string
  status?: string
  page?: number
  page_size?: number
}

export interface AuditLogParams {
  q?: string
  action?: string
  resource_type?: string
  outcome?: string
  page?: number
  page_size?: number
}

export interface AdminSummary {
  user_count: number
  disabled_user_count: number
  tunnel_count: number
  invite_count: number
  unused_invite_count: number
  used_remote_port_count: number
  remote_port_capacity: number
}

export interface FrpsStatus {
  server_addr: string
  bind_port: number
  token_set: boolean
  remote_port_min: number
  remote_port_max: number
  config_path: string
  status: string
  state: 'running' | 'stopped' | 'restarting' | 'unknown' | string
  version: string
  display_status: string
  restarting: boolean
  restart_command_configured: boolean
  dashboard_addr: string
  dashboard_port: number | null
  dashboard_user: string
  dashboard_configured: boolean
  dashboard_available: boolean
  enable_prometheus: boolean
  prometheus_configured: boolean
  vhost_http_port: number | null
  vhost_https_port: number | null
}

export interface FrpsUpdate {
  server_addr: string
  bind_port: number
  auth_token: string
  remote_port_min: number
  remote_port_max: number
  dashboard_addr: string
  dashboard_port: number | null
  dashboard_user: string
  dashboard_password: string
  enable_prometheus: boolean
  vhost_http_port: number | null
  vhost_https_port: number | null
}

export interface CertificateInfo {
  id: string
  name: string
  domains: string[]
  not_before: string
  not_after: string
  fingerprint_sha256: string
  created_at: string
}

export interface PanelTlsStatus {
  enabled: boolean
  bind: string
  domain: string
  domains: string[]
  not_after: string | null
  fingerprint_sha256: string
}

export interface CaddyStatus {
  enabled: boolean
  domain: string
  config_path: string
  available: boolean
  upstream: string
  app_bind: string
  app_bind_local: boolean
}

export interface ConfigResponse {
  frps_server_addr: string
  frps_bind_port: number
  remote_port_min: number
  remote_port_max: number
  user_max_tunnels: number
}

export interface DashboardSummary {
  tunnel_count: number
  user_max_tunnels: number
  remaining_tunnels: number
  username: string
  role: string
  disabled: boolean
  created_at: string
  effective_max_tunnels: number
  frps_server_addr: string
  frps_bind_port: number
  remote_port_min: number
  remote_port_max: number
  vhost_http_port: number | null
  vhost_https_port: number | null
}

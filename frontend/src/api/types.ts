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
  protocol: 'tcp' | 'udp' | string
  local_host: string
  local_port: number
  remote_port: number
  created_at: string
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
  upgrade_supported: boolean
  available_versions: string[]
}

export interface FrpsUpdate {
  server_addr: string
  bind_port: number
  auth_token: string
  remote_port_min: number
  remote_port_max: number
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
  frps_server_addr: string
  frps_bind_port: number
  remote_port_min: number
  remote_port_max: number
}

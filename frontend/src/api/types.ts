export interface PublicUser {
  id: string
  username: string
  role: 'admin' | 'user' | string
  disabled: boolean
  created_at: string
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
}

export interface ConfigResponse {
  frps_server_addr: string
  frps_bind_port: number
  remote_port_min: number
  remote_port_max: number
  user_max_tunnels: number
}

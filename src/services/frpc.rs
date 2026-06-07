use crate::{
    config::Config,
    entities::{tunnels, users},
};

pub fn render_frpc_toml(config: &Config, user: &users::Model, tunnel: &tunnels::Model) -> String {
    format!(
        r#"serverAddr = "{server_addr}"
serverPort = {server_port}

[auth]
method = "token"
token = "{token}"

user = "{username}"

[[proxies]]
name = "{proxy_name}"
type = "tcp"
localIP = "{local_host}"
localPort = {local_port}
remotePort = {remote_port}
"#,
        server_addr = config.frps_server_addr,
        server_port = config.frps_bind_port,
        token = config.frps_auth_token,
        username = user.username,
        proxy_name = tunnel.name,
        local_host = tunnel.local_host,
        local_port = tunnel.local_port,
        remote_port = tunnel.remote_port,
    )
}

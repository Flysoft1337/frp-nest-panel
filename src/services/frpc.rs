use crate::{
    entities::{tunnels, users},
    services::frps::FrpsRuntimeConfig,
};

pub fn render_frpc_toml(
    config: &FrpsRuntimeConfig,
    user: &users::Model,
    tunnel: &tunnels::Model,
) -> String {
    format!(
        r#"serverAddr = "{server_addr}"
serverPort = {server_port}

[auth]
method = "token"
token = "{token}"

user = "{username}"

[[proxies]]
name = "{proxy_name}"
type = "{protocol}"
localIP = "{local_host}"
localPort = {local_port}
remotePort = {remote_port}
"#,
        server_addr = config.server_addr,
        server_port = config.bind_port,
        token = config.auth_token,
        username = user.username,
        proxy_name = tunnel.name,
        protocol = tunnel.protocol,
        local_host = tunnel.local_host,
        local_port = tunnel.local_port,
        remote_port = tunnel.remote_port,
    )
}

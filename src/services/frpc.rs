use serde::Serialize;

use crate::{
    entities::{tunnels, users},
    services::frps::FrpsRuntimeConfig,
};

#[derive(Serialize)]
struct FrpcToml<'a> {
    #[serde(rename = "serverAddr")]
    server_addr: &'a str,
    #[serde(rename = "serverPort")]
    server_port: u16,
    auth: FrpcTomlAuth<'a>,
    user: &'a str,
    proxies: Vec<FrpcTomlProxy<'a>>,
}

#[derive(Serialize)]
struct FrpcTomlAuth<'a> {
    method: &'a str,
    token: &'a str,
}

#[derive(Serialize)]
struct FrpcTomlProxy<'a> {
    name: &'a str,
    #[serde(rename = "type")]
    protocol: &'a str,
    #[serde(rename = "localIP", skip_serializing_if = "Option::is_none")]
    local_ip: Option<&'a str>,
    #[serde(rename = "localPort", skip_serializing_if = "Option::is_none")]
    local_port: Option<i32>,
    #[serde(rename = "remotePort", skip_serializing_if = "Option::is_none")]
    remote_port: Option<i32>,
    #[serde(rename = "customDomains", skip_serializing_if = "Option::is_none")]
    custom_domains: Option<Vec<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plugin: Option<FrpcTomlPlugin>,
}

#[derive(Serialize)]
struct FrpcTomlPlugin {
    #[serde(rename = "type")]
    plugin_type: &'static str,
    #[serde(rename = "localAddr")]
    local_addr: String,
    #[serde(rename = "crtPath")]
    crt_path: &'static str,
    #[serde(rename = "keyPath")]
    key_path: &'static str,
}

pub fn proxy_names(username: &str, tunnel_name: &str) -> Vec<String> {
    let user_proxy = format!("{username}.{tunnel_name}");
    if username.is_empty() || user_proxy == tunnel_name {
        vec![tunnel_name.to_owned()]
    } else {
        vec![user_proxy, tunnel_name.to_owned()]
    }
}

pub fn proxy_key(protocol: &str, proxy_name: &str) -> (String, String) {
    (protocol.to_owned(), proxy_name.to_owned())
}

pub fn render_frpc_toml(
    config: &FrpsRuntimeConfig,
    user: &users::Model,
    tunnel: &tunnels::Model,
) -> String {
    let proxy = match tunnel.protocol.as_str() {
        "http" => domain_proxy(tunnel, None),
        "https" if tunnel.tls_mode.as_deref() == Some("uploaded_cert") => domain_proxy(
            tunnel,
            Some(FrpcTomlPlugin {
                plugin_type: "https2http",
                local_addr: format!("{}:{}", tunnel.local_host, tunnel.local_port),
                crt_path: "./cert.pem",
                key_path: "./key.pem",
            }),
        ),
        "https" => domain_proxy(tunnel, None),
        _ => FrpcTomlProxy {
            name: &tunnel.name,
            protocol: &tunnel.protocol,
            local_ip: Some(&tunnel.local_host),
            local_port: Some(tunnel.local_port),
            remote_port: tunnel.remote_port,
            custom_domains: None,
            plugin: None,
        },
    };

    toml::to_string_pretty(&FrpcToml {
        server_addr: &config.server_addr,
        server_port: config.bind_port,
        auth: FrpcTomlAuth {
            method: "token",
            token: &config.auth_token,
        },
        user: &user.username,
        proxies: vec![proxy],
    })
    .unwrap_or_default()
}

fn custom_domains(value: &str) -> Vec<&str> {
    value
        .split(',')
        .map(str::trim)
        .filter(|domain| !domain.is_empty())
        .collect()
}

fn domain_proxy<'a>(
    tunnel: &'a tunnels::Model,
    plugin: Option<FrpcTomlPlugin>,
) -> FrpcTomlProxy<'a> {
    let uploaded_cert = plugin.is_some();
    FrpcTomlProxy {
        name: &tunnel.name,
        protocol: &tunnel.protocol,
        local_ip: (!uploaded_cert).then_some(tunnel.local_host.as_str()),
        local_port: (!uploaded_cert).then_some(tunnel.local_port),
        remote_port: None,
        custom_domains: tunnel.custom_domain.as_deref().map(custom_domains),
        plugin,
    }
}

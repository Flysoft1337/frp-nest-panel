use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use chrono::{DateTime, Duration as ChronoDuration, FixedOffset, Utc};
use sea_orm::DbErr;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use uuid::Uuid;

use crate::{
    entities::{traffic_samples, tunnels, users},
    services::{frpc, frps::FrpsRuntimeConfig},
    state::AppState,
};

#[derive(Clone)]
pub struct PersistentTraffic {
    pub traffic_in: u64,
    pub traffic_out: u64,
    pub sampled_at: DateTime<FixedOffset>,
}

pub struct TrafficHistoryPoint {
    pub traffic_in: u64,
    pub traffic_out: u64,
    pub sampled_at: DateTime<FixedOffset>,
}

#[derive(Clone, Copy)]
pub enum TrafficHistoryRange {
    OneHour,
    OneDay,
    SevenDays,
}

impl TrafficHistoryRange {
    pub fn parse(value: Option<&str>) -> Option<Self> {
        match value.unwrap_or("24h") {
            "1h" => Some(Self::OneHour),
            "24h" => Some(Self::OneDay),
            "7d" => Some(Self::SevenDays),
            _ => None,
        }
    }

    fn duration(self) -> ChronoDuration {
        match self {
            Self::OneHour => ChronoDuration::hours(1),
            Self::OneDay => ChronoDuration::days(1),
            Self::SevenDays => ChronoDuration::days(7),
        }
    }
}

#[derive(Default)]
struct ProxyTrafficCounter {
    traffic_in: Option<u64>,
    traffic_out: Option<u64>,
}

struct MetricLine {
    name: String,
    labels: HashMap<String, String>,
    value: u64,
}

pub async fn run_collector(state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        if let Err(error) = collect_once(&state).await {
            tracing::warn!(%error, "failed to collect frps prometheus traffic");
        }
    }
}

pub async fn latest_by_tunnel(
    db: &DatabaseConnection,
    tunnel_ids: &[Uuid],
) -> std::result::Result<HashMap<Uuid, PersistentTraffic>, DbErr> {
    if tunnel_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let samples = traffic_samples::Entity::find()
        .filter(traffic_samples::Column::TunnelId.is_in(tunnel_ids.to_vec()))
        .order_by_desc(traffic_samples::Column::SampledAt)
        .all(db)
        .await?;
    let mut latest = HashMap::new();
    for sample in samples {
        let Some(tunnel_id) = sample.tunnel_id else {
            continue;
        };
        latest
            .entry(tunnel_id)
            .or_insert_with(|| PersistentTraffic {
                traffic_in: u64::try_from(sample.traffic_in).unwrap_or(0),
                traffic_out: u64::try_from(sample.traffic_out).unwrap_or(0),
                sampled_at: sample.sampled_at,
            });
    }
    Ok(latest)
}

pub async fn history_by_tunnel(
    db: &DatabaseConnection,
    tunnel_id: Uuid,
    range: TrafficHistoryRange,
) -> std::result::Result<Vec<TrafficHistoryPoint>, DbErr> {
    let since = (Utc::now() - range.duration()).fixed_offset();
    let samples = traffic_samples::Entity::find()
        .filter(traffic_samples::Column::TunnelId.eq(tunnel_id))
        .filter(traffic_samples::Column::SampledAt.gte(since))
        .order_by_asc(traffic_samples::Column::SampledAt)
        .all(db)
        .await?;
    Ok(samples
        .into_iter()
        .map(|sample| TrafficHistoryPoint {
            traffic_in: u64::try_from(sample.traffic_in).unwrap_or(0),
            traffic_out: u64::try_from(sample.traffic_out).unwrap_or(0),
            sampled_at: sample.sampled_at,
        })
        .collect())
}

async fn collect_once(state: &AppState) -> Result<()> {
    let config = state.frps.read().await.clone();
    if config.dashboard_port.is_none() || !config.enable_prometheus {
        return Ok(());
    }

    let metrics = fetch_metrics(&config).await?;
    let counters = parse_proxy_traffic(&metrics);
    if counters.is_empty() {
        return Ok(());
    }

    let tunnels = tunnels::Entity::find().all(&state.db).await?;
    let users = users::Entity::find().all(&state.db).await?;
    let usernames = users
        .into_iter()
        .map(|user| (user.id, user.username))
        .collect::<HashMap<_, _>>();
    let tunnel_lookup = tunnels
        .into_iter()
        .flat_map(|tunnel| {
            let username = usernames.get(&tunnel.user_id).cloned().unwrap_or_default();
            frpc::proxy_names(&username, &tunnel.name)
                .into_iter()
                .map(move |proxy_name| (frpc::proxy_key(&tunnel.protocol, &proxy_name), tunnel.id))
        })
        .collect::<HashMap<_, _>>();
    let sampled_at = Utc::now().fixed_offset();

    for ((protocol, proxy_name), counter) in counters {
        let Some(traffic_in) = counter.traffic_in else {
            continue;
        };
        let Some(traffic_out) = counter.traffic_out else {
            continue;
        };
        let traffic_in = i64::try_from(traffic_in).unwrap_or(i64::MAX);
        let traffic_out = i64::try_from(traffic_out).unwrap_or(i64::MAX);
        traffic_samples::ActiveModel {
            id: Set(Uuid::new_v4()),
            tunnel_id: Set(tunnel_lookup
                .get(&(protocol.clone(), proxy_name.clone()))
                .copied()),
            protocol: Set(protocol),
            proxy_name: Set(proxy_name),
            traffic_in: Set(traffic_in),
            traffic_out: Set(traffic_out),
            sampled_at: Set(sampled_at),
        }
        .insert(&state.db)
        .await?;
    }

    Ok(())
}

async fn fetch_metrics(config: &FrpsRuntimeConfig) -> Result<String> {
    let Some(port) = config.dashboard_port else {
        anyhow::bail!("frps dashboard is not configured");
    };
    let url = format!("http://{}:{port}/metrics", config.dashboard_addr);
    let client = reqwest::Client::new();
    let mut request = client.get(url).timeout(Duration::from_secs(2));
    if !config.dashboard_user.is_empty() {
        request = request.basic_auth(&config.dashboard_user, Some(&config.dashboard_password));
    }
    let response = request.send().await?;
    if !response.status().is_success() {
        anyhow::bail!("frps prometheus returned {}", response.status());
    }
    Ok(response.text().await?)
}

fn parse_proxy_traffic(metrics: &str) -> HashMap<(String, String), ProxyTrafficCounter> {
    let mut counters: HashMap<(String, String), ProxyTrafficCounter> = HashMap::new();
    for line in metrics.lines().filter_map(parse_metric_line) {
        let Some(protocol) = label_value(&line.labels, &["type", "proxy_type", "protocol"]) else {
            continue;
        };
        let Some(proxy_name) = label_value(&line.labels, &["name", "proxy_name", "proxy"]) else {
            continue;
        };
        let Some(direction) = traffic_direction(&line.name, &line.labels) else {
            continue;
        };
        let counter = counters
            .entry((protocol.to_owned(), proxy_name.to_owned()))
            .or_default();
        match direction {
            TrafficDirection::In => counter.traffic_in = Some(line.value),
            TrafficDirection::Out => counter.traffic_out = Some(line.value),
        }
    }
    counters
}

fn parse_metric_line(line: &str) -> Option<MetricLine> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let (metric, value) = line.split_once(char::is_whitespace)?;
    let value = value.split_whitespace().next()?.parse::<f64>().ok()?;
    if !value.is_finite() || value < 0.0 {
        return None;
    }
    let value = value.trunc() as u64;
    let (name, labels) = match metric.split_once('{') {
        Some((name, rest)) => (name.to_owned(), parse_labels(rest.trim_end_matches('}'))),
        None => (metric.to_owned(), HashMap::new()),
    };
    Some(MetricLine {
        name,
        labels,
        value,
    })
}

fn parse_labels(labels: &str) -> HashMap<String, String> {
    let mut values = HashMap::new();
    let mut key = String::new();
    let mut value = String::new();
    let mut reading_key = true;
    let mut in_quotes = false;
    let mut escaped = false;

    for character in labels.chars() {
        if reading_key {
            if character == '=' {
                reading_key = false;
            } else if character != ',' {
                key.push(character);
            }
            continue;
        }

        if escaped {
            value.push(character);
            escaped = false;
            continue;
        }
        match character {
            '\\' if in_quotes => escaped = true,
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                if !key.trim().is_empty() {
                    values.insert(key.trim().to_owned(), value.clone());
                }
                key.clear();
                value.clear();
                reading_key = true;
            }
            _ => value.push(character),
        }
    }
    if !key.trim().is_empty() {
        values.insert(key.trim().to_owned(), value);
    }
    values
}

#[derive(Clone, Copy)]
enum TrafficDirection {
    In,
    Out,
}

fn traffic_direction(
    metric_name: &str,
    labels: &HashMap<String, String>,
) -> Option<TrafficDirection> {
    if !metric_name.contains("traffic") && !metric_name.contains("bytes") {
        return None;
    }
    if let Some(direction) = label_value(labels, &["direction", "side"]) {
        return match direction {
            "in" | "read" | "rx" => Some(TrafficDirection::In),
            "out" | "write" | "tx" => Some(TrafficDirection::Out),
            _ => None,
        };
    }
    if metric_name.contains("traffic_in")
        || metric_name.contains("traffic_read")
        || metric_name.contains("bytes_in")
        || metric_name.contains("read_bytes")
    {
        return Some(TrafficDirection::In);
    }
    if metric_name.contains("traffic_out")
        || metric_name.contains("traffic_write")
        || metric_name.contains("bytes_out")
        || metric_name.contains("write_bytes")
    {
        return Some(TrafficDirection::Out);
    }
    None
}

fn label_value<'a>(labels: &'a HashMap<String, String>, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| labels.get(*key).map(String::as_str))
        .filter(|value| !value.is_empty())
}

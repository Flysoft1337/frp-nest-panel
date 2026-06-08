use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use openssl::{
    hash::MessageDigest,
    nid::Nid,
    pkey::{PKey, Private},
    x509::X509,
};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    services::validation,
};

const CERTIFICATE_DIR: &str = "data/certificates";
const MAX_CERT_PEM_SIZE: usize = 64 * 1024;
const MAX_KEY_PEM_SIZE: usize = 64 * 1024;

pub struct ParsedCertificate {
    pub domains: Vec<String>,
    pub not_before: DateTime<chrono::FixedOffset>,
    pub not_after: DateTime<chrono::FixedOffset>,
    pub fingerprint_sha256: String,
}

pub fn parse_and_validate(
    certificate_pem: &str,
    private_key_pem: &str,
) -> AppResult<ParsedCertificate> {
    if certificate_pem.len() > MAX_CERT_PEM_SIZE || private_key_pem.len() > MAX_KEY_PEM_SIZE {
        return Err(AppError::BadRequest("证书或私钥文件过大".to_owned()));
    }

    let cert = X509::from_pem(certificate_pem.as_bytes())
        .map_err(|_| AppError::BadRequest("证书 PEM 格式无效".to_owned()))?;
    let key = PKey::private_key_from_pem(private_key_pem.as_bytes())
        .map_err(|_| AppError::BadRequest("私钥 PEM 格式无效".to_owned()))?;
    validate_key_matches(&cert, &key)?;

    let not_before = openssl_time_to_chrono(cert.not_before().to_string().as_str())?;
    let not_after = openssl_time_to_chrono(cert.not_after().to_string().as_str())?;
    let now = Utc::now().fixed_offset();
    if not_before > now || not_after <= now {
        return Err(AppError::BadRequest("证书不在有效期内".to_owned()));
    }

    let domains = certificate_domains(&cert)?;
    let fingerprint = cert
        .digest(MessageDigest::sha256())
        .map_err(|_| AppError::BadRequest("证书指纹计算失败".to_owned()))?
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();

    Ok(ParsedCertificate {
        domains,
        not_before,
        not_after,
        fingerprint_sha256: fingerprint,
    })
}

pub async fn write_certificate_files(
    user_id: Uuid,
    certificate_id: Uuid,
    certificate_pem: &str,
    private_key_pem: &str,
) -> Result<(String, String)> {
    let dir = certificate_dir(user_id, certificate_id);
    tokio::fs::create_dir_all(&dir)
        .await
        .with_context(|| format!("failed to create {}", dir.display()))?;
    let cert_path = dir.join("cert.pem");
    let key_path = dir.join("key.pem");
    tokio::fs::write(&cert_path, certificate_pem)
        .await
        .with_context(|| format!("failed to write {}", cert_path.display()))?;
    tokio::fs::write(&key_path, private_key_pem)
        .await
        .with_context(|| format!("failed to write {}", key_path.display()))?;
    Ok((
        cert_path.to_string_lossy().into_owned(),
        key_path.to_string_lossy().into_owned(),
    ))
}

pub async fn remove_certificate_files(user_id: Uuid, certificate_id: Uuid) -> Result<()> {
    let dir = certificate_dir(user_id, certificate_id);
    if tokio::fs::try_exists(&dir).await.unwrap_or(false) {
        tokio::fs::remove_dir_all(&dir)
            .await
            .with_context(|| format!("failed to remove {}", dir.display()))?;
    }
    Ok(())
}

pub async fn read_certificate_bundle(
    cert_path: &str,
    key_path: &str,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let cert = tokio::fs::read(cert_path)
        .await
        .with_context(|| format!("failed to read {cert_path}"))?;
    let key = tokio::fs::read(key_path)
        .await
        .with_context(|| format!("failed to read {key_path}"))?;
    Ok((cert, key))
}

pub fn domains_from_json(value: &str) -> Vec<String> {
    serde_json::from_str(value).unwrap_or_default()
}

pub fn certificate_covers_domain(domains: &[String], domain: &str) -> bool {
    domains.iter().any(|cert_domain| {
        cert_domain == domain
            || cert_domain
                .strip_prefix("*.")
                .map(|suffix| {
                    domain.ends_with(suffix)
                        && domain.len() > suffix.len()
                        && domain[..domain.len() - suffix.len()]
                            .trim_end_matches('.')
                            .contains('.')
                            == false
                })
                .unwrap_or(false)
    })
}

fn certificate_dir(user_id: Uuid, certificate_id: Uuid) -> PathBuf {
    PathBuf::from(CERTIFICATE_DIR)
        .join(user_id.to_string())
        .join(certificate_id.to_string())
}

fn validate_key_matches(cert: &X509, key: &PKey<Private>) -> AppResult<()> {
    let public_key = cert
        .public_key()
        .map_err(|_| AppError::BadRequest("证书公钥读取失败".to_owned()))?;
    if !public_key.public_eq(key) {
        return Err(AppError::BadRequest("证书和私钥不匹配".to_owned()));
    }
    Ok(())
}

fn certificate_domain(value: &str) -> AppResult<String> {
    if let Some(domain) = value.trim().strip_prefix("*.") {
        return validation::domain(domain).map(|domain| format!("*.{domain}"));
    }
    validation::domain(value)
}

fn certificate_domains(cert: &X509) -> AppResult<Vec<String>> {
    let mut domains = Vec::new();
    if let Some(names) = cert.subject_alt_names() {
        for name in names {
            if let Some(dns) = name.dnsname() {
                domains.push(certificate_domain(dns)?);
            }
        }
    }
    if domains.is_empty() {
        for entry in cert.subject_name().entries_by_nid(Nid::COMMONNAME) {
            let cn = entry
                .data()
                .as_utf8()
                .map_err(|_| AppError::BadRequest("证书 CN 无法读取".to_owned()))?;
            domains.push(certificate_domain(cn.as_ref())?);
        }
    }
    domains.sort();
    domains.dedup();
    if domains.is_empty() {
        return Err(AppError::BadRequest("证书没有可用域名".to_owned()));
    }
    Ok(domains)
}

fn openssl_time_to_chrono(value: &str) -> AppResult<DateTime<chrono::FixedOffset>> {
    NaiveDateTime::parse_from_str(value, "%b %e %H:%M:%S %Y GMT")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%b %d %H:%M:%S %Y GMT"))
        .map(|time| time.and_utc().fixed_offset())
        .map_err(|_| AppError::BadRequest("证书有效期无法解析".to_owned()))
}

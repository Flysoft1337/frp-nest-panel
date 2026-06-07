use crate::error::{AppError, AppResult};

pub fn username(value: &str) -> AppResult<String> {
    let value = value.trim();
    if !(3..=32).contains(&value.len()) {
        return Err(AppError::BadRequest("用户名长度必须为 3-32 位".to_owned()));
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err(AppError::BadRequest(
            "用户名只能包含字母、数字、下划线和短横线".to_owned(),
        ));
    }
    Ok(value.to_owned())
}

pub fn tunnel_name(value: &str) -> AppResult<String> {
    let value = value.trim();
    if !(1..=64).contains(&value.len()) {
        return Err(AppError::BadRequest(
            "隧道名称长度必须为 1-64 位".to_owned(),
        ));
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-' || byte == b'.')
    {
        return Err(AppError::BadRequest(
            "隧道名称只能包含字母、数字、下划线、短横线和点".to_owned(),
        ));
    }
    Ok(value.to_owned())
}

pub fn local_host(value: &str) -> AppResult<String> {
    let value = value.trim();
    let value = if value.is_empty() { "127.0.0.1" } else { value };
    if value.len() > 255 {
        return Err(AppError::BadRequest("本地地址过长".to_owned()));
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':'))
    {
        return Err(AppError::BadRequest("本地地址包含不支持的字符".to_owned()));
    }
    Ok(value.to_owned())
}

pub fn local_port(value: i32) -> AppResult<i32> {
    if !(1..=65535).contains(&value) {
        return Err(AppError::BadRequest("本地端口不合法".to_owned()));
    }
    Ok(value)
}

pub fn password(value: &str) -> AppResult<()> {
    if value.len() < 8 {
        return Err(AppError::BadRequest("密码至少 8 位".to_owned()));
    }
    Ok(())
}

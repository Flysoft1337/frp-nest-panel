use std::sync::Arc;

use axum::response::{Html, IntoResponse};
use minijinja::{context, Environment};

use crate::error::AppResult;

pub fn load_templates() -> Environment<'static> {
    let mut env = Environment::new();
    env.add_template("base.html", include_str!("templates/base.html"))
        .unwrap();
    env.add_template("login.html", include_str!("templates/login.html"))
        .unwrap();
    env.add_template("register.html", include_str!("templates/register.html"))
        .unwrap();
    env.add_template("dashboard.html", include_str!("templates/dashboard.html"))
        .unwrap();
    env.add_template("tunnel_new.html", include_str!("templates/tunnel_new.html"))
        .unwrap();
    env.add_template("admin.html", include_str!("templates/admin.html"))
        .unwrap();
    env.add_template(
        "admin_invites.html",
        include_str!("templates/admin_invites.html"),
    )
    .unwrap();
    env.add_template(
        "admin_users.html",
        include_str!("templates/admin_users.html"),
    )
    .unwrap();
    env.add_template(
        "admin_tunnels.html",
        include_str!("templates/admin_tunnels.html"),
    )
    .unwrap();
    env
}

pub fn render(
    env: &Arc<Environment<'static>>,
    name: &str,
    ctx: minijinja::Value,
) -> AppResult<impl IntoResponse> {
    let template = env.get_template(name)?;
    Ok(Html(template.render(ctx)?))
}

pub fn empty_context() -> minijinja::Value {
    context! {}
}

# frp-nest-panel

[中文](README.md)

`frp-nest-panel` is a lightweight multi-user self-service panel built on top of the official frp project. It does not replace frps/frpc. Instead, it manages users, invite codes, remote port allocation, and `frpc.toml` generation so friends or team members can create TCP/UDP tunnels by themselves.

## Features

- Admin login
- Invite-code registration
- Enable or disable users
- Create and delete TCP/UDP tunnels
- Automatic remote port allocation
- Download generated `frpc.toml`
- Admin views for users, invite codes, and all tunnels
- `/healthz` health check

## Tech Stack

- Rust
- Axum
- SeaORM
- PostgreSQL
- Vue 3
- Vite
- Tailwind CSS
- Official frps/frpc

## Current Limitations

The first version supports TCP/UDP tunnels. The following features are not implemented yet:

- HTTP/HTTPS subdomain tunnels
- Traffic statistics
- Online status detection
- Billing or plans
- OAuth login

## Quick Start: Docker Compose

Copy the environment file:

```bash
cp .env.example .env
```

Edit `.env`:

```env
SESSION_SECRET=change-this-to-a-long-random-secret
INITIAL_ADMIN_USERNAME=admin
INITIAL_ADMIN_PASSWORD=change-this-admin-password
FRPS_SERVER_ADDR=your-server-ip-or-domain
FRPS_AUTH_TOKEN=change-this-frps-token
```

Start the stack:

```bash
docker compose up -d --build
```

Open:

```text
http://127.0.0.1:8080
```

## Binary Deployment

The build machine needs Node.js 20.19+ or 22.12+, plus Rust stable.

Build frontend and backend on a build machine:

```bash
cd frontend
npm ci
npm run build
cd ..
cargo build --release
```

Copy the binary and frontend build output to your server:

```bash
scp target/release/frp-nest-panel root@your-server:/opt/frp-nest-panel/frp-nest-panel
scp -r frontend/dist root@your-server:/opt/frp-nest-panel/frontend/dist
```

Prepare `.env`, then run the binary with systemd or your preferred process manager.

## PVE Build Example

To build on Proxmox VE:

```bash
apt update && apt install -y build-essential pkg-config libssl-dev git curl ca-certificates
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
cd frontend && npm ci && npm run build
cd .. && cargo build --release
```

If your network is slow, configure a Cargo or rustup mirror/proxy.

## Generated frpc.toml Example

After a user creates a tunnel, the panel generates a config similar to:

```toml
serverAddr = "example.com"
serverPort = 7000

[auth]
method = "token"
token = "your-frps-token"

[[proxies]]
name = "alice-mc"
type = "tcp"
localIP = "127.0.0.1"
localPort = 25565
remotePort = 20001
```

## Security Notes

- Do not commit `.env`.
- Do not commit real tokens, passwords, database files, or private keys.
- `SESSION_SECRET` must be at least 32 characters. Use a random value in production.
- Production deployments should run behind an HTTPS reverse proxy.
- Login rate limiting can be handled by a reverse proxy, WAF, or gateway.
- PostgreSQL should not be exposed to the public Internet.

## FAQ

### Does this project modify frps configuration automatically?

Admins can save local frps settings from the panel. Saving writes `frps/frps.toml` but does not restart frps automatically; restart requires manual confirmation in the admin page.

### Can friends create tunnels by themselves?

Yes. An admin creates invite codes, and invited users can register and create their own TCP/UDP tunnels.

### Does it support HTTP/HTTPS tunnels?

Not yet. The current version supports TCP/UDP first. HTTP/HTTPS subdomains and custom domains may be added later.

## License

This project is licensed under the [PolyForm Noncommercial License 1.0.0](LICENSE).

Non-commercial use, modification, and distribution are permitted. Commercial use requires prior written permission from the copyright holder. Copyright and license notices must be preserved.

## Contributing

Issues and pull requests are welcome. By submitting a contribution, you agree to license your contribution under this project's license.

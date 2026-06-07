# frp-lite-panel

[English](README.en.md)

`frp-lite-panel` 是一个基于官方 frp 的轻量多用户自助面板。它不替代 frps/frpc，而是负责用户、邀请码、端口分配和 `frpc.toml` 生成，让朋友或团队成员可以自助创建 TCP 隧道。

## 功能

- 管理员登录
- 邀请码注册
- 用户启用/禁用
- TCP 隧道创建与删除
- 自动分配远程端口
- 下载 `frpc.toml`
- 管理员查看用户、邀请码和全部隧道
- `/healthz` 健康检查

## 技术栈

- Rust
- Axum
- SeaORM
- PostgreSQL
- MiniJinja
- Pico.css
- 官方 frps/frpc

## 当前限制

第一版只支持 TCP 隧道。以下功能暂未实现：

- UDP 隧道
- HTTP/HTTPS 子域名隧道
- 流量统计
- 在线状态检测
- 计费/套餐
- OAuth 登录

## 快速开始：Docker Compose

复制环境变量文件：

```bash
cp .env.example .env
```

修改 `.env`：

```env
SESSION_SECRET=change-this-to-a-long-random-secret
INITIAL_ADMIN_USERNAME=admin
INITIAL_ADMIN_PASSWORD=change-this-admin-password
FRPS_SERVER_ADDR=your-server-ip-or-domain
FRPS_AUTH_TOKEN=change-this-frps-token
```

启动：

```bash
docker compose up -d --build
```

访问：

```text
http://127.0.0.1:8080
```

## 二进制部署

在构建机上编译：

```bash
cargo build --release
```

复制二进制到服务器：

```bash
scp target/release/frp-lite-panel root@your-server:/opt/frp-lite-panel/frp-lite-panel
```

准备 `.env` 后，用 systemd 或你自己的进程管理工具运行。

## PVE 编译示例

如果你想在 PVE 上编译：

```bash
apt update && apt install -y build-essential pkg-config libssl-dev git curl ca-certificates
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
cargo build --release
```

国内网络环境下可以给 Cargo 或 rustup 配置代理/镜像。

## frpc.toml 示例

用户创建隧道后，面板会生成类似配置：

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

## 安全建议

- 不要提交 `.env`。
- 不要提交真实 token、密码、数据库文件、证书私钥。
- `SESSION_SECRET` 至少 32 位，生产环境建议使用随机值。
- 生产环境建议放在 HTTPS 反向代理之后。
- 登录限流可以由反代、WAF 或网关实现。
- PostgreSQL 不建议暴露到公网。

## 常见问题

### 这个项目会自动修改 frps 配置吗？

不会。第一版保持简单：frps 独立运行，面板只负责分配端口和生成 frpc 配置。

### 可以给朋友自助开隧道吗？

可以。管理员创建邀请码，朋友用邀请码注册后即可创建自己的 TCP 隧道。

### 为什么只支持 TCP？

为了先保证最常见的用途稳定，例如 SSH、Minecraft、RDP 或其他 TCP 服务。HTTP/HTTPS/UDP 会在后续版本考虑。

## 许可证

本项目使用 [PolyForm Noncommercial License 1.0.0](LICENSE)。

本项目允许非商业用途自由使用、修改和分发；商业用途需获得版权所有者的事先书面授权。使用、修改或分发本项目时必须保留原始著作权和许可证声明。

## 贡献

欢迎提交 issue 和 pull request。提交贡献即表示你同意按本项目许可证授权你的贡献内容。

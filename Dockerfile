FROM node:22-bookworm-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend ./
RUN npm run build

FROM rust:1-bookworm AS rust-builder
WORKDIR /app
COPY . .
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=rust-builder /app/target/release/frp-nest-panel /app/frp-nest-panel
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist
EXPOSE 8080
CMD ["/app/frp-nest-panel"]

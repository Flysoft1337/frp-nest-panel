FROM rust:1-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/frp-nest-panel /app/frp-nest-panel
COPY src/static /app/src/static
EXPOSE 8080
CMD ["/app/frp-nest-panel"]

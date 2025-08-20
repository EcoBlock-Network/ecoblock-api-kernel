FROM rust:1.73-slim-bullseye as builder
WORKDIR /usr/src/app
COPY . .
RUN apt-get update && apt-get install -y libssl-dev pkg-config libpq-dev ca-certificates && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /usr/src/app
COPY --from=builder /usr/src/app/target/release/ecoblock-api-kernel ./ecoblock-api-kernel
COPY prometheus.yml ./prometheus.yml
EXPOSE 3000
CMD ["./ecoblock-api-kernel"]

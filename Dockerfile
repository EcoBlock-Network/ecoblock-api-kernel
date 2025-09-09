FROM debian:bullseye-slim AS builder

RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
		ca-certificates \
		curl \
		build-essential \
		libssl-dev \
		pkg-config \
		libpq-dev \
		git \
	&& rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo PATH="/usr/local/cargo/bin:${PATH}"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /usr/src/app
COPY --from=builder /usr/src/app/target/release/ecoblock-api-kernel ./ecoblock-api-kernel
COPY prometheus.yml ./prometheus.yml
EXPOSE 3000
CMD ["./ecoblock-api-kernel"]

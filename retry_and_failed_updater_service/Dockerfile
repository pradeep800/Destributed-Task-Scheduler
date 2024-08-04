FROM rust:1.80 as builder 
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get -y install ca-certificates cmake musl-tools libssl-dev && rm -rf /var/lib/apt/lists/*
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV SQLX_OFFLINE=true
RUN cargo build --target x86_64-unknown-linux-musl --bin retry_and_failed_updater_service --release
FROM scratch
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/retry_and_failed_updater_service /app/retry_and_failed_updater_service
EXPOSE 3000
CMD ["/app/retry_and_failed_updater_service"]



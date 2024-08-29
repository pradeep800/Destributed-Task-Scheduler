FROM clux/muslrust:stable AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Notice that we are specifying the --target flag!
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE=true

ENV K8S_OPENAPI_ENABLED_VERSION=1.30
RUN cargo build --release --target x86_64-unknown-linux-musl --bin worker_main 

FROM alpine AS runtime
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/worker_main /usr/local/bin/
CMD ["/usr/local/bin/worker_main"]


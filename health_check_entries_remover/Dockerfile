FROM rust:latest as builder

WORKDIR /usr/src/app
COPY ./ .

RUN SQLX_OFFLINE=true cargo build --bin remove_health_check_entry --release

FROM debian:bullseye-slim

RUN apt-get update 
COPY --from=builder /usr/src/app/target/release/remove_health_check_entry /usr/local/bin/

CMD ["remove_health_check_entry"]

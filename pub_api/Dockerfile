FROM rust:latest as builder

WORKDIR /usr/src/app
COPY ./ .

RUN SQLX_OFFLINE=true cargo build --bin pub_api --release

FROM debian:bullseye-slim

RUN apt-get update 
COPY --from=builder /usr/src/app/target/release/pub_api /usr/local/bin/

CMD ["pub_api"]

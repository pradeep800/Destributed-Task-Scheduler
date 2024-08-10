We want statically link binary so we can do it by two types
# 1st type
install x86_64-unknown-linux-musl 

# 2nd type
With docker
## step 1
copy this docker file
```Dockerfile
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
RUN cargo build --release --target x86_64-unknown-linux-musl 

FROM alpine AS runtime
RUN addgroup -S myuser && adduser -S myuser -G myuser
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/<file-name> /usr/local/bin/
USER myuser
CMD ["/usr/local/bin/<file-name>"]
```

change file name with your file name 

## step 2

build the image

```bash
docker build . -t <image-name>:<tag> 
```

## step 3

run the container 

```bash
docker run <image-name>:<tag>
```

## step 4
copy file from docker to local
```bash
docker cp <containerId>:/usr/local/bin/<file-name> .
```

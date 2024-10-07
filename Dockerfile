FROM alpine:latest AS builder
RUN apk add --no-cache rust cargo pkgconfig openssl openssl-dev rustup
WORKDIR /build
COPY Cargo.lock .
COPY Cargo.toml .
COPY src/ src
#RUN rustup target add x86_64-unknown-linux-musl
#RUN cargo build --target=aarch64-unknown-linux-musl
RUN cargo build --package swappy2 --bin swappy2 --release 

FROM alpine:latest
RUN apk add --no-cache openssl openssl-dev libgcc
COPY --from=builder /build/target/release/swappy2 /usr/bin/
#VOLUME ["/sys/fs/cgroup"]
EXPOSE 8443
CMD ["swappy2"]

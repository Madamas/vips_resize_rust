FROM rust:1.53.0 as builder

WORKDIR /opt/thumbnailer_rust
COPY . /opt/thumbnailer_rust
RUN cargo install --path .

FROM debian:stable-slim

COPY --from=builder /usr/local/cargo/bin/thumbnailer_rust /app/thumbnailer_rust
COPY --from=builder /opt/thumbnailer_rust/Cargo.toml /app/Cargo.toml
WORKDIR /app

CMD [ "./thumbnailer_rust" ]
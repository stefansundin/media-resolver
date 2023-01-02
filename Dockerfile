FROM rust:1-bullseye AS builder

RUN rustup --version

WORKDIR /src

# Cache the dependencies to speed up future builds:
COPY Cargo.toml .
COPY Cargo.lock .
RUN mkdir -p .cargo
RUN cargo vendor > .cargo/config

COPY . .
RUN cargo build --release


FROM debian:bullseye-slim

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y ca-certificates

COPY --from=builder /src/target/release/media-resolver /

EXPOSE 8080
ENTRYPOINT [ "/media-resolver" ]

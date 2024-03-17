FROM rust:1-bookworm AS builder

RUN rustup --version

WORKDIR /src

# Cache the dependencies to speed up future builds:
COPY Cargo.toml .
COPY Cargo.lock .
RUN mkdir -p .cargo
RUN cargo vendor > .cargo/config

COPY . .
RUN cargo build --release


# Vector can ship the logs when the app is hosted on fly, configure in vector.yaml
FROM timberio/vector:0.36.X-debian AS vector


FROM debian:bookworm-slim

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y ca-certificates

COPY --from=builder /src/target/release/media-resolver /
COPY --from=builder /src/entrypoint.sh /entrypoint.sh
COPY --from=builder /src/vector.sh /vector.sh
COPY --from=builder /src/vector.yaml /etc/vector/vector.yaml
COPY --from=vector /usr/bin/vector /usr/bin/vector
RUN mkdir -p /var/lib/vector/

ENV HOST=0.0.0.0
ENV PORT=8080

EXPOSE 8080
ENTRYPOINT [ "/entrypoint.sh" ]

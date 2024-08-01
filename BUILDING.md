# Binaries

```shell
# Build for Linux:
rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu armv7-unknown-linux-gnueabihf
cargo build --release --target=x86_64-unknown-linux-gnu --target=aarch64-unknown-linux-gnu --target=armv7-unknown-linux-gnueabihf

# Build for macOS:
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --release --target=aarch64-apple-darwin --target=x86_64-apple-darwin
# Create universal binary:
lipo -create -output target/media-resolver-universal target/aarch64-apple-darwin/release/media-resolver target/x86_64-apple-darwin/release/media-resolver

# Build for Windows:
rustup target add x86_64-pc-windows-msvc
cargo build --release --target=x86_64-pc-windows-msvc
```

# Docker

You can build the docker image by running:

```shell
# Simple build for your current architecture:
docker build --pull --progress plain -t media-resolver .
```

To build a multi-arch docker image you can run:

```shell
# Use buildx to build multi-arch images:
docker buildx create --use --name multiarch --node multiarch0

# Beta release (may be limited to one architecture):
docker buildx build --pull --push --progress plain --platform linux/arm64 -t stefansundin/media-resolver:beta .

# Stable release:
docker buildx build --pull --push --progress plain --platform linux/arm64,linux/amd64,linux/arm/v7 -t stefansundin/media-resolver:v0.1.0 .

# If the new version is stable then update tags:
docker buildx imagetools create -t stefansundin/media-resolver:v0 stefansundin/media-resolver:v0.1.0
docker buildx imagetools create -t stefansundin/media-resolver:latest stefansundin/media-resolver:v0
```

If the build crashes then it is likely that Docker ran out of memory. Increase the amount of RAM allocated to Docker and quit other programs during the build. You can also try passing `--build-arg CARGO_BUILD_JOBS=1` to docker. To build one architecture at a time, [use `--config` when creating the builder instance](https://gist.github.com/stefansundin/fa1c1dd7a60ebe2f8a2aa6d32631b119).

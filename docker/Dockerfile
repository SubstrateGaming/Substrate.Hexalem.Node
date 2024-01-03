# Builder
FROM docker.io/library/rust:1.75-bullseye as builder

ARG features
ARG bin

COPY . /hexalem
WORKDIR /hexalem

RUN apt update && apt install -y git clang curl libssl-dev llvm libudev-dev cmake protobuf-compiler
RUN cargo build --locked --release --no-default-features --features ${features} --bin ${bin}

# Runner
FROM docker.io/library/ubuntu:22.04

ARG bin

COPY --from=builder /hexalem/target/release/${bin} /usr/local/bin/ajuna
COPY --from=builder /hexalem/resources/ /

RUN useradd -m -u 1000 -U -s /bin/sh -d /hexalem hexalem && \
  mkdir -p /data /hexalem/.local/share && \
  chown -R hexalem:hexalem /data  && \
  ln -s /data /hexalem/.local/share/hexalem && \
  rm -rf /usr/sbin && \
  ajuna --version

USER hexalem

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]
ENTRYPOINT ["/usr/local/bin/hexalem"]

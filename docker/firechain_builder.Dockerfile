FROM rust:1.61.0 AS builder

WORKDIR /5ire

COPY . /5ire

ARG environment

ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

RUN apt-get update && apt-get install -y protobuf-compiler libclang-dev

RUN cargo build --release --features firechain-${environment}

FROM debian:stable-slim

ARG environment

WORKDIR /5ire

COPY --from=builder /5ire/target/release/firechain-node /5ire/firechain-node

COPY --from=builder /5ire/specs/5ire-${environment}-specRaw.json /5ire/specs/5ire-${environment}-specRaw.json

RUN ldd /5ire/firechain-node
RUN /5ire/firechain-node --version

EXPOSE 30333 9944

VOLUME ["5ire/data"]

ENTRYPOINT ["/5ire/firechain-node"]
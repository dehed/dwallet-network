FROM rust:1.81-bullseye as build

ARG PROFILE=release
WORKDIR /work

RUN apt-get update && apt-get install -y cmake clang

COPY .git/ .git/
COPY Cargo.toml Cargo.lock ./
COPY consensus consensus
COPY crates crates
COPY ika-execution ika-execution
COPY narwhal narwhal
COPY external-crates external-crates

RUN cargo build --profile ${PROFILE} --bin ika-proxy

FROM gcr.io/distroless/cc-debian12 as deploy

COPY --from=build --chmod=755 /work/target/release/ika-proxy /opt/ika/bin/ika-proxy

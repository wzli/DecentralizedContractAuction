FROM ubuntu:18.04

RUN apt update && apt install -y \
  vim \
  screen \
  cmake \
  pkg-config \
  libssl-dev \
  git \
  build-essential \
  clang \
  libclang-dev \
  curl \
  && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

RUN . ~/.cargo/env \
  && rustup install nightly \
  && rustup target add wasm32-unknown-unknown --toolchain nightly \
  && rustup component add rust-src --toolchain nightly \
  && rustup target add wasm32-unknown-unknown --toolchain stable \
  && cargo +nightly install canvas-node --git https://github.com/paritytech/canvas-node.git --force \
  && cargo install cargo-contract --force

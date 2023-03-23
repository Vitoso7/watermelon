FROM rustlang/rust:nightly
RUN apt-get update && apt-get install -y \
  mingw-w64 \
  && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-pc-windows-gnu

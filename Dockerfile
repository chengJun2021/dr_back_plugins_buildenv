FROM rust:buster as builder
WORKDIR /app
RUN mkdir src; echo 'fn main() {}' > src/main.rs
COPY Cargo.* ./
RUN cargo build --release

COPY . .
RUN cargo build --release

FROM node:lts-buster
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
WORKDIR /overlay
COPY overlay /overlay/
RUN npm install

WORKDIR /env
COPY --from=builder /app/target/release/dr_plugins_build_env /usr/local/bin/dr_plugins_build_env
CMD ["dr_plugins_build_env"]

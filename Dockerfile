FROM rust:buster as builder
WORKDIR /app
RUN mkdir src; echo 'fn main() {}' > src/main.rs
COPY Cargo.* ./
RUN cargo build --release

COPY . .
RUN cargo build --release

FROM node:lts-buster
RUN apt-get update && apt-get install -y sudo && rm -rf /var/lib/apt/lists/*
WORKDIR /env
COPY overlay /env/
RUN npm install

COPY --from=builder /app/target/release/dr_plugins_build_env /usr/local/bin/dr_plugins_build_env

# -r-x------ for the executable, prevents exploits that attempts to bundle
# system resources by require() or import()
#
# There will be static analysis in addition to this
RUN chmod 500 /usr/local/bin/dr_plugins_build_env
CMD ["dr_plugins_build_env"]

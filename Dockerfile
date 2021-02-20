FROM rust:buster as builder
RUN rustup default nightly

WORKDIR /app
RUN mkdir src; echo 'fn main() {}' > src/main.rs
COPY Cargo.* ./

COPY plugins_commons/Cargo.* plugins_commons/
RUN mkdir plugins_commons/src; touch plugins_commons/src/lib.rs

RUN cargo build --release

COPY . .
COPY plugins_commons/ plugins_commons/
RUN cargo build --release


FROM node:lts-buster
WORKDIR /honeypot
COPY honeypot /honeypot/
RUN chmod 400 /honeypot/*privileged*

WORKDIR /env
RUN apt-get update && apt-get install -y sudo && rm -rf /var/lib/apt/lists/*
COPY overlay /env/
RUN npm install

COPY --from=builder /app/target/release/dr_plugins_build_env /usr/local/bin/dr_plugins_build_env

# -r-x------ for the executable, prevents exploits that attempts to bundle
# system resources by require() or import()
#
# There will be static analysis in addition to this
RUN chmod 500 /usr/local/bin/dr_plugins_build_env
CMD ["dr_plugins_build_env"]

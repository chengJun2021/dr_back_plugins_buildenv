FROM rust:buster as builder
RUN rustup default nightly

WORKDIR /app
COPY . .
RUN cargo build --release


FROM node:lts-buster
WORKDIR /honeypot
COPY honeypot /honeypot/
RUN chmod 400 /honeypot/*privileged*

WORKDIR /env
RUN apt-get update && apt-get install -y sudo && rm -rf /var/lib/apt/lists/*
COPY overlay /env/
RUN npm install
RUN chmod -R +x .
RUN groupadd builder; useradd bob; usermod -aG builder bob

COPY --from=builder /app/target/release/buildenv /usr/local/bin/buildenv

# -r-x------ for the executable, prevents exploits that attempts to bundle
# system resources by require() or import()
#
# There will be static analysis in addition to this
RUN chmod 500 /usr/local/bin/buildenv
CMD ["buildenv"]

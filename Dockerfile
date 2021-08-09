FROM rust:buster as rustcache
RUN rustup default nightly

WORKDIR /app
COPY Cargo.* ./
COPY plugins_commons/Cargo.* plugins_commons/
RUN mkdir src plugins_commons/src; echo "fn main() { panic!(\"Cached executable is being used\") }" > src/main.rs; touch plugins_commons/src/lib.rs
RUN cargo build --release
RUN rm -r src plugins_commons target/release/buildenv*
RUN find $PWD -name "*plugins_commons*" -depth -print0 | xargs -0 rm -r

FROM rust:buster as rustbuilder
RUN rustup default nightly

WORKDIR /app
COPY --from=rustcache /app/ ./
COPY . .

RUN cargo build --release

FROM node:lts-buster as nodenv
WORKDIR /env
COPY overlay /env/
RUN npm install
RUN shred .npmrc; rm .npmrc

FROM node:lts-buster
WORKDIR /env
RUN apt-get update && apt-get install -y sudo && rm -rf /var/lib/apt/lists/*
COPY --from=nodenv /env/ /env/
RUN chmod -R 555 .
RUN groupadd builder; useradd bob; usermod -aG builder bob

COPY --from=rustbuilder /app/target/release/buildenv /usr/local/bin/buildenv

# ---x------ for the executable, prevents exploits that attempts to bundle
# system resources by require() or import()
#
# There will be static analysis in addition to this
RUN chmod 100 /usr/local/bin/buildenv
CMD ["buildenv"]

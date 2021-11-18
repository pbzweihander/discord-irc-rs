FROM clux/muslrust:1.56.1-stable

WORKDIR /
RUN USER=root cargo new --bin app
WORKDIR /app

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release &&\
    rm -r src

COPY ./src ./src

RUN cargo build --release

FROM alpine

COPY --from=0 /app/target/x86_64-unknown-linux-musl/release/discord-irc /usr/local/bin/

RUN apk add --no-cache tini
ENTRYPOINT ["/sbin/tini", "--"]

CMD ["discord-irc", "/config.toml"]

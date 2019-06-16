FROM clux/muslrust:nightly-2019-05-22

WORKDIR /
RUN USER=root cargo new --bin discord-irc
WORKDIR /discord-irc

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm -r src
RUN rm ./target/x86_64-unknown-linux-musl/release/deps/discord_irc*

COPY ./src ./src

RUN cargo build --release

FROM alpine

COPY --from=0 /discord-irc/target/x86_64-unknown-linux-musl/release/discord-irc /

RUN apk add --no-cache tini
ENTRYPOINT ["/sbin/tini", "--"]

CMD ["/discord-irc", "/config.toml"]

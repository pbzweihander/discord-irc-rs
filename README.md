# discord-irc-rs

[![CircleCI Status Badge]][CircleCI]
[![Docker Pulls Badge]][Docker Hub]

A Discord to IRC and vice-versa bridge bot written in Rust.

## Usage

Requires nightly-2019-05-22 version of rustc.

```bash
$ cp sample.toml config.toml
# Edit config.toml ...
$ cargo run -- config.toml
```

OR

```bash
$ cp sample.toml config.toml
# Edit config.toml ...
$ docker run --rm -it -v $PWD/config.toml:/config.toml pbzweihander/discord-irc-rs
```

------

_discord-irs-rs_ is distributed under the terms of both [MIT license] and [Apache License 2.0]. See [COPYRIGHT] for details.

[CircleCI Status Badge]: https://circleci.com/gh/pbzweihander/discord-irc-rs.svg?style=svg
[CircleCI]: https://circleci.com/gh/pbzweihander/discord-irc-rs
[Docker Pulls Badge]: https://badgen.net/docker/pulls/pbzweihander/discord-irc-rs
[Docker Hub]: https://hub.docker.com/r/pbzweihander/discord-irc-rs

[MIT license]: LICENSE-MIT
[Apache License 2.0]: LICENSE-APACHE
[COPYRIGHT]: COPYRIGHT

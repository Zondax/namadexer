##### BUILDER #####
FROM rust:slim-bookworm as builder

WORKDIR /usr/src/namadexer
COPY . .

RUN apt-get update && apt-get install -y protobuf-compiler build-essential wget

RUN make download-checksum

RUN cargo install --path . -F prometheus

##### RUNNER #####
FROM debian:12-slim

LABEL author="Lola Rigaut-Luczak <lola@zondax.ch>"

WORKDIR /app

COPY --from=builder /usr/local/cargo/bin/server /usr/local/bin/server
COPY --from=builder /usr/local/cargo/bin/indexer /usr/local/bin/indexer
COPY --from=builder /usr/src/namadexer/checksums.json /app

RUN apt-get update && rm -rf /var/lib/apt/lists/*

# if you wish to run with a config file, mount it at /app/config/Settings.toml
# and set the env variable INDEXER_CONFIG_PATH to /app/config/Settings.toml

CMD indexer

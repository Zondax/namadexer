##### BUILDER #####
FROM rustlang/rust:nightly as builder

WORKDIR /usr/src/namadexer
COPY . .
# We need a specific protoc version
RUN wget https://github.com/protocolbuffers/protobuf/releases/download/v3.16.3/protoc-3.16.3-linux-x86_64.zip
RUN unzip protoc-3.16.3-linux-x86_64.zip -d ./protoc
RUN chmod -R 777 ./protoc

ENV PROTOC "/usr/src/namadexer/protoc/bin/protoc"

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

# default env
ENV INDEXER_CONFIG_PATH "/app/config/Settings.toml"
ENV RUST_LOG "namadexer=debug"

CMD indexer

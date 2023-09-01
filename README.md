# Namadexer

Namadexer is an indexer for [Namada](https://github.com/anoma/namada).

## Overview

The project is composed of 2 entities : the `indexer` and the `server`. They are both written in Rust.

![Namadexer graph](./docs/namadexer.jpg)

- the `indexer`: it connects to the Namada node throught rpc and collect the blocks and transactions. Then it stores them in the postgres database. The indexer doesn't know about the server and can be started independently.

- the `server`: it is a JSON server that allows querying block and transaction using unique identifier. Other useful endpoints like `/blocks/latest` can be found too. A list of all the endpoints and their description can be find in the documenttaion.

Those services requires a connection to a [postgres](https://www.postgresql.org/) database. Support for [OpenTelemetry](https://opentelemetry.io/) was also added.

## Documentation

You can find more information about the indexer in the [`./docs`](./docs/) folder.

## Dev

You will need access to a namada node and inform its tendermint rpc host and port in the `Settings.toml` file.

### Dev dependencies

You will need rust installed and a running node of namada accessible locally.

It will install teh right version of protoc (at least 3.12) in this repo to avoid conflict with other installed version
```
$ make install-deps
```

### Start developping

Start the docker database :
```
$ make postgres
```

You will need to use this command if you want to avoid issues with protoc.
```
$ make run
```

## Telemetry

Run jaeger in background
```
$ docker run -d -p6831:6831/udp -p6832:6832/udp -p16686:16686 -p14268:14268 jaegertracing/all-in-one:latest
```
Start the indexer
```
$ RUST_LOG=trace cargo run --bin indexer
```

View spans
```
$ firefox http://localhost:16686/
```
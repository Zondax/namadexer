# Namadexer

Namadexer is an indexer for [Namada](https://github.com/anoma/namada).

## Overview

The project is composed of 2 entities : the `indexer` and the `server`. They are both written in Rust.

![Namadexer graph](./docs/assets/namadexer.jpg)

- the `indexer`: This component establishes a connection to the Namada node via RPC and collects blocks and transactions. It then stores this data in a PostgreSQL database. The indexer operates independently of the server and can be initiated on its own.

- the `server`: This is a JSON-based server that facilitates querying of blocks and transactions using unique identifiers. It also provides additional useful endpoints, such as  `/blocks/latest`.  A comprehensive list of all endpoints, along with their descriptions, is available in the documentation.

These services require a connection to a [postgres](https://www.postgresql.org/) database. Support for [OpenTelemetry](https://opentelemetry.io/) has also been added.


## Documentation

You can find more information about the indexer in the [`./docs`](./docs/) folder.

## Development

You will need access to a namada node and specify its tendermint rpc host and port in the `Settings.toml` file. You can use the `Settings.example.toml` as a template.

### Dev dependencies

To proceed, you must have Rust installed and a Namada node operational and accessible locally.

The system will automatically install the appropriate version of protoc (version 3.12 or higher) within this repository. This ensures no conflicts arise with other versions of protoc that may be installed on your system

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

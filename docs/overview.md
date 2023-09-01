# Overview

The project is composed of 2 entities : the `indexer` and the `server`. They are both written in Rust.

![Namadexer graph](./namadexer.jpg)

- the `indexer`: it connects to the Namada node throught rpc and collect the blocks and transactions. Then it stores them in the postgres database. The indexer doesn't know about the server and can be started independently.

- the `server`: it is a JSON server that allows querying block and transaction using unique identifier. Other useful endpoints like `/blocks/latest` can be found too. A list of all the endpoints and their description can be find in the documenttaion.

Those services requires a connection to a [postgres](https://www.postgresql.org/) database. Support for [OpenTelemetry](https://opentelemetry.io/) was also added.
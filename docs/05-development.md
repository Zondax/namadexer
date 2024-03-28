# Development


The full project can be run using the example `docker-compose.yml` file found under `./contrib`.

```yml
version: '3'
services:
  postgres:
    image: postgres:14
    container_name: postgres
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: wow
      POSTGRES_DB: blockchain 
    ports:
      - "5433:5432"

  indexer:
    build: ../.
    container_name: indexer
    environment:
      - RUST_LOG="namadexer=debug"
      - INDEXER_CONFIG_PATH=/app/config/Settings.toml
    volumes:
      - ../config:/app/config
      - ${PWD}/checksums.json:/app/checksums.json
    depends_on:
      - postgres
    command: ["/bin/bash", "-c", " /usr/local/bin/indexer"]

  server:
    build: ../.
    container_name: server
    environment:
      - RUST_LOG="namadexer=debug"
      - INDEXER_CONFIG_PATH=/app/config/Settings.toml
    volumes:
       - ../config:/app/config
    ports:
      - "30303:30303"
    depends_on:
      - postgres
      - indexer
    command: ["/bin/bash", "-c", "/usr/local/bin/server"]
```

The `Settings.toml` contains the required configuration data to connect to the Namada node and to the database. We also need the `checksums.json` file from the Namada node but only for the indexer. It maps the hash code to the transaction type and is needed for deseriliazing transactions in the indexer.

Launch the containers:
```
$ docker compose -f contrib/docker-compose.yaml up --build
```

## Monitoring

In addition, prometheus and grafana can be used to collect logs if the feature is activated. The docker compose file can be found under `./contrib/prometheus-compose.yml`.

More info in the [telemetry](./telemetry.md) section.


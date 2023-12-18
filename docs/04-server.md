# Server

The JSON server is written in Rust. It is using `axum` to create endpoints. It requires an access to the PostgreSQL database where the blocks and transactions have been stored.

## Configuration

Settings.toml
```
log_level = "info"

# Connection information for the PostgreSQL database
[database]
host = "localhost"
user = "postgres"
password = "wow"
dbname = "blockchain"


[server]
serve_at = "0.0.0.0"
port = 30303
```

## Endpoints

The list of endpoints available.

## /block/height/:block_height

This endpoint look for a specific block by its `height`.

Example:
```
$ curl -H 'Content-Type: application/json' localhost:30303/block/height/1
```

## /block/hash/:block_hash

This endpoint look for a specific block by its `hash`.
Example:
```
$ curl -H 'Content-Type: application/json' localhost:30303/block/hash/9d6dad4409536ab763c0b814379be71ad1f9176efe17292f143831fbad72109c
```

## /block/last

This endpoint will return the last block indexed.
Example:
```
$ curl -H 'Content-Type: application/json' localhost:30303/block/last
```

## /tx/:tx_hash

This endpoint will look for a specific transaction identified by tx_hash.
Example:
```
$ curl -H 'Content-Type: application/json' localhost:30303/tx/c602b2f3b88811bfd7f3fdf866af3b1487bfd21c5b5ea7f7f9a16fb6bb915c24
```

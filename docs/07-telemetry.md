## Indexer Application Telemetry Documentation

### Overview

This documentation provides details on the telemetry configuration for the indexer application using Prometheus, Jaeger and Grafana. It will guide you through the setup, the metrics that are being recorded, and how to launch/use those services.

## Jaeger Tracing for Indexer Application Documentation

Jaeger is a powerful tool used for monitoring and troubleshooting microservices-based distributed systems. With capabilities like distributed context propagation and transaction monitoring, Jaeger traces requests as they traverse through various services, helping developers identify bottlenecks and performance issues.

For the indexer application, Jaeger has been configured to trace the following operations:
- `save_block`: Tracks the process of saving a block.
- `save_evidences`: Monitors the time and operations taken to save block evidences.
- `save_transactions`: Observes the transaction saving process.
- `get_block`: Tracks the time and steps required to retrieve a block.

This section provides details on the Jaeger configuration, its deployment using Docker, and the process of accessing its user interface.

### Jaeger Configuration

Here's the indexer's Jaeger configuration:

```toml
[jaeger]
enable = true
host = "localhost"
port = 6831
```

#### Configuration Breakdown:

- **enable**: Determines if Jaeger tracing is activated for the indexer application.
- **host**: The hostname where the Jaeger server listens. Set to `localhost` by default.
- **port**: The port at which Jaeger collects trace data. By default, it uses port `6831`, a standard for Jaeger.

### Deploying Jaeger with Docker

To deploy a Jaeger service that will collect tracing data from the indexer application and listen on `localhost:6831`, use the Docker command provided:

```bash
# Run Jaeger in the background
docker run -d -p6831:6831/udp -p6832:6832/udp -p16686:16686 -p14268:14268 jaegertracing/all-in-one:latest
```

This command employs Jaeger's all-in-one Docker image, encompassing the UI, collector, query, and agent components.

### Running the Indexer

To kickstart the indexer application with tracing enabled:

```bash
RUST_LOG=trace cargo run --bin indexer
```

### Accessing the Jaeger UI

To delve into the traces gathered by Jaeger:

1. Launch Firefox or your browser of choice.
2. Head over to [http://localhost:16686/](http://localhost:16686/).

The Jaeger UI presents an in-depth analysis of trace data, offering you a vantage point to navigate individual traces, view request timelines, and much more.

Integrating Jaeger's distributed tracing with the indexer application furnishes granular insights into service operations, serving as a diagnostic tool for performance issues, latency roadblocks, and other potential challenges. For advanced configurations or a deeper dive into Jaeger's myriad features, consider referring to Jaeger's official documentation or seek guidance from tracing aficionados.

### Configuration Details for Prometheus

The following is the Prometheus configuration for the indexer:

```toml
[prometheus]
host = "0.0.0.0"
port = 9000
```

This configuration specifies the address for the HTTP server that the application launches. It allows any Prometheus service collector to make requests for metrics collection.

### Metrics Collection

The metrics collected by the service include:

- **get_block**: Measures the time taken to retrieve a block from the network using the Namada RPC client.
- **indexer_get_block_duration**: Measures the time required to save a block into the database.
- **db_save_transactions_duration**: Similar to the block save metric, this metric captures the time spent to save a transaction.
- **db_save_evidences_duration**: Measures the duration to store block evidences into the database.
- **db_save_block_count**: Tracks the total number of blocks saved to the database since the indexer application initiation.

### Enabling Prometheus Server

The Prometheus server for the indexer application is controlled via a Rust feature flag:

```toml
[features]
default = []
prometheus = ["metrics-exporter-prometheus", "axum-prometheus"]
```
This feature flags also enables prometheus metrics for the json-server.

By default, the Prometheus feature is disabled. To enable Prometheus and run the indexer service, use the following command:

```bash
RUST_LOG=info cargo run --bin indexer --features prometheus
```
This will deploy a server that the prometheus service can make request to for metrics. Bellow how to configure the prometheus service that this application provides and can easily be run  using docker compose.

## Prometheus Scrape Configuration

Prometheus operates by "scraping" or polling exposed endpoints (or targets) at regular intervals to collect metrics. 
The configuration that dictates which endpoints to scrape, how often to scrape them, and other scraping-related parameters is 
defined under `scrape_configs` in the `monitoring/prometheus/prometheus.yml` configuration file. 

### Scrape Configuration Breakdown

Below is the detailed configuration for the Prometheus scrape setup:

```yaml
scrape_configs:
  - job_name: namada_indexer_metrics
    scrape_interval: 5s
    static_configs:
      - targets: ['host.docker.internal:9000']

  - job_name: cadvisor
    scrape_interval: 5s
    static_configs:
      - targets: ['cadvisor:8080']
```

#### Configuration Breakdown:

- **job_name**: It's a user-defined string that designates a particular scrape configuration. In the above setup, there are two jobs: `namada_indexer_metrics` for the indexer application metrics and `cadvisor` for container monitoring.

- **scrape_interval**: Specifies how often Prometheus should scrape the target endpoints. Both jobs are set to be scraped every 5 seconds.

- **static_configs** & **targets**: 
  - For `namada_indexer_metrics`, the target is set to `host.docker.internal:9000`. This configuration works seamlessly on Windows and macOS. For Linux environments, ensure that the `network_mode` is set to `host` in the Docker Compose configuration for Prometheus.
  - For `cadvisor`, the target is `cadvisor:8080`, which is a typical setting when using cAdvisor for container monitoring.

### Important Note on Target Configuration

The `targets` field in the scrape configuration *must align* with where your service exposes its metrics. In this context, the indexer application is the service that deploys a server exposing metrics. Thus, for `namada_indexer_metrics`, the target `host.docker.internal:9000` should match the port (`9000`) and address where the indexer service exposes its Prometheus metrics.


For advanced configurations, troubleshooting, or further reading, it's recommended to refer to the official Prometheus documentation or consult with monitoring experts.


### Prometheus Metrics Collector Deployment

A Docker Compose file is available to facilitate the deployment of a container, which runs a Prometheus metrics collector. This collector is pre-configured to fetch data from the server where the indexer service is active. Moreover, this container configuration supports modification of the data collection address.

Here's the Docker Compose configuration:

```yaml
version: '3.8'
services:
  # Services for monitoring
  # Comment out the services below if monitoring is not required
  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    restart: always
    ports:
      - '9090:9090'
    network_mode: "host"
    volumes:
      - ../monitoring/prometheus:/etc/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--web.external-url=http://localhost:9090'

  grafana:
    image: grafana/grafana:latest
    container_name: grafana
    user: "472:472"
    restart: always
    ports:
      - '3000:3000'

    network_mode: "host"
    volumes:
      - ../monitoring/grafana/data:/var/lib/grafana
      - ../monitoring/grafana/provisioning:/etc/grafana/provisioning
    environment:
      GF_SECURITY_ADMIN_USER: admin
      GF_SECURITY_ADMIN_PASSWORD: admin
```

To launch the Prometheus collector service on a Linux environment, ensure that the indexer has been compiled with Prometheus enabled and is currently running. Then, execute:

```bash
docker compose -f contrib/prometheus-compose.yaml up -d

```
## Grafana Telemetry Configuration


Grafana provides a powerful dashboarding platform that complements Prometheus by visualizing the metrics collected. This section details how Grafana is set up to communicate with the Prometheus service collector associated with the indexer application.

### Grafana Container Configuration

The Docker Compose configuration for Grafana is structured as follows:

```yaml
grafana:
  image: grafana/grafana:latest
  container_name: grafana
  user: "472:472"
  restart: always
  ports:
    - '3000:3000'
  network_mode: "host"
  volumes:
    - ../monitoring/grafana/data:/var/lib/grafana
    - ../monitoring/grafana/provisioning:/etc/grafana/provisioning
  environment:
    GF_SECURITY_ADMIN_USER: admin
    GF_SECURITY_ADMIN_PASSWORD: admin
```

#### Configuration Breakdown:

- **Image**: The Grafana container is built from the official latest Grafana image.
  
- **Ports**: By default, Grafana is accessible on port 3000.
  
- **Volumes**: Two volumes are defined:
  - **Data volume**: Stores Grafana data like dashboard settings, panels, etc.
  - **Provisioning volume**: Contains Grafana's provisioning configurations.

- **Environment Variables**: Default credentials for the Grafana admin user are set. For production environments, it's advisable to change these for enhanced security.

### Interaction with Prometheus

While the Grafana container configuration itself doesn't relate directly to the indexer application, it is intended to work with the Prometheus metrics collector service. Specifically, within Grafana, one can set up a data source pointing to the Prometheus service to fetch and visualize the metrics.

### Starting the Grafana Service

Both Grafana and Prometheus services can be started using the previously provided Docker Compose command:

```bash
docker compose -f contrib/prometheus-compose.yaml up -d
```

Upon launching, Grafana's UI should be accessible via a web browser at `http://localhost:3000`. Using the default `admin` credentials, users can log in, set up data sources, and create dashboards to visualize the metrics captured by Prometheus.

To maximize the effectiveness of telemetry, ensure that Grafana's data source configuration is correctly pointing to the Prometheus service. Remember to secure Grafana, especially in production environments, by setting strong credentials and employing other security practices.

For further assistance or queries, please consult the main documentation or contact the support team.

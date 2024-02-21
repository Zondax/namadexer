use tracing::Subscriber;
use tracing_subscriber::Layer;

use tracing_subscriber::{filter::filter_fn, layer::SubscriberExt, EnvFilter, Registry};

use crate::{JaegerConfig, LogFormat, Settings};
use opentelemetry_api::global;

/// Setup looging this includes a fmt::layer and jaeger(if enable)
/// # panics:
/// This methods panics if there is already a global tracing::subscriber
/// configured.
pub fn setup_logging(cfg: &Settings) {
    let sub = get_subscriber(cfg);
    init_subscriber(sub);
}

/// Creates a subscriber that collect traces and send them to either Jaeger(if enable)
/// and std::out.
/// Users can call this function instead of (setup_logging)[crate::setup_logging] if they
/// want more controll over how to configure they tracing layers and subscribers.
pub fn get_subscriber(cfg: &Settings) -> impl Subscriber + Send + Sync {
    // Get tracing filter either log, debug, trace, error,warning
    // otherwise use the log_level setting from configuration file.
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&cfg.log_level));

    // send logs to stdout in the configured format
    let (json, pretty) = match cfg.log_format {
        LogFormat::Json => (Some(tracing_subscriber::fmt::layer().json()), None),
        LogFormat::Pretty => (None, Some(tracing_subscriber::fmt::layer().pretty())),
    };

    // check if jaeger telemetry is enable and configure it accordingly
    let jaeger_cfg = cfg.jaeger_config();
    let mut jaeger = None;
    if jaeger_cfg.enable {
        jaeger = Some(jaeger_layer(jaeger_cfg));
    }

    Registry::default()
        .with(env_filter)
        .with(json)
        .with(pretty)
        .with(jaeger)
}

/// Sets the passed in `subscriber` as the global one.
/// # panics:
/// If there is a global subscriber already configured
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    tracing::subscriber::set_global_default(subscriber).expect("Could not set global logger");
}

// # Run jaeger in background
// $ docker run -d -p6831:6831/udp -p6832:6832/udp -p16686:16686 -p14268:14268 jaegertracing/all-in-one:latest
//
// # Start the indexer
// $ RUST_LOG=trace cargo run --bin indexer
//
// # View spans
// $ firefox http://localhost:16686/

/// Creates a tracing layer that sends traces to configured Jaeger service.
pub fn jaeger_layer<S>(cfg: &JaegerConfig) -> impl Layer<S> + Send + Sync
where
    S: tracing::Subscriber
        + Sync
        + Send
        + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    let endpoint = format!("{}:{}", cfg.host, cfg.port);

    // setup Jaeger which is a distributed tracer collector, prometheus will focus on metrics
    // that would be collected differently.
    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());

    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_endpoint(endpoint)
        .with_service_name("indexer-traces")
        .install_simple()
        .expect("Could not create Jaeger pipeline");

    let jaeger_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Only send database traces to Jaeger, the ones we care about like:
    // - save_blocks
    // - save evidences
    // - save transactions
    // - get_block
    jaeger_layer.with_filter(filter_fn(|metadata| {
        let target = metadata.name();
        target.contains("save_block")
            || target.contains("save_evidences")
            || target.contains("save_transactions")
            || target.contains("get_block")
    }))
}

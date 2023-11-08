use opentelemetry::global;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::runtime::TokioCurrentThread;
use std::{
    fs::{self, File},
    io::{self, stdout},
    sync::Arc,
};
use tracing_bunyan_formatter::JsonStorageLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Layer, Registry};

pub fn init_telemetry() -> io::Result<()> {
    LogTracer::init().expect("Failed to set logger");
    let app_name = env!("CARGO_PKG_NAME");

    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name(app_name)
        .install_batch(TokioCurrentThread)
        .expect("Failed to install OpenTelemetry tracer.");

    fs::create_dir_all("./logs").expect("Could not create directory"); // TODO: make log file a configuration options
    let stdout_log = tracing_subscriber::fmt::layer()
        .pretty()
        .with_writer(stdout); // TODO: make stdout log a configuration option
    let file = File::create("logs/debug.log")?;

    let debug_log = tracing_subscriber::fmt::layer().with_writer(Arc::new(file));
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(stdout_log.and_then(debug_log))
        .with(telemetry)
        .with(JsonStorageLayer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to install `tracing` subscriber.");

    Ok(())
}

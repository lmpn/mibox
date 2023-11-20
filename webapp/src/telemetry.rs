use tokio::task::JoinHandle;
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

pub fn get_subscriber<Sink>(name: &str, level: &str, sink: Sink) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // We are falling back to printing all spans at info-level or above // if the RUST_LOG environment variable has not been set.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    let formatting_layer = BunyanFormattingLayer::new(
        name.into(), // Output the formatted spans to sink.
        sink,
    );
    // The `with` method is provided by `SubscriberExt`, an extension // trait for `Subscriber` exposed by `tracing_subscriber`
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Redirect all `log`'s events to our subscriber
    LogTracer::init().expect("Failed to set logger");
    // `set_global_default` can be used by applications to specify
    // what subscriber should be used to process spans.
    set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}

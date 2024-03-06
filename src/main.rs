use rust_template_server::{configuration, startup::Server, telemetry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = configuration::get_configuration().expect("failed to read configuration");
    let subscriber = telemetry::get_subscriber(
        &settings.application.app_name,
        &settings.application.log_level,
        std::io::stdout,
    );
    telemetry::init_subscriber(subscriber);

    Server::with_settings(settings)
        .await
        .expect("error creating server")
        .serve()
        .await
}

use once_cell::sync::Lazy;
use rand::Rng;
use reqwest::Client;
use webapp::configuration::get_configuration;
use webapp::server::Server;
use webapp::telemetry;

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = telemetry::get_subscriber("test_app", "info", std::io::stdout);
        telemetry::init_subscriber(subscriber);
    } else {
        let subscriber = telemetry::get_subscriber("test_app", "info", std::io::sink);
        telemetry::init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub port: u16,
    pub address: String,
    pub client: Client,
}

impl TestApp {}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mut configuration = get_configuration().expect("could not read configuration");
    configuration.application.port = rand::thread_rng().gen();
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();
    let server = Server::with_settings(configuration.clone())
        .await
        .expect("error configuring server");
    let app = TestApp {
        port: server.address().port(),
        address: format!("http://localhost:{}", server.address().port()),
        client,
    };
    tokio::spawn(async move { server.serve().await.unwrap() });
    app
}

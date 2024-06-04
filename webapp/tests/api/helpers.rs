use std::io::Write;
use std::path::PathBuf;

use once_cell::sync::Lazy;
use rand::Rng;
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
    pub client: HttpClient,
    pub drive_base: String,
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
    let address = format!("http://localhost:{}", server.address().port());
    let app = TestApp {
        port: server.address().port(),
        address,
        client: HttpClient { inner: client },
        drive_base: configuration.application.drive,
    };
    tokio::spawn(async move { server.serve().await.unwrap() });
    app
}

pub struct HttpClient {
    inner: reqwest::Client,
}

impl HttpClient {
    pub async fn upload_files(
        &self,
        address: &str,
        files: Vec<(&str, &str)>,
    ) -> anyhow::Result<reqwest::Response> {
        if files.is_empty() {
            return Ok(self.inner.post(address).send().await?);
        }

        //create the multipart form
        let mut form = reqwest::multipart::Form::new();
        for (file, name) in files.into_iter() {
            let file = tokio::fs::File::open(file)
                .await
                .expect("error reading file");
            // read file body stream
            let stream =
                tokio_util::codec::FramedRead::new(file, tokio_util::codec::BytesCodec::new());
            let file_body = reqwest::Body::wrap_stream(stream);

            //make form part of file
            let some_file = reqwest::multipart::Part::stream(file_body)
                .file_name(name.to_owned())
                .mime_str("text/plain")?;
            form = form.part("file", some_file);
        }

        //send request
        let response = self
            .inner
            .post(address)
            .multipart(form)
            .send()
            .await
            .expect("error uploading files");

        Ok(response)
    }

    pub async fn download_file(&self, address: &str) -> anyhow::Result<reqwest::Response> {
        Ok(self
            .inner
            .get(address)
            .send()
            .await
            .expect("failed to download file"))
    }

    pub async fn list(&self, address: &str) -> anyhow::Result<reqwest::Response> {
        Ok(self
            .inner
            .get(address)
            .send()
            .await
            .expect("failed to delete file"))
    }

    pub async fn update_dir(&self, address: &str) -> anyhow::Result<reqwest::Response> {
        Ok(self
            .inner
            .put(address)
            .send()
            .await
            .expect("failed to delete file"))
    }

    pub async fn delete_dir(&self, address: &str) -> anyhow::Result<reqwest::Response> {
        Ok(self
            .inner
            .delete(address)
            .send()
            .await
            .expect("failed to delete file"))
    }
    pub async fn create_dir(&self, address: &str) -> anyhow::Result<reqwest::Response> {
        Ok(self
            .inner
            .post(address)
            .send()
            .await
            .expect("failed to delete file"))
    }

    pub async fn delete_file(&self, address: &str) -> anyhow::Result<reqwest::Response> {
        Ok(self
            .inner
            .delete(address)
            .send()
            .await
            .expect("failed to delete file"))
    }
}

pub fn random_name(len: usize) -> String {
    let chars: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"; // characters to choose from

    let mut rng = rand::thread_rng();
    let alphabet_size = chars.len();
    let get_random_char = |_| {
        let index = rng.gen_range(0..alphabet_size);
        chars[index] as char
    };
    (0..len).map(get_random_char).collect::<String>()
}

pub fn random_file(base_path: String) -> PathBuf {
    let name = random_name(10);
    let path = PathBuf::from(base_path).join(name);
    let mut file = std::fs::File::create(path.clone()).unwrap();
    file.write(b"RANDOM CONTENT").unwrap();
    return path;
}

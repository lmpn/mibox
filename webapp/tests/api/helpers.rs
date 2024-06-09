use once_cell::sync::Lazy;
use rand::Rng;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use webapp::configuration::get_configuration;
use webapp::handlers::directory::DriveView;
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
    pub address: String,
    pub client: HttpClient,
}

impl TestApp {}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mut configuration = get_configuration().expect("could not read configuration");
    configuration.application.port = rand::thread_rng().gen_range(1024..u16::MAX);
    let p = rand::thread_rng().gen_range(0..500) + 100;
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
        address,
        client: HttpClient { inner: client },
    };
    tokio::spawn(async move { server.serve().await.unwrap() });
    tokio::time::sleep(Duration::from_millis(p)).await;
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

    pub async fn list(&self, address: &str, path: &str) -> Vec<DriveView> {
        let address = format!("{}/v1/drive?path={path}", address);
        self.inner
            .get(address)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .map(|r| serde_json::from_str::<serde_json::Value>(&r).unwrap()["result"].clone())
            .map(|r| serde_json::from_value::<Vec<DriveView>>(r))
            .unwrap()
            .unwrap()
    }

    pub async fn update_dir(&self, address: &str, dir: &str, new_dir: &str) -> reqwest::Response {
        let to = if !new_dir.is_empty() {
            "to=".to_string() + new_dir
        } else {
            "".to_string()
        };
        let from = if !dir.is_empty() {
            "from=".to_string() + dir + "&"
        } else {
            "".to_string()
        };
        let address = format!("{}/v1/drive?{from}{to}", address);
        self.inner
            .put(address)
            .send()
            .await
            .expect("failed to update file")
    }

    pub async fn delete_dir(&self, address: &str, path: &str) -> reqwest::Response {
        let path = if !path.is_empty() {
            "path=".to_string() + path
        } else {
            "".to_string()
        };
        let address = format!("{}/v1/drive?{path}", address);
        self.inner
            .delete(address)
            .send()
            .await
            .expect("failed to delete file")
    }

    pub async fn create_dir(&self, address: &str, path: &str) -> reqwest::Response {
        let path = if !path.is_empty() {
            "path=".to_string() + path
        } else {
            "".to_string()
        };
        let address = format!("{}/v1/drive?{path}", address);
        self.inner
            .post(address)
            .send()
            .await
            .expect("failed to delete file")
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

#[allow(dead_code)]
pub fn random_file(base_path: String) -> PathBuf {
    let name = random_name(10);
    let path = PathBuf::from(base_path).join(name);
    let mut file = std::fs::File::create(path.clone()).unwrap();
    file.write(b"RANDOM CONTENT").unwrap();
    return path;
}

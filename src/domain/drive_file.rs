use anyhow::{anyhow, Result};
use mime_guess::mime::TEXT_PLAIN_UTF_8;
use std::path::Path;
use tokio::fs::File;
use tokio::io::BufWriter;
use tokio_util::io::ReaderStream;

pub struct DriveFile {
    file: File,
    content_type: String,
}

impl From<DriveFile> for ReaderStream<File> {
    fn from(val: DriveFile) -> Self {
        ReaderStream::new(val.file)
    }
}

impl From<DriveFile> for BufWriter<File> {
    fn from(val: DriveFile) -> Self {
        BufWriter::new(val.file)
    }
}

impl DriveFile {
    pub fn new(file: File, content_type: String) -> Self {
        Self { file, content_type }
    }

    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let content_type = mime_guess::from_path(&path)
            .first_raw()
            .unwrap_or_else(|| TEXT_PLAIN_UTF_8.as_ref());
        let file = tokio::fs::File::open(&path).await.map_err(|err| {
            tracing::error!("Error opening file: {}", err);
            anyhow!("File not found")
        })?;
        Ok(Self::new(file, content_type.to_string()))
    }

    pub async fn create(path: impl AsRef<Path>) -> Result<Self> {
        let content_type = mime_guess::from_path(&path)
            .first_raw()
            .unwrap_or_else(|| TEXT_PLAIN_UTF_8.as_ref());
        let file = tokio::fs::File::create(path).await.map_err(|err| {
            tracing::error!("{}", err);
            anyhow!("{}", err)
        })?;
        Ok(Self::new(file, content_type.to_string()))
    }

    pub fn content_type(&self) -> &str {
        self.content_type.as_ref()
    }
}

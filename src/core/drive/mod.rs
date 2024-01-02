use self::drive_file::DriveFile;
use anyhow::{anyhow, Result};
use itertools::Itertools;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use tokio::io::{AsyncRead, BufWriter};
pub mod drive_file;

pub struct Drive {
    root: String,
}

impl Default for Drive {
    fn default() -> Self {
        Self {
            root: crate::axum_server::DRIVE_DIRECTORY.to_string(),
        }
    }
}

impl Drive {
    pub fn new(root: String) -> Self {
        Self { root }
    }

    fn normalize_path(&self, path: String) -> Result<PathBuf> {
        let path = Path::new(&path);
        let has_disallowed_components = path
            .components()
            .filter(|cmp| {
                cmp == &Component::ParentDir
                    || cmp == &Component::RootDir
                    || cmp == &Component::CurDir
            })
            .count()
            > 0;
        if has_disallowed_components {
            tracing::error!("tree walk detected {:?}", path);
            return Err(anyhow!("File name is invalid"));
        }

        let path = Path::new(&self.root)
            .join(path)
            .canonicalize()
            .map_err(|err| {
                tracing::error!("Non-canonical path {:?} : {}", path, err);
                anyhow!("File not found")
            })?;

        Ok(path)
    }

    pub async fn upload<S>(&self, path: String, mut stream: S) -> Result<()>
    where
        S: AsyncRead + Unpin,
    {
        let path = self.normalize_path(path)?;

        let file = DriveFile::create(path).await?;
        let mut buffer = BufWriter::from(file);

        tokio::io::copy(&mut stream, &mut buffer)
            .await
            .map(|_| ())
            .map_err(|err| {
                tracing::error!("{}", err);
                anyhow!("{}", err)
            })
    }

    pub async fn download(&self, path: String) -> Result<DriveFile> {
        let path = self.normalize_path(path)?;
        DriveFile::open(path).await
    }

    pub async fn delete(&self, path: String) -> Result<()> {
        let path = self.normalize_path(path)?;
        tokio::fs::remove_file(&path)
            .await
            .map(|_| ())
            .map_err(|err| {
                tracing::error!("Non-canonical path {:?} : {}", path, err);
                anyhow!("File could not be deleted")
            })
    }

    pub async fn list(&self, page: usize, count: usize) -> Result<String> {
        let path = std::path::Path::new(&self.root);
        let files = path
            .read_dir()
            .map_err(|err| {
                tracing::error!("{}", err);
                anyhow!("error loading files")
            })?
            .filter_map(Result::ok)
            .sorted_by(|a, b| Ord::cmp(&a.path(), &b.path()))
            .skip(page * count)
            .take(count)
            .filter_map(|entry| entry.path().file_name().map(OsStr::to_os_string))
            .filter_map(|entry| entry.to_str().map(str::to_string));
        Ok(Itertools::intersperse(files, ",".to_string()).collect())
    }

    pub async fn search(&self, query: &str, page: usize, count: usize) -> Result<String> {
        let path = std::path::Path::new(&self.root);
        let files = path
            .read_dir()
            .map_err(|err| {
                tracing::error!("{}", err);
                anyhow::anyhow!("error loading files")
            })?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .path()
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .starts_with(query)
            })
            .sorted_by(|a, b| Ord::cmp(&a.path(), &b.path()))
            .skip(page * count)
            .take(count)
            .filter_map(|entry| entry.path().file_name().map(OsStr::to_os_string))
            .filter_map(|entry| entry.to_str().map(str::to_string));
        Ok(itertools::Itertools::intersperse(files, ",".to_string()).collect())
    }
}

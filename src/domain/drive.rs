use anyhow::{anyhow, Result};
use drive_file::DriveFile;
use itertools::Itertools;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use tokio::io::{AsyncRead, BufWriter};

use super::drive_file;

pub struct Drive {
    root: String,
}

impl Drive {
    pub fn new(root: String) -> Self {
        Self { root }
    }

    fn normalize_path(&self, path: String) -> Result<PathBuf> {
        let path = Path::new(&path);
        check_tree_walk(path)?;

        let path = Path::new(&self.root)
            .join(path)
            .canonicalize()
            .map_err(|err| anyhow!("Non-canonical path {:?} : {}", path, err.to_string()))?;

        Ok(path)
    }

    pub async fn upload<S>(&self, path: String, mut stream: S) -> Result<()>
    where
        S: AsyncRead + Unpin,
    {
        let path = Path::new(&self.root).join(&path);
        check_tree_walk(&path)?;

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

    pub async fn remove(&self, path: String) -> Result<()> {
        let path = self.normalize_path(path)?;
        tokio::fs::remove_file(&path)
            .await
            .map(|_| ())
            .map_err(|err| {
                tracing::error!("Non-canonical path {:?} : {}", path, err);
                anyhow!("File could not be deleted")
            })
    }

    pub async fn list(&self, count: usize, offset: usize) -> Result<Vec<String>> {
        let path = std::path::Path::new(&self.root);
        let files = path
            .read_dir()
            .map_err(|err| {
                tracing::error!("{}", err);
                anyhow!("error loading files")
            })?
            .filter_map(Result::ok)
            .sorted_by(|a, b| Ord::cmp(&a.path(), &b.path()))
            .skip(offset)
            .take(count)
            .filter_map(|entry| entry.path().file_name().map(OsStr::to_os_string))
            .filter_map(|entry| entry.to_str().map(str::to_string))
            .collect_vec();
        Ok(files)
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

fn check_tree_walk(path: &Path) -> Result<(), anyhow::Error> {
    let components_count = path
        .components()
        .filter(|cmp| {
            cmp == &Component::ParentDir || cmp == &Component::RootDir || cmp == &Component::CurDir
        })
        .count();
    if components_count > 0 {
        tracing::error!("tree walk detected {:?}", path);
        return Err(anyhow!("File name is invalid"));
    }
    Ok(())
}

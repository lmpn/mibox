use crate::entry::Entry;
use std::path::{Path, PathBuf};

use bytes::Buf;
use error::DriveError;
use tokio::io::AsyncReadExt;
use tokio::pin;
use tokio_util::io::ReaderStream;
pub mod entry;
pub mod error;
pub struct Drive {
    base: PathBuf,
}

type Result<T> = std::result::Result<T, DriveError>;

impl Drive {
    pub fn new(base: impl AsRef<Path>) -> Self {
        let base = base.as_ref().to_path_buf();
        Self { base }
    }

    /// Checks if the path exists and if not an error is returned.
    fn entry_exists(path: &PathBuf) -> Result<()> {
        if !path.exists() {
            return Err(DriveError::EntryNotFound(format!("{:?} not found", path)));
        }
        return Ok(());
    }

    /// Validates that the provided path does not walk through the
    /// file tree and if so an error is returned.
    ///
    /// This is achieved by checking if all the path components
    /// are of type std::path::Component::Normal.
    fn entry_valid(path: &PathBuf) -> Result<()> {
        if !path
            .components()
            .into_iter()
            .all(|component| match component {
                std::path::Component::Normal(_) => true,
                _ => false,
            })
        {
            return Err(DriveError::EntryNameInvalid(format!("{:?} invalid", path)));
        }
        Ok(())
    }

    /// The method that create an entry given a path.
    ///
    /// The entry will only be created if the path exists and there are no
    /// path walks in the final path (Self::entry_valid).
    async fn entry(&self, path: impl AsRef<Path>) -> Result<entry::Entry> {
        Self::entry_valid(&path.as_ref().to_path_buf())?;
        let entry = self.base.join(path);
        Self::entry_exists(&entry)?;
        let metadata = entry.metadata().map_err(DriveError::EntryMetadata)?;
        Ok(entry::Entry::new(entry, Some(metadata)))
    }

    /// The method that returns a PathBuf after checking it doesn't exists
    /// and there are no path walks in the final path (Self::entry_valid).
    fn entry_non_existant(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
        Self::entry_valid(&path.as_ref().to_path_buf())?;
        let entry = self.base.join(path);
        if Self::entry_exists(&entry).is_ok() {
            return Err(DriveError::EntryExists(format!(
                "{:?} already exists",
                entry
            )));
        }
        Ok(entry)
    }

    /// Create a directory if it does not exists
    pub async fn create_directory(&self, path: impl AsRef<Path>) -> Result<()> {
        let entry_to = self.entry_non_existant(path)?;
        tokio::fs::create_dir(entry_to)
            .await
            .map_err(DriveError::EntryCreate)
    }

    /// Renames a directory entry.
    ///
    /// This operation won't overwrite the `to` path.
    pub async fn rename_directory(
        &self,
        from: impl AsRef<Path>,
        to: impl AsRef<Path>,
    ) -> Result<()> {
        let entry_from = self.entry(from).await?;
        let entry_to = self.entry_non_existant(to)?;
        if !(entry_from.is_directory()) {
            return Err(DriveError::EntryUnexpectedType(format!(
                "{:?} is not an directory",
                entry_to
            )));
        }
        tokio::fs::rename(entry_from.path(), entry_to)
            .await
            .map_err(DriveError::EntryRename)
    }

    /// Queries all entries from a given path
    ///
    /// An error will be returned if the path does not correspond to a directory.
    pub async fn entries(&self, path: impl AsRef<Path>) -> Result<Vec<Entry>> {
        let path = if path.as_ref().as_os_str().is_empty() {
            &self.base
        } else {
            let entry = self.entry(path).await?;
            if !(entry.is_directory()) {
                return Err(DriveError::EntryUnexpectedType(format!(
                    "{:?} is not an directory",
                    entry
                )));
            }
            &entry.path().to_path_buf()
        };
        let mut entries = vec![];
        let mut directory = tokio::fs::read_dir(path)
            .await
            .map_err(DriveError::EntryWalk)?;
        while let Some(read_dir_entry) = directory
            .next_entry()
            .await
            .map_err(DriveError::EntryWalk)?
        {
            let path = read_dir_entry.path();
            let metadata = read_dir_entry
                .metadata()
                .await
                .map_err(DriveError::EntryMetadata)?;

            entries.push(Entry::new(path, Some(metadata)));
        }
        Ok(entries)
    }

    pub async fn download_file<
        B: Buf,
        S: futures_core::Stream<Item = std::result::Result<B, std::io::Error>>,
    >(
        &self,
        path: PathBuf,
    ) -> Result<S> {
        let entry = self.entry(path).await?;
        if entry.is_directory() {}
        let file = tokio::fs::File::open(entry.path())
            .await
            .map_err(|_e| DriveError::EntryNameInvalid(format!("invalid path")))?;
        let reader = ReaderStream::new(file);
        return Ok(reader);

        // let path = application.drive.join(params.path.clone());
        // let file = tokio::fs::File::open(path.clone())
        //     .await
        //     .context(format!("error opening file {:?}", path))?;
        //
        // if file.metadata().await.context("no metadata")?.is_file() {
        //     let reader = ReaderStream::new(file);
        //     let body = Body::from_stream(reader);
        //
        //     let headers = [
        //         (header::CONTENT_TYPE, "text/toml; charset=utf-8".to_owned()),
        //         (
        //             header::CONTENT_DISPOSITION,
        //             format!(
        //                 "attachment; filename=\"{}\"",
        //                 params.path.split('/').last().unwrap_or("")
        //             ),
        //         ),
        //     ];
        //
        //     return Ok((headers, body));
    }

    pub async fn upload_file<
        B: Buf,
        S: futures_core::Stream<Item = std::result::Result<B, std::io::Error>>,
    >(
        &self,
        stream: S,
        path: PathBuf,
    ) -> Result<()> {
        let file = tokio::fs::File::create(path.clone())
            .await
            .map_err(|_e| DriveError::EntryNameInvalid(format!("invalid path")))?;
        pin! {
            let reader = tokio_util::io::StreamReader::new(stream);
            let writer = tokio::io::BufWriter::new(file);
        };
        tokio::io::copy(&mut reader, &mut writer).await.unwrap();
        Ok(())
    }
}

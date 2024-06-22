use crate::entry::Entry;
use std::path::{Path, PathBuf};

use bytes::Buf;
use error::DriveError;
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
    fn entry_exists(path: impl AsRef<Path>) -> Result<()> {
        if !path.as_ref().exists() {
            return Err(DriveError::EntryNotFound(format!(
                "{:?} not found",
                path.as_ref()
            )));
        }
        Ok(())
    }

    /// Validates that the provided path does not walk through the
    /// file tree and if so an error is returned otherwise returns a path
    /// with the base as prefix
    ///
    /// This is achieved by checking if all the path components
    /// are of type std::path::Component::Normal.
    fn entry_valid(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
        if !path
            .as_ref()
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
        {
            return Err(DriveError::EntryNameInvalid(format!(
                "{:?} invalid",
                path.as_ref()
            )));
        }
        let entry = self.base.join(path);
        Ok(entry)
    }

    /// The method that create an entry given a path.
    ///
    /// The entry will only be created if the path exists and there are no
    /// path walks in the final path (Self::entry_valid).
    fn entry(&self, path: impl AsRef<Path>) -> Result<entry::Entry> {
        let entry = self.entry_valid(path.as_ref())?;
        Self::entry_exists(&entry)?;
        let metadata = entry.metadata().map_err(DriveError::EntryMetadata)?;
        let name = entry
            .file_name()
            .and_then(|m| m.to_str())
            .map(|m| m.to_string())
            //TODO Handle Error!!
            .unwrap();

        Ok(entry::Entry::new(entry, name, Some(metadata)))
    }

    /// The method that returns a PathBuf after checking it doesn't exists
    /// and there are no path walks in the final path (Self::entry_valid).
    fn entry_non_existant(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
        let entry = self.entry_valid(path.as_ref())?;
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
        let entry_from = self.entry(from)?;
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
            self.base.clone()
        } else {
            let entry = self.entry(path)?;
            if !(entry.is_directory()) {
                return Err(DriveError::EntryUnexpectedType(format!(
                    "{:?} is not an directory",
                    entry
                )));
            }
            entry.path().to_path_buf()
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
            let name = path
                .file_name()
                .and_then(|m| m.to_str())
                .map(|m| m.to_string())
                //TODO Handle Error!!
                .unwrap();
            let metadata = read_dir_entry
                .metadata()
                .await
                .map_err(DriveError::EntryMetadata)?;

            entries.push(Entry::new(path, name, Some(metadata)));
        }
        Ok(entries)
    }

    /// Reads the file provided by the path as a stream.
    pub async fn read(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<impl futures_core::Stream<Item = std::result::Result<bytes::Bytes, std::io::Error>>>
    {
        let entry = self.entry(path)?;
        if entry.is_directory() {
            return Err(DriveError::EntryUnexpectedType(
                "Entry is a directory".to_string(),
            ));
        }
        let file = tokio::fs::File::open(entry.path())
            .await
            .map_err(|_e| DriveError::EntryNameInvalid("invalid path".to_string()))?;
        let reader = ReaderStream::new(file);
        Ok::<ReaderStream<tokio::fs::File>, DriveError>(reader)
    }

    /// Writes the contents of the stream into a file.
    ///
    /// If destination file exists then it will be overwritten.
    pub async fn write<
        B: Buf,
        S: futures_core::Stream<Item = std::result::Result<B, std::io::Error>>,
    >(
        &self,
        stream: S,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let entry_to = self.entry_valid(path.as_ref())?;
        let file = tokio::fs::File::create(entry_to.as_path())
            .await
            .map_err(|_e| DriveError::EntryNameInvalid(format!("{:?} invalid path", entry_to)))?;
        pin! {
            let reader = tokio_util::io::StreamReader::new(stream);
            let writer = tokio::io::BufWriter::new(file);
        };
        tokio::io::copy(&mut reader, &mut writer).await.unwrap();
        Ok(())
    }
}

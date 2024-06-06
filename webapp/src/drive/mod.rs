use self::{entry::Entry, error::DriveError};
use crate::drive;
use std::path::{Path, PathBuf};
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
    fn entry_exists(path: &PathBuf) -> drive::Result<()> {
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
    fn entry_valid(path: &PathBuf) -> drive::Result<()> {
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
    async fn entry(&self, path: impl AsRef<Path>) -> drive::Result<drive::entry::Entry> {
        Self::entry_valid(&path.as_ref().to_path_buf())?;
        let entry = self.base.join(path);
        Self::entry_exists(&entry)?;
        let metadata = entry.metadata().map_err(DriveError::EntryMetadata)?;
        Ok(drive::entry::Entry::new(entry, Some(metadata)))
    }

    /// The method that returns a PathBuf after checking it doesn't exists
    /// and there are no path walks in the final path (Self::entry_valid).
    fn entry_non_existant(&self, path: impl AsRef<Path>) -> drive::Result<PathBuf> {
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
    pub async fn create_directory(&self, path: impl AsRef<Path>) -> drive::Result<()> {
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
    ) -> drive::Result<()> {
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
    pub async fn entries(&self, path: impl AsRef<Path>) -> drive::Result<Vec<Entry>> {
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
}

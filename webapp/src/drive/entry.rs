use std::{fs::Metadata, path::PathBuf};

#[derive(Debug)]
pub struct Entry {
    path: PathBuf,
    metadata: Option<Metadata>,
}

impl Entry {
    pub fn new(path: PathBuf, metadata: Option<Metadata>) -> Self {
        Self { path, metadata }
    }
    pub fn is_directory(&self) -> bool {
        match self.metadata {
            Some(ref metadata) => metadata.is_dir(),
            None => false,
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn name(&self) -> Option<String> {
        self.path
            .file_name()
            .and_then(|m| m.to_str())
            .map(|m| m.to_string())
    }
}

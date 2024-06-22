use std::{fs::Metadata, path::PathBuf};

#[derive(Debug)]
pub struct Entry {
    path: PathBuf,
    name: String,
    metadata: Option<Metadata>,
}

impl Entry {
    pub fn new(path: PathBuf, name: String, metadata: Option<Metadata>) -> Self {
        Self {
            path,
            name,
            metadata,
        }
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

    pub fn file_name(&self) -> String {
        self.name.clone()
    }
}

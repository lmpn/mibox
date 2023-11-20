use std::path::PathBuf;

#[derive(Clone)]
pub struct Application {
    pub base_url: String,
    pub drive: PathBuf,
}

impl Application {
    pub fn new(base_url: String, drive: PathBuf) -> Self {
        Self { base_url, drive }
    }
}

#[derive(thiserror::Error)]
pub enum DriveError {
    #[error("{0}")]
    EntryNameInvalid(String),
    #[error("{0}")]
    EntryUnexpectedType(String),
    #[error("{0}")]
    EntryExists(String),
    #[error("{0}")]
    EntryNotFound(String),
    #[error("error retrieving metadata")]
    EntryMetadata(#[source] std::io::Error),
    #[error("error performing entry rename operation")]
    EntryRename(#[source] std::io::Error),
    #[error("error performing entry walk operation")]
    EntryWalk(#[source] std::io::Error),
    #[error("error performing entry create operation")]
    EntryCreate(#[source] std::io::Error),
}

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

impl std::fmt::Debug for DriveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

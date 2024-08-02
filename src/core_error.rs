use glob::{GlobError, PatternError};
use std::io::Error;

#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("pattern error {0}")]
    PatternError(#[from] PatternError),

    #[error("glob error {0}")]
    GlobError(#[from] GlobError),

    #[error("io error {0}")]
    IOError(#[from] Error),
}

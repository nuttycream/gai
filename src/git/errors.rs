use std::error::Error;

/// errors for git2rs
/// or wrapper related
/// error types
#[derive(Debug)]
pub enum GitError {
    Git2(git2::Error),
    BareRepo,
    InvalidHunk(String),
    NoHead,
    Generic(String),
    PatchError,
    RebaseConflict,
}

impl std::fmt::Display for GitError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            GitError::Git2(e) => {
                write!(f, "{}", e)
            }
            GitError::BareRepo => {
                write!(f, "This is a bare repository")
            }
            GitError::InvalidHunk(h) => {
                write!(f, "Invalid Hunk:{}", h)
            }
            GitError::NoHead => write!(f, "No Head found"),
            GitError::Generic(e) => write!(f, "{}", e),
            GitError::PatchError => write!(f, "Patch Error"),
            GitError::RebaseConflict => {
                write!(f, "Conflict exists, aborting")
            }
        }
    }
}

impl Error for GitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GitError::Git2(e) => Some(e),
            _ => None,
        }
    }
}

pub mod branch;
pub mod checkout;
pub mod commit;
pub mod diffs;
pub mod errors;
pub mod lines;
pub mod log;
pub mod patches;
pub mod rebase;
pub mod repo;
pub mod staging;
pub mod status;
pub mod utils;

pub use diffs::{DiffStrategy, Diffs};
pub use repo::GitRepo;
pub use staging::StagingStrategy;
pub use status::StatusStrategy;

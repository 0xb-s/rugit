use git2::Repository as GitRepository;
use std::path::Path;

pub struct Repository {
    pub repo: GitRepository,
}

impl Repository {
    /// Initialize a new Git repository at the given path
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self, git2::Error> {
        let repo = GitRepository::init(path)?;
        Ok(Repository { repo })
    }

    /// Open an existing Git repository
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, git2::Error> {
        let repo = GitRepository::open(path)?;
        Ok(Repository { repo })
    }
}

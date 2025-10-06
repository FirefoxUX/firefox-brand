use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// RAII wrapper for temporary directories that automatically cleans up on drop
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    /// Create a new temporary directory in the system temp folder
    pub fn new(prefix: &str) -> Result<Self> {
        let temp_base = std::env::temp_dir();
        let unique_name = format!("{}-{}", prefix, uuid::Uuid::new_v4());
        let path = temp_base.join(unique_name);

        fs::create_dir_all(&path)?;

        Ok(Self { path })
    }

    /// Get the path to the temporary directory
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Create a subdirectory within the temp directory
    pub fn create_dir(&self, subpath: &str) -> Result<PathBuf> {
        let dir_path = self.path.join(subpath);
        fs::create_dir_all(&dir_path)?;
        Ok(dir_path)
    }

    /// Join a path to the temp directory
    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.path.join(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // Silently ignore cleanup errors
        let _ = fs::remove_dir_all(&self.path);
    }
}

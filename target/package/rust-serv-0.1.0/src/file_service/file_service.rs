use crate::error::Result;
use std::fs;
use std::path::Path;

/// File metadata
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub is_dir: bool,
}

/// File service for reading files and directories
pub struct FileService;

impl FileService {
    /// Read a file's content
    pub fn read_file(path: &Path) -> Result<Vec<u8>> {
        if !path.exists() {
            return Err(crate::error::Error::NotFound(path.display().to_string()));
        }
        fs::read(path).map_err(Into::into)
    }

    /// Check if path is a directory
    pub fn is_directory(path: &Path) -> bool {
        path.is_dir()
    }

    /// List directory contents
    pub fn list_directory(path: &Path) -> Result<Vec<FileMetadata>> {
        let entries = fs::read_dir(path)?;
        let mut files = Vec::new();

        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;

            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files
            if name.starts_with('.') {
                continue;
            }

            files.push(FileMetadata {
                path: entry.path().display().to_string(),
                name,
                size: metadata.len(),
                is_dir: metadata.is_dir(),
            });
        }

        files.sort_by(|a, b| {
            // Directories first
            if a.is_dir && !b.is_dir {
                return std::cmp::Ordering::Less;
            }
            if !a.is_dir && b.is_dir {
                return std::cmp::Ordering::Greater;
            }
            // Then alphabetically
            a.name.cmp(&b.name)
        });

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "hello world").unwrap();

        let content = FileService::read_file(&file_path).unwrap();
        assert_eq!(content, b"hello world");
    }

    #[test]
    fn test_read_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.txt");

        let result = FileService::read_file(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_directory() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();
        fs::create_dir(temp_dir.path().join("subdir")).unwrap();

        let files = FileService::list_directory(temp_dir.path()).unwrap();
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_list_directory_skips_hidden() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("visible.txt"), "content").unwrap();
        fs::write(temp_dir.path().join(".hidden.txt"), "hidden").unwrap();

        let files = FileService::list_directory(temp_dir.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "visible.txt");
    }

    #[test]
    fn test_list_directory_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let nonexist_path = temp_dir.path().join("nonexistent");

        let result = FileService::list_directory(&nonexist_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_directory_sorts_by_type() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join("subdir")).unwrap();
        fs::write(temp_dir.path().join("file.txt"), "content").unwrap();

        let files = FileService::list_directory(temp_dir.path()).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files[0].is_dir);
        assert!(!files[1].is_dir);
    }

    #[test]
    fn test_read_file_permission_error() {
        let result = FileService::read_file(Path::new("/root/.bashrc"));
        // On non-root systems, this should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_list_directory_empty() {
        let temp_dir = TempDir::new().unwrap();
        // Don't create any files

        let files = FileService::list_directory(temp_dir.path()).unwrap();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_is_directory_true() {
        let temp_dir = TempDir::new().unwrap();
        let dir = temp_dir.path().join("testdir");
        fs::create_dir(&dir).unwrap();

        assert!(FileService::is_directory(&dir));
    }

    #[test]
    fn test_is_directory_false() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("testfile.txt");
        fs::write(&file, "content").unwrap();

        assert!(!FileService::is_directory(&file));
    }

    #[test]
    fn test_is_directory_nonexistent() {
        assert!(!FileService::is_directory(Path::new("/nonexistent/path")));
    }
}

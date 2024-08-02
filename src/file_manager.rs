use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use glob::glob;

use crate::core_error::CoreError;

pub struct FileManager {
    files: HashMap<String, PathBuf>,
}

impl FileManager {
    pub fn load(patterns: &Vec<String>) -> Result<Self, CoreError> {
        let mut files = HashMap::<String, PathBuf>::new();
        for pattern in patterns {
            for path in glob(pattern)? {
                let path = path?;
                // Check if the path is a file
                if !path.is_file() {
                    continue;
                }
                files.insert(path.display().to_string(), path);
            }
        }
        Ok(Self { files })
    }
}

impl FileManager {
    pub fn update<F>(&self, updater: F) -> Result<(), CoreError>
    where
        F: Fn(String) -> String,
    {
        for (file_name, file_path) in &self.files {
            let mut file = fs::File::open(file_path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            let result = updater(content);
            let mut file = fs::File::create(file_name)?; // Open the file in write mode (truncate the file)
            file.write_all(result.as_bytes())?; // Write the new content
            println!("{:?}: updated", file_name)
        }
        Ok(())
    }

    pub fn print(&self) {
        eprintln!("files = {:#?}", self.files);
    }
}

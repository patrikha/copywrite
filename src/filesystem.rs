use std::ffi::OsString;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

fn is_excluded(entry: &DirEntry, excludes: &[OsString]) -> bool {
    for exclude in excludes {
        if exclude == entry.file_name() {
            log::info!("Directory {:?} is excluded, skipping.", entry.path());
            return true;
        }
    }
    false
}

pub fn walk(path: &Path, excludes: &[OsString]) -> Vec<OsString> {
    let mut files: Vec<OsString> = Vec::new();
    for entry in WalkDir::new(&path)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| !is_excluded(e, excludes))
    {
        let file_path: PathBuf = match entry {
            Ok(e) => e.path().to_path_buf(),
            Err(_) => continue,
        };
        if !file_path.is_file() {
            continue;
        }
        for exclude in excludes {
            if let Some(file_name) = file_path.file_name() {
                if exclude == file_name {
                    log::info!("File {:?} is excluded, skipping.", file_path);
                    continue;
                }
            }
        }
        if let Ok(metadata) = file_path.metadata() {
            if metadata.len() == 0 {
                log::debug!("{:?} is empty, skipping.", file_path);
                continue;
            }
        }
        files.push(file_path.as_os_str().to_os_string());
    }
    files
}

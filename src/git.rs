use git2::{Error, IndexAddOption, Repository};
use os_str_bytes::OsStrBytes;
use std::ffi::{OsStr, OsString};
use std::fs::canonicalize;
use std::path::{Path, PathBuf};

pub fn git_index(path: &Path, excludes: &Vec<OsString>) -> Result<Vec<OsString>, Error> {
    let repo = Repository::discover(path)?;
    let index = repo.index()?;
    let mut index_files: Vec<OsString> = Vec::new();
    for index_entry in index.iter() {
        let mut excluded = false;
        let index_path = OsStr::from_raw_bytes(index_entry.path).unwrap();
        let file_path = match canonicalize(PathBuf::from(&index_path)) {
            Ok(p) => p,
            Err(_) => {
                log::error!("Can't canonicalize {:?}", path);
                continue;
            }
        };
        if !file_path.starts_with(path) {
            log::info!("Git index path {:?} is not subpath of {:?}", file_path, path);
            continue;
        }
        for exclude in excludes {
            if let Some(file_name) = file_path.file_name() {
                if exclude == file_name {
                    log::info!("File {:?} is excluded, skipping.", file_path);
                    excluded = true;
                    break;
                }
            }
            for ancestor in file_path.ancestors() {
                if let Some(folder_name) = ancestor.file_name() {
                    if exclude == folder_name {
                        log::info!("Folder {:?} is excluded, skipping.", ancestor);
                        excluded = true;
                        break;
                    }
                }
            }
            if excluded {
                break;
            }
        }
        if !excluded {
            index_files.push(index_path.to_os_string());
        }
    }
    Ok(index_files)
}

pub fn git_staged(path: &Path) -> Result<Vec<OsString>, Error> {
    let repo = Repository::discover(path)?;
    let head = repo.head()?;
    let tree = head.peel_to_tree()?;
    let index = repo.index()?;
    let diff = repo.diff_tree_to_index(Some(&tree), Some(&index), None)?;
    let mut staged_files: Vec<OsString> = Vec::new();
    for delta in diff.deltas() {
        let path = OsStr::from_raw_bytes(delta.new_file().path_bytes().unwrap()).unwrap();
        staged_files.push(path.to_os_string());
    }
    Ok(staged_files)
}

pub fn git_add(path: &Path, files: &[OsString]) -> Result<(), Error> {
    let repo = Repository::discover(path)?;
    let mut index = repo.index()?;
    index.add_all(files, IndexAddOption::DEFAULT, None)?;
    index.write()?;
    Ok(())
}

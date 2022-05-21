use git2::{Error, IndexAddOption, Repository};
use os_str_bytes::OsStrBytes;
use std::ffi::{OsStr, OsString};
use std::path::Path;

pub fn git_index(path: &Path) -> Result<Vec<OsString>, Error> {
    let repo = Repository::discover(path)?;
    let index = repo.index()?;
    let mut index_files: Vec<OsString> = Vec::new();
    for index_entry in index.iter() {
        let path = OsStr::from_raw_bytes(index_entry.path).unwrap();
        index_files.push(path.to_os_string());
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

extern crate git2;
extern crate os_str_bytes;

use std::process::exit;
use std::path::Path;
use std::ffi::{OsStr, OsString};
use std::collections::HashSet;
use git2::Repository;
use os_str_bytes::OsStrBytes;

pub fn git_tree(path: &Path) -> HashSet<OsString> {
    let repo;
    match Repository::discover(path) {
        Ok(r) => repo = r,
        Err(why) => {
            log::error!("{}", why);
            exit(31);
        }
    };

    let index;
    match repo.index() {
        Ok(i) => index = i,
        Err(why) => {
            log::error!("{}", why);
            exit(32);
        }
    };

    let mut index_files = HashSet::new();
    for index_entry in index.iter() {
        let path = OsStr::from_raw_bytes(index_entry.path).unwrap();
        index_files.insert(path.to_os_string());
    }

    index_files
}

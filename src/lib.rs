use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

pub mod copywriter;
pub mod filesystem;
pub mod git;
pub mod template;

lazy_static! {
    static ref YEARS_PATTERN: Regex =
        RegexBuilder::new(r"(Copyright\s*(?:\(\s*[Cc©]\s*\)\s*))?([0-9][0-9][0-9][0-9](?:-[0-9][0-9]?[0-9]?[0-9]?)?)")
            .case_insensitive(true)
            .build()
            .unwrap();
    static ref LICENSE_PATTERN: Regex = RegexBuilder::new(r"license").case_insensitive(true).build().unwrap();
    static ref EMPTY_PATTERN: Regex = RegexBuilder::new(r"^\s*$").build().unwrap();
}

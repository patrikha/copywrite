#[macro_use]
extern crate lazy_static;

use serde::Deserialize;
use docopt::Docopt;
use std::process::exit;
use std::path::PathBuf;

mod copywriter;
mod template;
mod git;

use copywriter::copywrite_path;
use template::read_template;

const USAGE: &str = "
copywrite

Usage:
  copywrite <path> --template=TEMPLATE [--language=LANGUAGE] [--exclude=PATH]... [--gitindex]
  copywrite (-h | --help)
  copywrite --version
  copywrite -v

Options:
  --template=TEMPLATE  Path to tera (Jinja2) template file containing the copyright banner. All environment variables plus {{year}}
                       for current year are available in the template.
  --language=LANGUAGE  Restrict to only update files for specified language.
  --exclude=PATH       Exclude path, file och directory, can be repeated multiple times.
  --gitindex           Filter on files in git index only.
  -h --help            Show this screen.
  --version            Show version.
  -v                   Show shorthand version string.

Supported languages:
c, cpp, csharp, rust, go, swift, objective-c, kotlin, java, javascript, groovy, php, typescript, python, xml, svg, resx, proto, html, css, script
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_path: String,
    flag_template: String,
    flag_language: Option<String>,
    flag_exclude: Vec<String>,
    flag_gitindex: bool,
    flag_version: bool,
    flag_v: bool,
}

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn main() {
    env_logger::builder().format_timestamp(None).init();
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    // version
    if args.flag_version {
        println!("Version {}-{}", built_info::PKG_VERSION, built_info::GIT_VERSION.unwrap());
        println!("Built on {}", built_info::BUILT_TIME_UTC);
        exit(0);
    }
    if args.flag_v {
        println!("{}-{}", built_info::PKG_VERSION, built_info::GIT_VERSION.unwrap());
        exit(0);
    }

    // validate path
    let path = PathBuf::from(args.arg_path);
    if !path.exists() {
        log::error!("Invalid path {:?}", path);
        exit(1);
    }

    // template
    let template: Vec<String>;
    let template_path = PathBuf::from(args.flag_template);
    template = read_template(&template_path);

    // exclude
    let mut exclude_dirs: Vec<PathBuf> = Vec::new();
    let mut exclude_files: Vec<PathBuf> = Vec::new();
    for exclude_arg in args.flag_exclude {
        let exclude_path = PathBuf::from(exclude_arg);
        if exclude_path.exists() {
            if exclude_path.is_dir() {
                exclude_dirs.push(exclude_path);
            } else {
                exclude_files.push(exclude_path);
            }
        } else {
            log::debug!("Invalid exclude path {:?}, skipping.", exclude_path);
        }
    }

    // git tree
    let index_files;
    if args.flag_gitindex {
        index_files = Some(git::git_tree(&path));
    } else {
        index_files = None;
    }

    // update all files
    copywrite_path(&path, &template, &args.flag_language, &exclude_dirs, &exclude_files, &index_files);

    log::debug!("Done!");
    exit(0);
}

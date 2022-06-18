extern crate lazy_static;

use clap::{Arg, Command};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::exit;

use copywrite::copywriter;
use copywrite::filesystem;
use copywrite::git;
use copywrite::template::read_template;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn main() {
    env_logger::builder().format_timestamp(None).init();
    let version = format!(
        "{}-{}\nBuilt on {}",
        built_info::PKG_VERSION,
        built_info::GIT_VERSION.unwrap(),
        built_info::BUILT_TIME_UTC
    );

    let matches = Command::new("copywrite")
        .about("Add or update copyright banner in source files.")
        .version(version.as_str())
        .arg(Arg::new("PATH")
             .required(true)
             .index(1)
             .conflicts_with("v")
             .help("Path to update with copyright template."))
        .arg(Arg::new("v")
             .short('v')
             .conflicts_with_all(&["PATH", "TEMPLATE"])
             .help("Prints shorthand version information."))
        .arg(Arg::new("TEMPLATE")
             .short('t')
             .long("template")
             .required(true)
             .takes_value(true)
             .help("Path to tera (Jinja2) template file containing the copyright banner. All environment variables plus {{year}} for current year are available in the template."))
        .arg(Arg::new("LANGUAGE")
             .short('l')
             .long("language")
             .takes_value(true)
             .multiple_occurrences(true)
             .help("Restrict to only update files for specified language(s), can be repeated."))
        .arg(Arg::new("EXCLUDE")
             .short('e')
             .long("exclude")
             .takes_value(true)
             .multiple_occurrences(true)
             .help("Exclude path, file or directory name, can be repeated."))
        .arg(Arg::new("GITINDEX")
             .short('g')
             .long("gitindex")
             .conflicts_with("GITSTAGED")
             .help("Filter on files in git index only."))
        .arg(Arg::new("GITSTAGED")
             .short('d')
             .long("gitstaged")
             .conflicts_with_all(&["GITINDEX", "EXCLUDE"])
             .help("Filter on files added to git staging index only."))
        .after_help("Supported languages: c, cpp, csharp, rust, go, swift, objective-c, kotlin, java, javascript, groovy, php, typescript, python, xml, svg, resx, proto, html, css, script")
        .get_matches();

    // version
    if matches.is_present("v") {
        println!("{}-{}", built_info::PKG_VERSION, built_info::GIT_VERSION.unwrap());
        exit(0);
    }

    // validate path
    let path = PathBuf::from(matches.value_of("PATH").unwrap());
    if !path.exists() {
        log::error!("Invalid path {:?}", path);
        exit(1);
    }

    // template
    let template_path = PathBuf::from(matches.value_of("TEMPLATE").unwrap());
    let template: Vec<String> = read_template(&template_path);

    // exclude
    let mut excludes: Vec<OsString> = Vec::new();
    if matches.is_present("EXCLUDE") {
        let exclude_args: Vec<_> = matches.values_of("EXCLUDE").unwrap().collect();
        for exclude_arg in exclude_args {
            let exclude = OsString::from(exclude_arg);
            excludes.push(exclude);
        }
    }

    // languages
    let mut language_args: Vec<&str> = Vec::new();
    let languages = if matches.is_present("LANGUAGE") {
        for language_arg in matches.values_of("LANGUAGE").unwrap().collect::<Vec<_>>() {
            language_args.push(language_arg);
        }
        Some(&language_args as &[&str])
    } else {
        None
    };

    // git index / staged
    let files = if matches.is_present("GITINDEX") {
        let index_files = match git::git_index(&path, &excludes) {
            Ok(f) => f,
            Err(why) => {
                log::error!("{}", why);
                exit(2);
            }
        };
        index_files
    } else if matches.is_present("GITSTAGED") {
        let staged_files = match git::git_staged(&path) {
            Ok(f) => f,
            Err(why) => {
                log::error!("{}", why);
                exit(3);
            }
        };
        staged_files
    } else {
        filesystem::walk(&path, &excludes)
    };

    // update all files
    copywriter::copywrite_path(&files, &template, &languages);

    // if using gitstaged re-add updated files
    if matches.is_present("GITSTAGED") {
        if files.is_empty() {
            log::info!("No staged files found, skipping git add.");
        } else {
            log::info!("Re-adding {:?}", files);
            if let Err(why) = git::git_add(&path, &files) {
                log::error!("{}", why);
                exit(4);
            }
        }
    }

    log::debug!("Done!");
    exit(0);
}

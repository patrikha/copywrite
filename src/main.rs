use clap::{Arg, ArgAction, Command};
use std::ffi::OsString;
use std::fs::canonicalize;
use std::path::PathBuf;
use std::process::exit;

use copywrite::copywriter;
use copywrite::filesystem;
use copywrite::git;
use copywrite::template::read_template;

const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), ".", env!("BUILD"));

fn main() {
    env_logger::builder().format_timestamp(None).init();

    let matches = Command::new("copywrite")
        .about("Add or update copyright banner in source files.")
        .version(VERSION)
        .arg(Arg::new("BUILD")
                .short('v')
                .action(ArgAction::SetTrue)
                .conflicts_with_all(["PATH", "TEMPLATE"])
                .help("Prints shorthand version information."))
        .arg(Arg::new("PATH")
             .required(true)
             .index(1)
             .conflicts_with("BUILD"))
        .arg(Arg::new("TEMPLATE")
             .short('t')
             .long("template")
             .required(true)
             .num_args(1)
             .help("Path to tera (Jinja2) template file containing the copyright banner. All environment variables plus {{year}} for current year are available in the template."))
        .arg(Arg::new("LANGUAGE")
             .short('l')
             .long("language")
             .action(ArgAction::Append)
             .help("Restrict to only update files for specified language(s), can be repeated."))
        .arg(Arg::new("EXCLUDE")
             .short('e')
             .long("exclude")
             .action(ArgAction::Append)
             .help("Exclude path, file or directory name, can be repeated."))
        .arg(Arg::new("GITINDEX")
             .short('g')
             .long("gitindex")
             .action(ArgAction::SetTrue)
             .conflicts_with("GITSTAGED")
             .help("Filter on files in git index only."))
        .arg(Arg::new("GITSTAGED")
             .short('d')
             .long("gitstaged")
             .action(ArgAction::SetTrue)
             .conflicts_with_all(["GITINDEX", "EXCLUDE"])
             .help("Filter on files added to git staging index only."))
        .after_help("Supported languages: c, cpp, csharp, rust, go, swift, objective-c, kotlin, java, javascript, groovy, php, typescript, python, xml, svg, resx, proto, html, css, script")
        .get_matches();

    // version
    if matches.get_flag("BUILD") {
        println!("{}", VERSION);
        exit(0);
    }

    // validate path
    let path = match canonicalize(PathBuf::from(matches.get_one::<String>("PATH").unwrap())) {
        Ok(p) => p,
        Err(why) => {
            println!("Can't canonicalize path, {}", why);
            exit(1);
        }
    };
    if !path.exists() {
        log::error!("Invalid path {:?}", path);
        exit(2);
    }

    // template
    let template_path = PathBuf::from(matches.get_one::<String>("TEMPLATE").unwrap());
    let template: Vec<String> = read_template(&template_path);

    // exclude
    let excludes: Vec<OsString> = match matches.get_many::<String>("EXCLUDE") {
        Some(l) => l.map(OsString::from).collect(),
        None => vec![],
    };

    // languages
    let languages: Option<Vec<&str>> = matches
        .get_many::<String>("LANGUAGE")
        .map(|l| l.map(|s| s.as_str()).collect());

    // git index / staged
    let files = if matches.get_flag("GITINDEX") {
        match git::git_index(&path, &excludes) {
            Ok(f) => f,
            Err(why) => {
                log::error!("{}", why);
                exit(3);
            }
        }
    } else if matches.get_flag("GITSTAGED") {
        match git::git_staged(&path) {
            Ok(f) => f,
            Err(why) => {
                log::error!("{}", why);
                exit(4);
            }
        }
    } else {
        filesystem::walk(&path, &excludes)
    };

    // update all files
    copywriter::copywrite_path(&files, &template, &languages);

    // if using gitstaged re-add updated files
    if matches.get_flag("GITSTAGED") {
        if files.is_empty() {
            log::info!("No staged files found, skipping git add.");
        } else {
            log::info!("Re-adding {:?}", files);
            if let Err(why) = git::git_add(&path, &files) {
                log::error!("{}", why);
                exit(5);
            }
        }
    }

    log::debug!("Done!");
    exit(0);
}

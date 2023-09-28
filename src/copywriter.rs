use crate::{EMPTY_PATTERN, LICENSE_PATTERN, YEARS_PATTERN};
use encoding_rs;
use regex::Regex;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::{read, File};
use std::io::{self, prelude::*};
use std::path::{Path, PathBuf};
use std::process::exit;
use unicode_bom::Bom;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Language {
    extensions: Vec<OsString>,
    keep_first: Option<Regex>,
    block_comment_start_pattern: Option<Regex>,
    block_comment_end_pattern: Option<Regex>,
    line_comment_start_pattern: Option<Regex>,
    line_comment_end_pattern: Option<Regex>,
    header_start_line: Option<String>,
    header_end_line: Option<String>,
    header_line_prefix: Option<String>,
    header_line_suffix: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug)]
struct License {
    language_type: String,
    content: Content,
    skip: usize,
    head_start: Option<usize>,
    head_end: Option<usize>,
    years_line: Option<usize>,
    settings: Language,
    have_license: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Content {
    bom: Bom,
    bom_bytes: Option<Vec<u8>>,
    raw_lines: Vec<Vec<u8>>,
    lines: Vec<String>,
}

macro_rules! osvec {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push(OsString::from($x));
            )*
            temp_vec
        }
    };
}

fn create_c_style_language(extensions: Vec<OsString>) -> Language {
    Language {
        extensions,
        keep_first: None,
        block_comment_start_pattern: Some(Regex::new(r"^\s*/\*").unwrap()),
        block_comment_end_pattern: Some(Regex::new(r"\*/\s*$").unwrap()),
        line_comment_start_pattern: Some(Regex::new(r"^\s*//").unwrap()),
        line_comment_end_pattern: None,
        header_start_line: Some(String::from("/*")),
        header_end_line: Some(String::from(" */")),
        header_line_prefix: Some(String::from(" * ")),
        header_line_suffix: None,
    }
}

fn create_xml_style_language(extensions: Vec<OsString>) -> Language {
    Language {
        extensions,
        keep_first: Some(Regex::new(r"^\s*<\?xml.*\?>").unwrap()),
        block_comment_start_pattern: Some(Regex::new(r"^\s*<!--").unwrap()),
        block_comment_end_pattern: Some(Regex::new(r"-->\s*$").unwrap()),
        line_comment_start_pattern: None,
        line_comment_end_pattern: None,
        header_start_line: Some(String::from("<!--")),
        header_end_line: Some(String::from("-->")),
        header_line_prefix: Some(String::from("   ")),
        header_line_suffix: None,
    }
}

#[allow(clippy::vec_init_then_push)]
fn get_type_settings(languages: &Option<Vec<&str>>) -> HashMap<String, Language> {
    let type_settings: HashMap<String, Language> = {
        let mut t = HashMap::new();
        t.insert("c".to_string(), create_c_style_language(osvec!["c", "cc", "h"]));
        t.insert(
            "cpp".to_string(),
            create_c_style_language(osvec!["cpp", "hpp", "cxx", "hxx", "ixx"]),
        );
        t.insert("csharp".to_string(), create_c_style_language(osvec!["cs", "csx"]));
        t.insert("rust".to_string(), create_c_style_language(osvec!["rs"]));
        t.insert("go".to_string(), create_c_style_language(osvec!["go"]));
        t.insert("swift".to_string(), create_c_style_language(osvec!["swift"]));
        t.insert("objective-c".to_string(), create_c_style_language(osvec!["m", "mm"]));
        t.insert("kotlin".to_string(), create_c_style_language(osvec!["kt", "kts", "ktm"]));
        t.insert("java".to_string(), create_c_style_language(osvec!["java", "jape"]));
        t.insert("javascript".to_string(), create_c_style_language(osvec!["js", "cjs", "mjs"]));
        t.insert("groovy".to_string(), create_c_style_language(osvec!["groovy"]));
        t.insert(
            "php".to_string(),
            create_c_style_language(osvec![
                "php", "phtml", "php3", "php4", "php5", "php7", "phps", "php-s", "pht", "phar"
            ]),
        );
        t.insert(
            "typescript".to_string(),
            Language {
                extensions: osvec!["ts", "tsx"],
                keep_first: None,
                block_comment_start_pattern: Some(Regex::new(r"^\s*/\*").unwrap()),
                block_comment_end_pattern: Some(Regex::new(r"\*/\s*$").unwrap()),
                line_comment_start_pattern: Some(Regex::new(r"^\s*//").unwrap()),
                line_comment_end_pattern: None,
                header_start_line: Some(String::from("/*")),
                header_end_line: Some(String::from(" */")),
                header_line_prefix: Some(String::from(" * ")),
                header_line_suffix: None,
            },
        );
        t.insert(
            "python".to_string(),
            Language {
                extensions: osvec!["py"],
                keep_first: Some(
                    Regex::new(r"^#!|^# +pylint|^# +-\*-|^# +coding|^# +encoding|^# +type|^# +flake8").unwrap(),
                ),
                block_comment_start_pattern: None,
                block_comment_end_pattern: None,
                line_comment_start_pattern: Some(Regex::new(r"^\s*#").unwrap()),
                line_comment_end_pattern: None,
                header_start_line: Some(String::from("#")),
                header_end_line: Some(String::from("#")),
                header_line_prefix: Some(String::from("# ")),
                header_line_suffix: None,
            },
        );
        t.insert("xml".to_string(), create_xml_style_language(osvec!["xml"]));
        t.insert("svg".to_string(), create_xml_style_language(osvec!["svg"]));
        t.insert("resx".to_string(), create_xml_style_language(osvec!["resx"]));
        t.insert(
            "proto".to_string(),
            Language {
                extensions: osvec!["proto"],
                keep_first: None,
                block_comment_start_pattern: None,
                block_comment_end_pattern: None,
                line_comment_start_pattern: Some(Regex::new(r"^\s*//").unwrap()),
                line_comment_end_pattern: None,
                header_start_line: None,
                header_end_line: None,
                header_line_prefix: Some(String::from("// ")),
                header_line_suffix: None,
            },
        );
        t.insert(
            "html".to_string(),
            Language {
                extensions: osvec!["html"],
                keep_first: Some(Regex::new(r"^\s*<!DOCTYPE.*>").unwrap()),
                block_comment_start_pattern: Some(Regex::new(r"^\s*<!--").unwrap()),
                block_comment_end_pattern: Some(Regex::new(r"-->\s*$").unwrap()),
                line_comment_start_pattern: None,
                line_comment_end_pattern: None,
                header_start_line: Some(String::from("<!--")),
                header_end_line: Some(String::from("-->")),
                header_line_prefix: Some(String::from("   ")),
                header_line_suffix: None,
            },
        );
        t.insert(
            "css".to_string(),
            Language {
                extensions: osvec!["css"],
                keep_first: None,
                block_comment_start_pattern: Some(Regex::new(r"^\s*/\*").unwrap()),
                block_comment_end_pattern: Some(Regex::new(r"\*/\s*$").unwrap()),
                line_comment_start_pattern: None,
                line_comment_end_pattern: None,
                header_start_line: Some(String::from("/*")),
                header_end_line: Some(String::from("*/")),
                header_line_prefix: Some(String::from(" * ")),
                header_line_suffix: None,
            },
        );
        t.insert(
            "script".to_string(),
            Language {
                extensions: osvec!["sh", "csh", "pl"],
                keep_first: Some(Regex::new(r"^#!|^# -\*-").unwrap()),
                block_comment_start_pattern: None,
                block_comment_end_pattern: None,
                line_comment_start_pattern: Some(Regex::new(r"^\s*#").unwrap()),
                line_comment_end_pattern: None,
                header_start_line: Some(String::from("##")),
                header_end_line: Some(String::from("##")),
                header_line_prefix: Some(String::from("## ")),
                header_line_suffix: None,
            },
        );
        t
    };
    if let Some(lang_keys) = languages {
        let mut filtered_type_settings: HashMap<String, Language> = HashMap::new();
        for (key, value) in type_settings {
            if lang_keys.contains(&key.as_str()) {
                filtered_type_settings.insert(key, value);
            }
        }
        if filtered_type_settings.is_empty() {
            log::error!(
                "Specified languages {:?} are not supported, see help for more information.",
                lang_keys
            );
            exit(10);
        }
        return filtered_type_settings;
    }
    type_settings
}

fn format_template(template: &[String], settings: &Language) -> Vec<String> {
    let mut header: Vec<String> = Vec::new();
    if let Some(prefix) = settings.header_start_line.as_ref() {
        header.push(prefix.clone());
    }
    for line in template {
        let mut tmp: String = String::from(line);
        if settings.header_line_prefix.is_some() && line.is_empty() {
            tmp = format!("{}{}", settings.header_line_prefix.as_ref().unwrap().trim_end(), tmp);
        } else if settings.header_line_prefix.is_some() {
            tmp = format!("{}{}", settings.header_line_prefix.as_ref().unwrap(), tmp);
        }
        if settings.header_line_suffix.is_some() {
            tmp = format!("{}{}", tmp, settings.header_line_suffix.as_ref().unwrap())
        }
        header.push(tmp);
    }
    if let Some(suffix) = settings.header_end_line.as_ref() {
        header.push(suffix.clone());
    }

    header
}

fn read_content(path: &Path) -> io::Result<Content> {
    let buffer = read(path)?;
    let bom = Bom::from(&buffer[0..]);
    let mut bom_bytes: Option<Vec<u8>> = None;
    if bom != Bom::Null {
        log::debug!("BOM found: {:?}", bom);
        bom_bytes = Some(buffer[0..bom.len()].to_vec());
    }
    if bom == Bom::Utf16Be || bom == Bom::Utf16Le || bom == Bom::Utf32Be || bom == Bom::Utf32Le {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "copywrite don't support UTF16/32 encoded files.",
        ));
    }

    log::debug!("Reading lines: {:?}", path);
    let mut raw_lines: Vec<Vec<u8>> = Vec::new();
    let mut lines: Vec<String> = Vec::new();

    for slice in buffer[bom.len()..].split_inclusive(|b| b == &b'\n') {
        let (content, _, _) = if bom == Bom::Utf8 {
            encoding_rs::UTF_8.decode(slice)
        } else if bom == Bom::Utf16Be {
            encoding_rs::UTF_16BE.decode(slice)
        } else if bom == Bom::Utf16Le {
            encoding_rs::UTF_16LE.decode(slice)
        } else if cfg!(windows) {
            encoding_rs::WINDOWS_1252.decode(slice)
        } else {
            encoding_rs::UTF_8.decode(slice)
        };
        raw_lines.push(slice.to_vec());
        lines.push(content.to_string());
    }

    let content = Content {
        bom,
        bom_bytes,
        raw_lines,
        lines,
    };
    Ok(content)
}

#[allow(clippy::if_same_then_else)]
fn find_license(path: &Path, lang_type: &str, settings: &Language) -> Option<License> {
    let mut skip: usize = 0;
    let mut head_start = None;
    let mut years_line = None;
    let mut have_license = false;
    let content = match read_content(path) {
        Ok(c) => c,
        Err(why) => {
            log::error!("Could not read content from {:?}", path);
            log::error!("{}", why);
            return None;
        }
    };
    let lines_count = content.lines.len();
    let mut is_block_header = false;
    let mut i: usize = 0;
    for line in content.lines.iter() {
        if (i == 0 || i == skip)
            && settings.keep_first.is_some()
            && settings.keep_first.as_ref().unwrap().is_match(line)
        {
            skip = i + 1;
        } else if EMPTY_PATTERN.is_match(line) {
            // pass
        } else if settings.block_comment_start_pattern.is_some()
            && settings.block_comment_start_pattern.as_ref().unwrap().is_match(line)
        {
            head_start = Some(i);
            is_block_header = true;
            break;
        } else if settings.line_comment_start_pattern.is_some()
            && settings.line_comment_start_pattern.as_ref().unwrap().is_match(line)
        {
            head_start = Some(i);
            break;
        } else if settings.block_comment_start_pattern.is_none()
            && settings.line_comment_start_pattern.is_some()
            && settings.line_comment_start_pattern.as_ref().unwrap().is_match(line)
        {
            head_start = Some(i);
            break;
        } else {
            // we have reached something else, so no header in this file
            log::debug!("Did not find the start giving up at line {}, line is >{}<", i, line);
            return Some(License {
                language_type: lang_type.to_string(),
                content,
                skip,
                head_start: None,
                head_end: None,
                years_line: None,
                settings: settings.clone(),
                have_license,
            });
        }
        i += 1;
    }

    log::debug!(
        "Found preliminary start at {}, i={}, lines={}",
        head_start.unwrap_or_default(),
        i,
        lines_count
    );
    // now we have either reached the end, or we are at a line where a block start or line comment occurred
    // if we have reached the end, return default dictionary without info
    if i == lines_count {
        log::debug!("We have reached the end, did not find anything really");
        return Some(License {
            language_type: lang_type.to_string(),
            content,
            skip,
            head_start,
            head_end: None,
            years_line,
            settings: settings.clone(),
            have_license,
        });
    }

    // otherwise process the comment block until it ends
    if is_block_header {
        log::debug!("Found comment start, process until end");
        for j in i..lines_count {
            log::debug!("Checking line {}", j);
            if LICENSE_PATTERN.is_match(&content.lines[j]) {
                have_license = true;
            } else if settings
                .block_comment_end_pattern
                .as_ref()
                .unwrap()
                .is_match(&content.lines[j])
            {
                return Some(License {
                    language_type: lang_type.to_string(),
                    content,
                    skip,
                    head_start,
                    head_end: Some(j),
                    years_line,
                    settings: settings.clone(),
                    have_license,
                });
            } else if YEARS_PATTERN.is_match(&content.lines[j]) {
                have_license = true;
                years_line = Some(j);
            }
        }

        // if we went through all the lines without finding an end, maybe we have some syntax error or some other
        // unusual situation, so lets return no header
        log::debug!("Did not find the end of a block comment, returning no header");
        Some(License {
            language_type: lang_type.to_string(),
            content,
            skip,
            head_start: None,
            head_end: None,
            years_line: None,
            settings: settings.clone(),
            have_license,
        })
    } else {
        log::debug!("ELSE1");
        for j in i..lines_count {
            if settings
                .line_comment_start_pattern
                .as_ref()
                .unwrap()
                .is_match(&content.lines[j])
                && LICENSE_PATTERN.is_match(&content.lines[j])
            {
                have_license = true;
            } else if !settings
                .line_comment_start_pattern
                .as_ref()
                .unwrap()
                .is_match(&content.lines[j])
            {
                log::debug!("ELSE2");
                return Some(License {
                    language_type: lang_type.to_string(),
                    content,
                    skip,
                    head_start: Some(i),
                    head_end: Some(j - 1),
                    years_line,
                    settings: settings.clone(),
                    have_license,
                });
            } else if YEARS_PATTERN.is_match(&content.lines[j]) {
                have_license = true;
                years_line = Some(j);
            }
        }
        // if we went through all the lines without finding the end of the block, it could be that the whole
        // file only consisted of the header, so lets return the last line index
        log::debug!("RETURN");
        Some(License {
            language_type: lang_type.to_string(),
            content,
            skip,
            head_start: Some(i),
            head_end: Some(lines_count - 1),
            years_line,
            settings: settings.clone(),
            have_license,
        })
    }
}

fn need_update(license: &License, template: &[String]) -> bool {
    if license.have_license {
        let start = license.head_start.unwrap_or_default();
        for (i, line) in template.iter().enumerate() {
            if line.ne(&license.content.lines[i + start]) {
                return true;
            }
        }
        return false;
    }
    true
}

fn copywrite_file(path: &Path, lang_type: &str, settings: &Language, template: &[String]) {
    let license: License = match find_license(path, lang_type, settings) {
        Some(l) => l,
        None => return,
    };
    log::debug!(
        "Info for the file: head_start={:?}, head_end={:?}, have_license={}, skip={}, len={}, years_line={:?}",
        license.head_start,
        license.head_end,
        license.have_license,
        license.skip,
        license.content.lines.len(),
        license.years_line
    );
    if !need_update(&license, template) {
        log::info!("Header is up-to-date in file {:?}", path);
        return;
    }
    match File::create(path) {
        Ok(mut file) => {
            // if bom was found make sure to write it back
            if let Some(bom_bytes) = license.content.bom_bytes {
                file.write_all(&bom_bytes).expect("Can't write BOM to file");
            }
            if license.head_start.is_some() && license.head_end.is_some() && license.have_license {
                log::info!("Replacing header in file {:?}", path);
                let head_start = license.head_start.unwrap();
                let head_end = license.head_end.unwrap();
                // first write the lines before the header
                for raw_line in &license.content.raw_lines[0..head_start] {
                    file.write_all(raw_line).expect("Can't write header to file");
                }
                // now write the new header from the template lines
                for line in template {
                    writeln!(file, "{}", line).expect("Can't write template to file");
                }
                // now write the rest of the lines
                for raw_line in &license.content.raw_lines[head_end + 1..] {
                    file.write_all(raw_line).expect("Can't write body to file");
                }
            } else {
                log::info!("Adding header to file {:?}", path);
                let skip = license.skip;
                for raw_line in &license.content.raw_lines[0..skip] {
                    file.write_all(raw_line).expect("Can't write header to file");
                }
                for line in template {
                    writeln!(file, "{}", line).expect("Can't write template to file");
                }
                if license.head_start.is_some() && !license.have_license {
                    // there is some header, but not license - add an empty line
                    writeln!(file).expect("Can't write empty line");
                }
                for raw_line in &license.content.raw_lines[skip..] {
                    file.write_all(raw_line).expect("Can't write body to file");
                }
            }
        }
        Err(why) => {
            log::error!("Can't create file {:?}", path);
            log::error!("{}", why);
        }
    };
}

pub fn copywrite_path(files: &[OsString], template: &[String], languages: &Option<Vec<&str>>) {
    let type_settings = get_type_settings(languages);

    for file in files {
        let file_path = PathBuf::from(file);
        if let Some(extension) = file_path.extension() {
            for (lang_type, settings) in type_settings.iter() {
                if settings.extensions.iter().any(|x| x == extension) {
                    log::debug!("Checking file {:?}", file_path);
                    let header = format_template(template, settings);
                    copywrite_file(&file_path, lang_type, settings, &header);
                    continue;
                }
            }
        }
    }
}

use chrono::Datelike;
use std::env;
use std::fs::read_to_string;
use std::path::Path;
use std::process::exit;
use tera::{Context, Tera};

pub fn read_template(path: &Path) -> Vec<String> {
    if !path.exists() {
        log::error!("Cant find template {:?}", path);
        exit(21);
    }
    log::info!("Using template {:?}", path);
    let template: String = match read_to_string(path) {
        Ok(t) => t,
        Err(why) => {
            log::error!("{:?}", why);
            exit(22);
        }
    };

    let mut context = Context::new();
    let current_year = chrono::Utc::now().year().to_string();
    context.insert("year", &current_year);
    for (key, value) in env::vars() {
        log::debug!("Adding variable to context: {} = {}", key, value);
        context.insert(key, &value);
    }
    let result: String = match Tera::one_off(template.as_str(), &context, true) {
        Ok(r) => r,
        Err(why) => {
            log::error!("Could not create one off, {:?}", why);
            exit(23);
        }
    };

    let mut lines: Vec<String> = Vec::new();
    for part in result.replace("\r\n", "\n").split('\n') {
        lines.push(part.to_string());
    }

    lines
}

use super::BumpLevel;

extern crate cargo_version;

use std::fs;
use std::result;
use std::process::{Command, Stdio};

pub fn probability() -> f64 {
    match fs::metadata("Cargo.toml") {
        Ok(meta) => if meta.is_file() { 1f64 } else { 0f64 },
        Err(_) => 0f64,
    }
}

pub fn create_version(level: BumpLevel) {
    let level_argument = match level {
        BumpLevel::Major => cargo_version::BumpLevel::Major,
        BumpLevel::Minor => cargo_version::BumpLevel::Minor,
        BumpLevel::Patch => cargo_version::BumpLevel::Patch,
        BumpLevel::None => panic!("Cannot create a version with bump level \"None\""),
    };

    cargo_version::create_version(level_argument)
        .unwrap_or(());
        // .expect("Failed to create version");
}

pub fn publish_version() {
    Command::new("cargo")
        .arg("publish")
        .spawn()
        .expect("Failed to publish version")
        .wait()
        .expect("Failed to publish version");
}

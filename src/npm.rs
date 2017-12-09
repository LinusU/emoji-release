use super::BumpLevel;

use std::fs;
use std::process::Command;

pub fn probability() -> f64 {
    match fs::metadata("package.json") {
        Ok(meta) => if meta.is_file() { 1f64 } else { 0f64 },
        Err(_) => 0f64,
    }
}

pub fn create_version(level: BumpLevel) {
    let level_argument: &'static str = match level {
        BumpLevel::Major => "major",
        BumpLevel::Minor => "minor",
        BumpLevel::Patch => "patch",
        BumpLevel::None => panic!("Cannot create a version with bump level \"None\""),
    };

    Command::new("npm")
        .arg("version")
        .arg(level_argument)
        .arg("-m")
        .arg("🚢 %s")
        .spawn()
        .expect("Failed to create version")
        .wait()
        .expect("Failed to create version");
}

pub fn publish_version() {
    Command::new("npm")
        .arg("publish")
        .spawn()
        .expect("Failed to publish version")
        .wait()
        .expect("Failed to publish version");
}

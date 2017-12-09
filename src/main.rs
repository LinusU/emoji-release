// extern crate git2;
extern crate emoji_commit_type;

// use git2::Repository;

use std::fmt;
use std::io::BufRead;
use std::io::BufReader;
use std::process;
use std::process::{Command, Stdio};

mod npm;
mod cargo;

type CommitTitlesIter<'a> = std::io::Lines<std::io::BufReader<std::process::ChildStdout>>;

fn iter_commit_titles<'a>() -> CommitTitlesIter<'a> {
    let cmd = Command::new("git").arg("log").arg("--no-merges").arg("--format=format:%s").stdout(Stdio::piped()).spawn().unwrap();
    let buf = BufReader::new(cmd.stdout.unwrap());

    buf.lines()
}

#[derive(PartialEq, Debug)]
pub enum BumpLevel {
    Major,
    Minor,
    Patch,
    None,
}

enum Commit {
    Release,
    WithBumpLevel(BumpLevel),
    Invalid(String),
}

fn parse_commit_type(src: String) -> Commit {
    match src.chars().next() {
        Some('🚢') => Commit::Release,

        Some('💥') => Commit::WithBumpLevel(BumpLevel::Major),
        Some('🎉') => Commit::WithBumpLevel(BumpLevel::Minor),
        Some('🐛') => Commit::WithBumpLevel(BumpLevel::Patch),
        Some('🔥') => Commit::WithBumpLevel(BumpLevel::Patch),
        Some('🌹') => Commit::WithBumpLevel(BumpLevel::None),

        _ => Commit::Invalid(src),
    }
}

enum FromCommitTitlesError {
    InvalidCommitTitles(Vec<String>)
}

impl BumpLevel {
    fn max(lhs: BumpLevel, rhs: BumpLevel) -> BumpLevel {
        if lhs == BumpLevel::Major || rhs == BumpLevel::Major { return BumpLevel::Major; }
        if lhs == BumpLevel::Minor || rhs == BumpLevel::Minor { return BumpLevel::Minor; }
        if lhs == BumpLevel::Patch || rhs == BumpLevel::Patch { return BumpLevel::Patch; }

        BumpLevel::None
    }

    fn from_commit_titles(commit_titles: CommitTitlesIter) -> Result<BumpLevel, FromCommitTitlesError> {
        let mut level = BumpLevel::None;
        let mut invalids: Vec<String> = Vec::new();

        for line in commit_titles {
            let commit = parse_commit_type(line.unwrap());

            match commit {
                Commit::Release => break,
                Commit::WithBumpLevel(bump_level) => level = BumpLevel::max(level, bump_level),
                Commit::Invalid(message) => invalids.push(message),
            }
        }

        if !invalids.is_empty() {
            return Err(FromCommitTitlesError::InvalidCommitTitles(invalids))
        }

        Ok(level)
    }
}

impl fmt::Display for BumpLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BumpLevel::Major => write!(f, "major"),
            BumpLevel::Minor => write!(f, "minor"),
            BumpLevel::Patch => write!(f, "patch"),
            BumpLevel::None => write!(f, "none"),
        }
    }
}

fn git_push() -> Result<(), ()> {
    Command::new("git")
        .arg("push")
        .arg("--atomic")
        .arg("--follow-tags")
        .spawn()
        .expect("Failed to publish version")
        .wait()
        .expect("Failed to publish version");

    Ok(())
}

fn create_release(level: BumpLevel) -> Result<(), ()> {
    if cargo::probability() > 0.5 {
        cargo::create_version(level);
        try!(git_push());
        cargo::publish_version();
        return Ok(())
    }

    if npm::probability() > 0.5 {
        npm::create_version(level);
        try!(git_push());
        npm::publish_version();
        return Ok(())
    }

    Err(())
}

fn main() {
    let commit_titles = iter_commit_titles();
    let bump_level = BumpLevel::from_commit_titles(commit_titles);

    match bump_level {
        Ok(level) => {
            println!("Creating release with bump level: {}", level);

            match create_release(level) {
                Ok(()) => {
                    println!("Published new version!");
                },
                Err(_) => {
                    println!("An error occured");
                    process::exit(1);
                }
            }
        },
        Err(FromCommitTitlesError::InvalidCommitTitles(titles)) => {
            println!("The following commits have invalid titles:\n\n{}\n", titles.join("\n"));
            process::exit(1);
        }
    }
}

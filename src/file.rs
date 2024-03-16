use std::{
    ffi::OsString,
    fmt::{self, Display},
    fs,
};

use ignore::{DirEntry, WalkBuilder};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum FileType {
    Astro,
    Svelte,
    Rust,
    JavaScript,
    TypeScript,
    CSS,
    HTML,
    Python,
    Go,
    Markdown,
    JSON,
    CSV,
    SQL,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct File {
    pub name: String,
    pub path: String,
    pub loc: usize,
    pub extension: OsString,
    pub commits: Option<usize>,
}

impl File {
    pub fn get_file_type(&self) -> FileType {
        if self.name.ends_with("svelte.ts") || self.name.ends_with("svelte.js") {
            return FileType::Svelte;
        }

        match self.extension.to_str() {
            Some("js" | "cjs" | "mjs" | "jsx") => FileType::JavaScript,
            Some("ts" | "cts" | "mts" | "tsx") => FileType::TypeScript,
            Some("json" | "jsonb" | "jsonc") => FileType::JSON,
            Some("md" | "mdx") => FileType::Markdown,
            Some("svelte") => FileType::Svelte,
            Some("astro") => FileType::Astro,
            Some("py") => FileType::Python,
            Some("rs") => FileType::Rust,
            Some("css") => FileType::CSS,
            Some("html") => FileType::HTML,
            Some("go") => FileType::Go,
            Some("csv") => FileType::CSV,
            Some("sql") => FileType::SQL,
            _ => FileType::Other,
        }
    }

    pub fn add_commits(&mut self, commits: usize) {
        self.commits = Some(commits)
    }
}

impl Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn get_files(dir: &str, overrides: ignore::overrides::Override) -> Vec<File> {
    let mut files: Vec<File> = Vec::new();

    for result in WalkBuilder::new(dir)
        .overrides(overrides)
        .hidden(false)
        .build()
    {
        match result {
            Ok(entry) => {
                if entry.path().to_str().unwrap().contains(".git/") {
                    continue;
                }
                if let Some(file) = get_file_info(&entry, dir) {
                    files.push(file)
                }
            }
            Err(err) => println!("ERROR: {}", err),
        }
    }

    files
}

fn get_file_info(entry: &DirEntry, dir: &str) -> Option<File> {
    let entry = entry.clone();
    let path = entry
        .path()
        .strip_prefix(dir)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let file_name = entry.file_name().to_str().unwrap().to_string();
    let extension = match entry.path().extension() {
        Some(ext) => ext.to_owned(),
        None => OsString::from("config"),
    };

    if fs::read_to_string(entry.path()).is_ok() {
        let lines = fs::read_to_string(entry.path()).unwrap().lines().count() as usize;
        return Some(File {
            name: file_name,
            path,
            extension,
            loc: lines,
            commits: None,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_type() {
        let file = File {
            name: String::from("foo.rs"),
            path: String::from("foo.rs"),
            loc: 12,
            extension: OsString::from("rs"),
            commits: None,
        };

        assert_eq!(file.get_file_type(), FileType::Rust)
    }
}

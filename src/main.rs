use clap::{arg, command};
use ignore::{DirEntry, WalkBuilder};
use std::{collections::HashMap, env, ffi::OsString, fs};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
enum FileType {
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
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct File {
    name: String,
    path: String,
    loc: usize,
    extension: OsString,
}

impl File {
    fn get_file_type(self) -> FileType {
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
            _ => FileType::Other,
        }
    }
}

fn main() {
    let cwd = env::current_dir().unwrap();
    let default_dir = cwd.to_str().unwrap();

    let matches = command!("loc-test") // come up with better name
        .version("1.0")
        .about("Gain insights from a directory")
        .arg(arg!([directory] "Directory to search"))
        .arg(arg!([num]  "Number of files to display"))
        .arg(arg!([group] -g --group "Group the results by its extension"))
        .get_matches();

    let dir = match matches.get_one::<String>("directory") {
        Some(directory) => directory,
        None => default_dir,
    };

    let num: usize = match matches.get_one::<String>("num") {
        Some(num) => num.parse::<usize>().unwrap(),
        None => 10,
    };

    let group = matches.get_one::<bool>("group").unwrap();
    let mut files: Vec<File> = Vec::new();

    for result in WalkBuilder::new(dir).hidden(false).build() {
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

    if *group {
        grouped_info(&files)
    } else {
        simple_info(&files, num)
    }
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
        });
    }

    None
}

fn grouped_info(files: &Vec<File>) {
    let mut grouped_files: HashMap<FileType, Vec<File>> = HashMap::new();

    for file in files.iter() {
        if let Some(x) = grouped_files.get_mut(&file.clone().get_file_type()) {
            x.push(file.clone());
        } else {
            grouped_files.insert(file.clone().get_file_type(), vec![file.clone()]);
        }
    }

    let mut sorted_entries: Vec<_> = grouped_files.into_iter().collect();
    sorted_entries.sort_by(|(_, files1), (_, files2)| files2.len().cmp(&files1.len()));

    for (key, val) in sorted_entries.iter() {
        println!("Number of {:?} files: {} \n", key, val.len());
        println!(
            "Total lines of code: {}\n",
            val.iter().map(|x| x.loc).sum::<usize>()
        );
        println!("Top 5 files\n");
        let mut sorted_files: Vec<File> = val.clone().to_vec();
        sorted_files.sort_by(|a, b| b.loc.cmp(&a.loc));

        let largest_files = sorted_files.into_iter().take(5).collect::<Vec<_>>();
        for file in largest_files {
            println!("{}: {}", file.path, file.loc);
        }

        println!("\n==============");
    }
}

fn simple_info(files: &Vec<File>, num: usize) {
    let mut sorted_files: Vec<File> = files.clone().to_vec();
    sorted_files.sort_by(|a, b| b.loc.cmp(&a.loc));

    let largest_files = sorted_files.into_iter().take(num).collect::<Vec<_>>();
    for file in largest_files {
        println!("{}: {}", file.path, file.loc);
    }
}

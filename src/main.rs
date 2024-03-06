use clap::{arg, command};
use std::{env, ffi::OsString, fs, path::Path};
use walkdir::WalkDir;

fn main() {
    let cwd = env::current_dir().unwrap();
    let default_dir = cwd.to_str().unwrap();

    let matches = command!("loc-test") // come up with better name
        .version("1.0")
        .about("Gain insights from a directory")
        .arg(arg!([directory] "Directory to search"))
        .arg(arg!([group] -g --group "Group the results by its extension"))
        .get_matches();

    let dir = match matches.get_one::<String>("directory") {
        Some(directory) => directory,
        None => default_dir,
    };

    let group = matches.get_one::<bool>("group").unwrap();
    println!("group: {:?}", group);

    let gitignore_files = get_ignored_files(dir);

    let mut file_lines: Vec<(String, u64, OsString)> = Vec::new();

    for entry in WalkDir::new(dir) {
        let entry = entry.unwrap().clone();
        let path = entry.path().to_str().unwrap().to_string();
        let file_name = entry.file_name().to_str().unwrap().to_string();

        if gitignore_files.iter().any(|e| path.contains(e)) {
            continue;
        }

        if entry.path().is_file() && fs::read_to_string(entry.path()).is_ok() {
            let extension = match entry.path().extension() {
                Some(ext) => ext.to_owned(),
                None => OsString::from("config"),
            };
            let lines = fs::read_to_string(entry.path()).unwrap().lines().count() as u64;
            file_lines.push((file_name, lines, extension));
        }
    }

    println!("file_lines: {:?}", file_lines);
}

fn get_ignored_files(dir: &str) -> Vec<String> {
    let gitignore_path = Path::new(dir).join(".gitignore");
    let mut gitignore_files: Vec<String> = if gitignore_path.exists() {
        fs::read_to_string(gitignore_path)
            .unwrap()
            .lines()
            .filter_map(|s| {
                let s = s.to_string();

                if s.starts_with("#") || s.len() == 0 {
                    return None;
                }

                let s = s.replace("*", "").trim_end_matches("/").to_string();
                Some(s)
            })
            .collect()
    } else {
        Vec::new()
    };

    gitignore_files.push(".git".to_string());

    gitignore_files
}

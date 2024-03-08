use clap::{arg, command};
use std::{env, ffi::OsString, fs, path::Path};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
struct File {
    name: String,
    path: String,
    loc: usize,
    extension: OsString,
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
    let gitignore_files = get_ignored_files(dir);
    let files = get_files(dir, &gitignore_files);

    if *group {
        grouped_info(&files)
    } else {
        simple_info(&files, num)
    }
}

fn get_files(dir: &str, gitignore_files: &Vec<String>) -> Vec<File> {
    let mut files: Vec<File> = Vec::new();

    for entry in WalkDir::new(dir) {
        let entry = entry.unwrap().clone();
        let path = entry.path().to_str().unwrap().to_string();
        let file_name = entry.file_name().to_str().unwrap().to_string();
        let extension = match entry.path().extension() {
            Some(ext) => ext.to_owned(),
            None => OsString::from("config"),
        };

        if gitignore_files.iter().any(|e| path.contains(e)) {
            continue;
        }

        if entry.path().is_file() && fs::read_to_string(entry.path()).is_ok() {
            let lines = fs::read_to_string(entry.path()).unwrap().lines().count() as usize;
            files.push(File {
                name: file_name,
                path: path.replace(dir, ""),
                extension,
                loc: lines,
            });
        }
    }

    files
}

fn grouped_info(files: &Vec<File>) {
    todo!()
}

fn get_ignored_files(dir: &str) -> Vec<String> {
    let gitignore_path = Path::new(dir).join(".gitignore");
    let mut gitignore_files: Vec<String> = if gitignore_path.exists() {
        fs::read_to_string(gitignore_path)
            .unwrap()
            .lines()
            .filter_map(|s| {
                let s = s.to_string();

                if s.starts_with("#") || s.is_empty() {
                    return None;
                }

                let s = s.replace("*", "").to_string();
                Some(s)
            })
            .collect()
    } else {
        Vec::new()
    };

    gitignore_files.push(".git".to_string());

    gitignore_files
}

fn simple_info(files: &Vec<File>, num: usize) {
    // println!("files: {:?}", files);
    // println!("number of files: {}", files.len());
    // println!(
    //     "number of config files: {}",
    //     files
    //         .iter()
    //         .filter(|x| x.extension == "config")
    //         .collect::<Vec<_>>()
    //         .len()
    // );

    let mut sorted_files: Vec<File> = files.clone().to_vec();
    sorted_files.sort_by(|a, b| b.loc.cmp(&a.loc));

    let largest_files = sorted_files.into_iter().take(num).collect::<Vec<_>>();
    for file in largest_files {
        println!("{}: {}", file.name, file.loc);
    }
}

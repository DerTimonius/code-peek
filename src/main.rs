use clap::{arg, command};
use ignore::{DirEntry, WalkBuilder};
use std::{env, ffi::OsString, fs};

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
    let mut files: Vec<File> = Vec::new();

    for result in WalkBuilder::new(dir).hidden(false).build() {
        match result {
            Ok(entry) => {
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
    todo!()
}

fn simple_info(files: &Vec<File>, num: usize) {
    let mut sorted_files: Vec<File> = files.clone().to_vec();
    sorted_files.sort_by(|a, b| b.loc.cmp(&a.loc));

    let largest_files = sorted_files.into_iter().take(num).collect::<Vec<_>>();
    for file in largest_files {
        println!("{}: {}", file.path, file.loc);
    }
}

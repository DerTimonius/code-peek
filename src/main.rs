use clap::{arg, command, crate_version};
use ignore::{overrides::OverrideBuilder, DirEntry, WalkBuilder};
use nom::{
    bytes::complete::take_till,
    character::complete::{self, line_ending, multispace0, multispace1, not_line_ending},
    multi::separated_list1,
    sequence::{preceded, separated_pair},
    IResult,
};
use std::{
    collections::HashMap,
    env,
    ffi::OsString,
    fmt::{self, Display},
    fs,
    process::Command,
};
use term_table::{row::Row, TableBuilder, TableStyle};

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
    commits: Option<usize>,
}

impl File {
    fn get_file_type(&self) -> FileType {
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

    fn add_commits(&mut self, commits: usize) {
        self.commits = Some(commits)
    }
}

impl Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn main() {
    let cwd = env::current_dir().unwrap();
    let default_dir = cwd.to_str().unwrap();

    let matches = command!("code-peek")
        .name("Code Peek")
        .version(crate_version!())
        .about("A CLI tool to peek into codebases and gather insights")
        .arg(arg!([directory] "Directory to search, defauls to cwd").required(false))
        .arg(arg!([num]  "Number of files to display, defauls to 10").required(false))
        .arg(arg!([group] -g --group "Group the results by its extension").required(false))
        .arg(arg!([git] -t --git "Get git info - how many commits were made to each file").required(false))
        .arg(
            arg!([exclude]
                -e --exclude [GLOB] ... "Globs to exclude other than the files in the .gitignore, expects a comma separated list. E.g. '*.txt,*.csv'"
            )
            .required(false),
        )
        // .arg(
        //     arg!([include]
        //         -i --include [GLOB] ... "Globs to include, expects a comma separated list. E.g. '*.txt,*.csv'"
        //     )
        //     .required(false),
        // )
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
    let git = matches.get_one::<bool>("git").unwrap();

    let excludes = if let Some(globs) = matches.get_one::<String>("exclude") {
        globs.split(",").collect::<Vec<&str>>()
    } else {
        Vec::new()
    };

    // let includes = if let Some(globs) = matches.get_one::<String>("include") {
    //     globs.split(",").collect::<Vec<&str>>()
    // } else {
    //     Vec::new()
    // };

    let mut builder = OverrideBuilder::new(dir);
    for glob in excludes.iter() {
        let glob = format!("!{glob}"); // add ! to the front to exclude the glob
        builder.add(glob.as_str()).unwrap();
    }
    // for glob in includes.iter() {
    //     builder.add(glob).unwrap();
    // }
    let overrides = builder.build().unwrap();

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

    println!("Summary for project {dir}\n");
    println!("Total number of files: {}\n", files.len());
    println!(
        "Total lines of code: {}\n",
        files.iter().map(|x| x.loc).sum::<usize>()
    );

    if *git {
        add_git_info(&mut files, dir)
    }

    if *group {
        grouped_info(&files, *git)
    } else {
        simple_info(&files, num)
    }

    display_git_info(&files, num)
}

fn display_git_info(files: &Vec<File>, num: usize) {
    println!("\n===============\n");
    println!("Most changed files");
    let mut sorted_files: Vec<File> = files.clone().to_vec();
    sorted_files.sort_by(|a, b| b.commits.unwrap_or(1).cmp(&a.commits.unwrap_or(1)));

    let largest_files = sorted_files.into_iter().take(num).collect::<Vec<_>>();
    for file in largest_files {
        println!("\n{}: {} commits", file.path, file.commits.unwrap_or(1));
    }
}

fn add_git_info(files: &mut Vec<File>, dir: &str) {
    let commit_output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args([
                "/C",
                "git log --pretty=format: --name-only | sort | uniq -c",
            ])
            .current_dir(dir)
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("git log --pretty=format: --name-only | sort | uniq -c")
            .current_dir(dir)
            .output()
            .expect("failed to execute process")
    };

    if commit_output.status.success() {
        let output_str = String::from_utf8_lossy(&commit_output.stdout);
        let (_, file_map) = parse_git_commits(&output_str).expect("should parse commits");

        for file in files.iter_mut() {
            let commits = match file_map.get(file.path.as_str()) {
                Some(x) => *x as usize,
                _ => 1,
            };
            file.add_commits(commits)
        }
    }

    let author_output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "git log --format='%aN' | sort | uniq -c | sort -nr"])
            .current_dir(dir)
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("git log --format='%aN' | sort | uniq -c | sort -nr")
            .current_dir(dir)
            .output()
            .expect("failed to execute process")
    };

    if author_output.status.success() {
        let output_str = String::from_utf8_lossy(&author_output.stdout);
        let (_, authors) = parse_git_authors(&output_str).expect("should parse authors");

        println!("Most prolific contributors!\n");
        for (commits, name) in authors.iter().take(10) {
            println!("{name} contributed with {commits} commits")
        }
    }
}

fn parse_git_authors(input: &str) -> IResult<&str, Vec<(u32, &str)>> {
    let (input, authors) = separated_list1(
        line_ending,
        separated_pair(
            preceded(multispace0, complete::u32),
            multispace1,
            not_line_ending,
        ),
    )(input)?;

    Ok((input, authors))
}

fn parse_git_commits(input: &str) -> IResult<&str, HashMap<&str, u32>> {
    let (input, commits) = take_till(|x| x == '\n')(input)?;

    let commits = commits
        .trim()
        .parse::<usize>()
        .expect("should be a valid integer");

    println!("Total number of commits: {:?}", commits);

    let (input, file_info) = preceded(
        line_ending,
        separated_list1(
            line_ending,
            separated_pair(
                preceded(multispace0, complete::u32),
                multispace1,
                not_line_ending,
            ),
        ),
    )(input)?;

    let mut file_map: HashMap<&str, u32> = HashMap::new();

    for (num, name) in file_info.iter() {
        file_map.insert(name.trim(), *num);
    }

    Ok((input, file_map))
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

fn grouped_info(files: &Vec<File>, git: bool) {
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
        let mut table = TableBuilder::new()
            // .style(TableStyle::simple())
            .has_top_boarder(true)
            .style(TableStyle::elegant())
            .build();
        if git {
            table.add_row(Row::new(vec![
                key.to_string(),
                "Lines of Code".to_string(),
                "Associated commits".to_string(),
            ]));
        } else {
            table.add_row(Row::new(vec![key.to_string(), "Lines of Code".to_string()]));
        }
        // table.add_row(Row::new(vec![]));
        let mut sorted_files: Vec<File> = val.clone().to_vec();
        sorted_files.sort_by(|a, b| b.loc.cmp(&a.loc));

        let largest_files = sorted_files.into_iter().take(10).collect::<Vec<_>>();
        for file in largest_files {
            if git {
                table.add_row(Row::new(vec![
                    file.path,
                    file.loc.to_string(),
                    file.commits.unwrap_or(1).to_string(),
                ]));
            } else {
                table.add_row(Row::new(vec![file.path, file.loc.to_string()]));
            }
        }

        println!("{}", table.render());
        println!("Number of {:?} files: {} \n", key, val.len());
        println!(
            "Total lines of code: {}\n",
            val.iter().map(|x| x.loc).sum::<usize>()
        );
    }
}

fn simple_info(files: &Vec<File>, num: usize) {
    let mut sorted_files: Vec<File> = files.clone().to_vec();
    sorted_files.sort_by(|a, b| b.loc.cmp(&a.loc));

    let largest_files = sorted_files.into_iter().take(num).collect::<Vec<_>>();
    for file in largest_files {
        println!("\n{}: {}", file.path, file.loc);
    }
}

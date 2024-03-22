use std::env;

use anyhow::Result;
use clap::{arg, command, crate_version};

#[derive(Debug)]
pub struct Cli {
    pub dir: String,
    pub display_options: DisplayOptions,
    pub exclude: Vec<String>,
    pub matches: Vec<String>,
    pub num: usize,
}

#[derive(Debug)]
pub struct DisplayOptions {
    pub group: bool,
    pub git: bool,
    pub all: bool,
}

pub fn run_cli() -> Result<Cli> {
    let cwd = env::current_dir().unwrap();
    let default_dir = cwd.to_str().unwrap();

    let matches = command!("code-peek")
      .name("Code Peek")
      .version(crate_version!())
      .about("A CLI tool to peek into codebases and gather insights")
      .arg(arg!([directory] -d --dir [DIRECTORY] "Directory to search, defauls to cwd").required(false))
      .arg(arg!([num] -n --num [NUMBER]  "Number of files to display, defauls to 10").required(false))
      .arg(
          arg!([exclude]
            -e --exclude [GLOB] ... "Globs to exclude other than the files in the .gitignore, expects a comma separated list. E.g. '*.txt,*.csv'"
        )
        .required(false),
    )
    .arg(arg!([all] -a --all "Display all available information").required(false))
    .arg(arg!([group] -g --group "Group the results by its extension").required(false))
    .arg(arg!([git] -t --git "Get git info - how many commits were made to each file").required(false))
      .arg(
          arg!([match]
              -m --match [GLOB] ... "Globs to check, expects a comma separated list. E.g. '*.txt,*.csv' (Only files that match the pattern will be processed)"
          )
          .required(false),
      )
      .get_matches();

    let dir = match matches.get_one::<String>("directory") {
        Some(directory) => directory,
        None => default_dir,
    };

    let num: usize = match matches.get_one::<String>("num") {
        Some(num) => num.parse::<usize>().unwrap(),
        None => 10,
    };

    let all = matches.get_one::<bool>("all").unwrap().to_owned();
    let group = all || matches.get_one::<bool>("group").unwrap().to_owned();
    let git = all || matches.get_one::<bool>("git").unwrap().to_owned();

    let exclude = if let Some(globs) = matches.get_one::<String>("exclude") {
        globs
            .split(",")
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    } else {
        Vec::new()
    };

    let matches = if let Some(globs) = matches.get_one::<String>("match") {
        globs
            .split(",")
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    } else {
        Vec::new()
    };

    let cli = Cli {
        dir: dir.to_string(),
        num,
        display_options: DisplayOptions { all, group, git },
        exclude,
        matches,
    };

    Ok(cli)
}

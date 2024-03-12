use std::{collections::HashMap, process::Command};

use crate::file::File;
use nom::{
    bytes::complete::take_till,
    character::complete::{self, line_ending, multispace0, multispace1, not_line_ending},
    multi::separated_list1,
    sequence::{preceded, separated_pair},
    IResult,
};

#[derive(Debug)]
pub struct GitAuthor {
    pub name: String,
    pub commits: u32,
}

pub fn add_git_info(files: &mut Vec<File>, dir: &str) -> Option<usize> {
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
        let (_, (total_commits, file_map)) =
            parse_git_commits(&output_str).expect("should parse commits");

        for file in files.iter_mut() {
            let commits = match file_map.get(file.path.as_str()) {
                Some(x) => *x as usize,
                _ => 1,
            };
            file.add_commits(commits)
        }

        return Some(total_commits);
    }

    None
}

pub fn get_git_authors(dir: &str, num: usize) -> Option<Vec<GitAuthor>> {
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

        return Some(authors.into_iter().take(num).collect::<Vec<GitAuthor>>());
    }

    None
}

fn parse_git_authors(input: &str) -> IResult<&str, Vec<GitAuthor>> {
    let (input, authors) = separated_list1(
        line_ending,
        separated_pair(
            preceded(multispace0, complete::u32),
            multispace1,
            not_line_ending,
        ),
    )(input)?;

    let authors = authors
        .iter()
        .map(|(commits, name)| GitAuthor {
            name: name.to_string(),
            commits: *commits,
        })
        .collect();

    Ok((input, authors))
}

fn parse_git_commits(input: &str) -> IResult<&str, (usize, HashMap<&str, u32>)> {
    let (input, commits) = take_till(|x| x == '\n')(input)?;

    let commits = commits
        .trim()
        .parse::<usize>()
        .expect("should be a valid integer");

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

    Ok((input, (commits, file_map)))
}

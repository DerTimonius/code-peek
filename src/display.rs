use std::collections::HashMap;
use term_table::{row::Row, TableBuilder, TableStyle};

use crate::{
    cli::DisplayOptions,
    file::{File, FileType},
    git::get_git_authors,
};

pub fn display_info(
    files: &Vec<File>,
    dir: &str,
    options: DisplayOptions,
    total_commits: Option<usize>,
    num: usize,
) {
    println!("Summary for project {}\n", dir);
    println!("Total number of files: {}\n", files.len());
    println!(
        "Total lines of code: {}\n",
        files.iter().map(|x| x.loc).sum::<usize>()
    );

    if options.git {
        display_git_info(files, num, dir, total_commits.unwrap_or(0))
    }

    if options.group {
        grouped_info(files, options.git)
    } else {
        simple_info(files, num)
    }
}

fn display_git_info(files: &Vec<File>, num: usize, dir: &str, total_commits: usize) {
    println!("Information retrieved from the git log");
    println!("\n===============\n");

    println!("Total number of commits: {total_commits}");

    let authors = get_git_authors(dir, num).unwrap_or(vec![]);

    if !authors.is_empty() {
        println!("Most prolific contributors");

        for author in authors.iter().take(num) {
            let name = author.name.clone();
            let commits = author.commits;

            println!("{name} contributed with {commits} commits")
        }
    }
    println!("Most changed files");
    let mut sorted_files: Vec<File> = files.clone().to_vec();
    sorted_files.sort_by(|a, b| b.commits.unwrap_or(1).cmp(&a.commits.unwrap_or(1)));

    let largest_files = sorted_files.into_iter().take(num).collect::<Vec<_>>();
    for file in largest_files {
        println!("\n{}: {} commits", file.path, file.commits.unwrap_or(1));
    }
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

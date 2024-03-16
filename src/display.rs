use colored::Colorize;
use std::collections::HashMap;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableBuilder, TableStyle,
};

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
    println!(
        "\n{} {}\n",
        "Summary for project".bright_blue().bold(),
        dir.bright_blue().bold()
    );
    println!(
        "{} {}\n",
        "Total number of files:".bright_blue(),
        files.len()
    );
    println!(
        "{} {}\n",
        "Total lines of code:".bright_blue(),
        files.iter().map(|x| x.loc).sum::<usize>()
    );

    let mut grouped_files: HashMap<FileType, Vec<File>> = HashMap::new();

    for file in files.iter() {
        if let Some(x) = grouped_files.get_mut(&file.clone().get_file_type()) {
            x.push(file.clone());
        } else {
            grouped_files.insert(file.clone().get_file_type(), vec![file.clone()]);
        }
    }
    println!(
        "{} {}\n",
        "Number of languages used:".blue(),
        grouped_files.len()
    );

    if options.group || options.all {
        grouped_info(&grouped_files, options.git || options.all);
    } else {
        simple_info(files, num)
    }

    if options.git || options.all {
        display_git_info(files, num, dir, total_commits.unwrap_or(0))
    }
}

fn display_git_info(files: &Vec<File>, num: usize, dir: &str, total_commits: usize) {
    println!("\n===================================\n");
    println!(
        "\n{}",
        "Information retrieved from the git log".yellow().bold()
    );

    println!("{} {total_commits}\n", "Total number of commits:".yellow());

    let authors = get_git_authors(dir, num).unwrap_or(vec![]);

    if !authors.is_empty() {
        println!("-----------------------------------\n");
        println!("{}\n", "Most prolific contributors".yellow());

        let mut author_table = TableBuilder::new()
            .has_top_boarder(true)
            .style(TableStyle::thin())
            .build();
        author_table.add_row(Row::new(vec![
            TableCell::new_with_alignment(
                "Author".to_string().yellow().bold(),
                1,
                Alignment::Center,
            ),
            TableCell::new_with_alignment(
                "Number of commits".to_string().yellow().bold(),
                1,
                Alignment::Center,
            ),
        ]));

        for author in authors.iter().take(num) {
            let name = author.name.clone();
            let commits = author.commits;

            author_table.add_row(Row::new(vec![name, commits.to_string()]));
        }
        println!("{}", author_table.render())
    }
    println!("\n-----------------------------------\n");
    println!("{}", "Most changed files based on commits".yellow());
    let mut sorted_files: Vec<File> = files.clone().to_vec();
    sorted_files.sort_by(|a, b| b.commits.unwrap_or(1).cmp(&a.commits.unwrap_or(1)));

    let largest_files = sorted_files.into_iter().take(num).collect::<Vec<_>>();
    let mut commits_table = TableBuilder::new()
        .has_top_boarder(true)
        .style(TableStyle::thin())
        .build();
    commits_table.add_row(Row::new(vec![
        TableCell::new_with_alignment("File".to_string().yellow().bold(), 1, Alignment::Center),
        TableCell::new_with_alignment(
            "Number of commits".to_string().yellow().bold(),
            1,
            Alignment::Center,
        ),
    ]));
    for file in largest_files {
        commits_table.add_row(Row::new(vec![
            file.path,
            file.commits.unwrap_or(1).to_string(),
        ]));
    }

    println!("{}", commits_table.render())
}

fn grouped_info(grouped_files: &HashMap<FileType, Vec<File>>, git: bool) {
    println!("\n===================================\n");
    println!(
        "{}\n",
        "Grouped information about the files".bright_purple().bold()
    );

    let mut sorted_entries: Vec<_> = grouped_files.into_iter().collect();
    sorted_entries.sort_by(|(_, files1), (_, files2)| files2.len().cmp(&files1.len()));

    let mut file_type_table = TableBuilder::new()
        .has_top_boarder(true)
        .style(TableStyle::thin())
        .build();
    file_type_table.add_row(Row::new(vec![
        TableCell::new_with_alignment(
            "File type".to_string().bright_red().bold(),
            1,
            Alignment::Center,
        ),
        TableCell::new_with_alignment(
            "Number of files".to_string().bright_red().bold(),
            1,
            Alignment::Center,
        ),
        TableCell::new_with_alignment(
            "Lines of Code".to_string().bright_red().bold(),
            1,
            Alignment::Center,
        ),
    ]));

    let mut tables: Vec<Table> = Vec::new();

    for (key, val) in sorted_entries.iter() {
        let total_lines_of_code = val.iter().map(|x| x.loc).sum::<usize>();
        let number_of_files = val.len();
        file_type_table.add_row(Row::new(vec![
            key.to_string(),
            number_of_files.to_string(),
            total_lines_of_code.to_string(),
        ]));
        let mut table = TableBuilder::new()
            .has_top_boarder(true)
            .style(TableStyle::thin())
            .build();
        if git {
            table.add_row(Row::new(vec![
                TableCell::new_with_alignment(
                    key.to_string().bright_red().bold(),
                    1,
                    Alignment::Center,
                ),
                TableCell::new_with_alignment(
                    "Lines of Code".to_string().bright_red().bold(),
                    1,
                    Alignment::Center,
                ),
                TableCell::new_with_alignment(
                    "Number of associated commits"
                        .to_string()
                        .bright_red()
                        .bold(),
                    1,
                    Alignment::Center,
                ),
            ]));
        } else {
            table.add_row(Row::new(vec![
                TableCell::new_with_alignment(
                    key.to_string().bright_red().bold(),
                    1,
                    Alignment::Center,
                ),
                TableCell::new_with_alignment(
                    "Lines of Code".to_string().bright_red().bold(),
                    1,
                    Alignment::Center,
                ),
            ]));
        }
        let mut sorted_files: Vec<File> = val.to_vec();
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

        tables.push(table);
    }

    tables.insert(0, file_type_table);

    for table in tables.iter() {
        println!("{}", table.render())
    }
}

fn simple_info(files: &Vec<File>, num: usize) {
    println!("\n===================================\n");
    println!(
        "{}\n",
        "Largest files in your project".bright_green().bold()
    );
    let mut sorted_files: Vec<File> = files.clone().to_vec();
    sorted_files.sort_by(|a, b| b.loc.cmp(&a.loc));

    let largest_files = sorted_files.into_iter().take(num).collect::<Vec<_>>();
    let mut table = TableBuilder::new()
        .has_top_boarder(true)
        .style(TableStyle::thin())
        .build();
    table.add_row(Row::new(vec![
        TableCell::new_with_alignment(
            "File".to_string().bright_green().bold(),
            1,
            Alignment::Center,
        ),
        TableCell::new_with_alignment(
            "Lines of Code".to_string().bright_green().bold(),
            1,
            Alignment::Center,
        ),
    ]));
    for file in largest_files {
        table.add_row(Row::new(vec![file.path, file.loc.to_string()]));
    }
    println!("{}", table.render());
}

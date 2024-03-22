pub mod cli;
pub mod display;
pub mod file;
pub mod git;

use display::display_info;
use ignore::overrides::OverrideBuilder;

use crate::{cli::run_cli, file::get_files, git::add_git_info};

fn main() {
    let cli = run_cli().unwrap();
    let dir = cli.dir.as_str();

    let mut builder = OverrideBuilder::new(dir);
    for glob in cli.exclude.iter() {
        let glob = format!("!{glob}"); // add ! to the front to exclude the glob
        builder.add(glob.as_str()).unwrap();
    }
    for glob in cli.matches.iter() {
        builder.add(glob).unwrap();
    }
    let overrides = builder.build().unwrap();

    let mut files = get_files(dir, overrides);

    let total_commits = if cli.display_options.git || cli.display_options.all {
        add_git_info(&mut files, dir)
    } else {
        None
    };

    display_info(&files, dir, cli.display_options, total_commits, cli.num);
}

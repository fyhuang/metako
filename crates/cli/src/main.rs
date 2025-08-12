use std::path::PathBuf;

use clap::{command, Parser};

mod jobs;

#[derive(clap::Args)]
struct InitCommand {
    /// Path to the .mtk directory
    /// If not specified, defaults to the current directory
    metadir: Option<PathBuf>,
}

impl InitCommand {
    fn run(&self) {
        let metadir = self.metadir.as_ref().map_or_else(
            || PathBuf::from(".mtk"),
            |p| p.clone(),
        );

        // Create the .mtk directory if it doesn't exist
        if !metadir.exists() {
            std::fs::create_dir(&metadir)
                .expect(&format!("Failed to create .mtk directory at {}", metadir.display()));
        }

        // The .db files are automatically created on first use
    }
}

#[derive(Parser)]
enum Commands {
    Init(InitCommand),
    RunJobs(jobs::RunJobsCommand),
}

#[derive(Parser)]
#[command(name = "filercli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init(init) => init.run(),
        Commands::RunJobs(run_jobs) => run_jobs.run(&mtk::Vault::from_cwd()),
    }
}

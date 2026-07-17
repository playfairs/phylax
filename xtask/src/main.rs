use anyhow::Result;
use clap::{Parser, Subcommand};

mod build;
mod test;
mod lint;
mod docs;
mod package;
mod install;
mod util;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Build,
    Test,
    Lint,
    Docs,
    Package,
    Install,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Build => build::run(),
        Commands::Test => test::run(),
        Commands::Lint => lint::run(),
        Commands::Docs => docs::run(),
        Commands::Package => package::run(),
        Commands::Install => install::run(),
    }
}
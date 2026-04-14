use clap::Parser;
use anyhow::Result;

mod cli;
mod generator;
mod models;
mod utils;

use cli::{Cli, Commands, SetupMode};

fn main() -> Result<()> {
    // Parse the command line arguments
    let cli = Cli::parse();

    // Match against the provided subcommand
    match &cli.command {
        Commands::Generate { quiz, students, output, answers } => {
            println!("Starting test generation...");
            println!("Quiz: {}, Students: {}, Output: {}, Answers: {}", quiz, students, output, answers);
            generator::generate(quiz, students, output, answers)?;
            println!("Generation completed successfully.");
        }
        Commands::Mark { pdf, students, quiz, answers } => {
            println!("Starting test marking...");
            println!("PDF: {}, Students: {}, Quiz: {}", pdf, students, quiz);
            println!("Marking completed successfully.");
        }
        Commands::Setup { mode } => {
            match mode {
                SetupMode::Empty => {
                    println!("Creating empty templates...");
                }
                SetupMode::Interactive => {
                    println!("Starting interactive setup...");
                }
            }
        }
    }

    Ok(())
}
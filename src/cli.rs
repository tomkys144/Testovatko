use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    /// Generate PDF tests for students.
    Generate {
        #[arg(long, default_value = "quiz.json")]
        quiz: String,

        #[arg(long, default_value = "students.json")]
        students: String,

        #[arg(long, default_value = "quiz.pdf")]
        output: String,

        #[arg(long, default_value = "quiz_ans.json")]
        answers: String,
    },
    /// Mark scanned tests.
    Mark {
        #[arg(long, default_value = "quiz-filled.pdf")]
        pdf: String,

        #[arg(long, default_value = "students.json")]
        students: String,

        #[arg(long, default_value = "quiz.json")]
        quiz: String,

        #[arg(long, default_value = "quiz_ans.json")]
        answers: String,
    },
    /// Utility scripts to generate configuration files.
    Setup {
        #[arg(value_enum)]
        mode: SetupMode,
    },
}

#[derive(clap::ValueEnum, Clone)]
pub enum SetupMode {
    Empty,
    Interactive,
}
mod ingestor;
mod detector;
mod grader;
mod reporter;

use crate::models::{Quiz, Student};
use anyhow::{Context, Result};
use std::fs;

pub fn mark(
    pdf_path: &str,
    quiz_path: &str,
    students_path: &str,
    answers_path: &str,
) -> Result<()> {
    let quiz_str = fs::read_to_string(quiz_path).context("Reading quiz file")?;
    let quiz: Quiz = serde_json::from_str(&quiz_str).context("Deserializing quiz file")?;

    let students_str = fs::read_to_string(students_path).context("Reading students file")?;
    let students: Vec<Student> =
        serde_json::from_str(&students_str).context("Deserializing students file")?;

    let tests = ingestor::load_quiz(pdf_path)?;

    for (username, test) in tests.iter() {
        println!("Processing {}", username);

    }

    Ok(())
}

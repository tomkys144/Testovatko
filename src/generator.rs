mod compiler;
mod composer;
mod renderer;

use compiler::TypstWrapperWorld;
use composer::make_quiz;
use renderer::render_final;

use crate::models::{Quiz, Student};
use anyhow::{Context, Result};
use std::collections::{BTreeMap};
use std::fs;

const TYPST_TEMPLATE: &str = include_str!("../assets/template.typ");
const ARUCO_MARKERS: [&[u8]; 8] = [
    include_bytes!("../assets/marker_0.png"),
    include_bytes!("../assets/marker_1.png"),
    include_bytes!("../assets/marker_2.png"),
    include_bytes!("../assets/marker_3.png"),
    include_bytes!("../assets/marker_4.png"),
    include_bytes!("../assets/marker_5.png"),
    include_bytes!("../assets/marker_6.png"),
    include_bytes!("../assets/marker_7.png"),
];

pub fn generate(
    quiz_path: &str,
    students_path: &str,
    output_path: &str,
    answers_path: &str,
) -> Result<()> {
    let quiz_str = fs::read_to_string(quiz_path).context("Reading quiz file")?;
    let quiz: Quiz = serde_json::from_str(&quiz_str).context("Deserializing quiz file")?;

    let students_str = fs::read_to_string(students_path).context("Reading students file")?;
    let students: Vec<Student> =
        serde_json::from_str(&students_str).context("Deserializing students file")?;

    // Deploy markers
    for (i, &image_bytes) in ARUCO_MARKERS.iter().enumerate() {
        fs::write(format!("marker_{}.png", i), image_bytes)?;
    }

    let mut master_answers: BTreeMap<String, BTreeMap<usize, Vec<String>>> = BTreeMap::new();
    let mut generated_pdfs: Vec<String> = Vec::new();

    let current_dir = std::env::current_dir()?.to_string_lossy().to_string();

    for student in &students {
        let tmp_pdf = format!("tmp-{}.pdf", student.username);

        let (quiz_markup, answers) = make_quiz(&quiz, student)?;
        master_answers.insert(student.username.clone(), answers);

        let world = TypstWrapperWorld::new(current_dir.clone(), quiz_markup);

        let document = typst::compile(&world)
            .output
            .map_err(|e| anyhow::anyhow!("Typst Compilation Error: {:?}", e))?;

        let options = Default::default();
        let pdf = typst_pdf::pdf(&document, &options).expect("Error exporting PDF");

        fs::write(&tmp_pdf, pdf).expect("Error writing PDF.");
        generated_pdfs.push(tmp_pdf);
    }

    render_final(&generated_pdfs, output_path)?;

    // Clean up
    for pdf in generated_pdfs {
        let _ = fs::remove_file(pdf);
    }
    for i in 0..8 {
        let _ = fs::remove_file(format!("marker_{}.png", i));
    }

    let json_file = fs::File::create(answers_path).context("Failed to create JSON file")?;
    serde_json::to_writer_pretty(json_file, &master_answers)
        .context("Failed to write answers to JSON")?;

    Ok(())
}

mod ingestor;
mod detector;
mod grader;
mod reporter;

use crate::models::{Quiz, Student};
use anyhow::{Context, Result};
use std::fs;
use image::DynamicImage;

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

    let (doc, tests) = ingestor::load_quiz(pdf_path)?;

    for (username, test) in tests.iter() {
        println!("Processing {}", username);
        for (page_num, pdf_idx) in test.iter() {
            let page = ingestor::load_page(&doc, *pdf_idx, Some(4.0))?;
            let (mut aligned_page, tables) = detector::detect_tables_and_align(&page)?;

            for table in tables {
                if table.table_type == 0 {
                    continue;
                }

                let tab_img = aligned_page.crop(table.x, table.y, table.width, table.height);
                debug_save_img(&tab_img, &format!("{}_table", username));

                grader::mark_section(&tab_img);

            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn debug_save_img(img: &DynamicImage, name: &str) {
    let path = format!("/tmp/{}.png", name);
    img.save(&path).expect("Failed to save debug image");
    println!("Debug image saved to {}", path);
}
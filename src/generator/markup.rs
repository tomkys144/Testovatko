use crate::models::{Quiz, Section, Student};
use anyhow::Result;
use rand::rng;
use rand::seq::SliceRandom;
use std::collections::HashMap;

const TYPST_TEMPLATE: &str = include_str!("../../assets/template.typ");

pub fn make_quiz(quiz: &Quiz, student: &Student) -> Result<(String, HashMap<usize, Vec<String>>)> {
    let mut quiz_markup: String = TYPST_TEMPLATE.to_string();
    quiz_markup.push_str("#set heading(numbering: \"1.1.\")\n\n");
    quiz_markup.push_str(&format!(
        "#show: doc => exam_setup(\"{}\", \"{}\", doc)\n\n",
        student.name, student.username
    ));
    quiz_markup.push_str(&format!(
        "#assignment_header(\"{}\", \"{}\", \"{}\", \"{}\", \"{}\", \"{}\")\n",
        quiz.title,
        quiz.group.clone().unwrap_or_default(),
        quiz.class,
        quiz.date,
        student.name,
        student.username
    ));
    quiz_markup.push_str(&make_points_summary(quiz, Some(0))?);

    let mut answers = HashMap::new();
    for (section_num, section) in quiz.sections.iter().enumerate() {
        let (section_markup, section_answers) = make_section(section, section_num as u32 + 1)?;
        quiz_markup.push_str(&section_markup);
        answers.insert(section_num, section_answers);
    }
    quiz_markup.push_str(&format!(
        "\n#finish_exam(\"{}\", \"{}\")\n",
        student.name, student.username
    ));

    Ok((quiz_markup, answers))
}

fn make_points_summary(quiz: &Quiz, start_marker_id: Option<u32>) -> Result<String> {
    let points: Vec<u32> = quiz
        .sections
        .iter()
        .map(|section| section.questions.iter().map(|q| q.points).sum())
        .collect();

    let total_points: u32 = points.iter().sum();
    let num_sections = quiz.sections.len();

    let mut points_markup = String::from("#table(\n");

    points_markup.push_str(&format!(
        "\tcolumns: ({}),\n",
        vec!["5em"; num_sections + 1].join(", ")
    ));
    points_markup.push_str("\talign: center,\n");
    points_markup.push_str("\tinset: 8pt,\n");

    let mut headers: Vec<String> = (1..=num_sections)
        .map(|i| format!("text(weight: \"bold\")[{}]", i))
        .collect();
    headers.push(String::from("$Sigma$"));
    points_markup.push_str(&format!("\t{},\n", headers.join(", ")));

    let mut points_row: Vec<String> = points
        .iter()
        .map(|&pts| format!("text(weight: \"bold\")[{}]", pts))
        .collect();
    points_row.push(format!("text(weight: \"bold\")[{}]", total_points));
    points_markup.push_str(&format!("\t{},\n", points_row.join(", ")));

    let empty_row = vec!["v(1.5em)"; num_sections + 1].join(", ");
    points_markup.push_str(&format!("\t{},\n)\n", empty_row));

    let table_markup = format!(
        "#add_markers_around_table(start_id: {})[\n{}\n]\n",
        start_marker_id.unwrap_or(0),
        points_markup
    );

    Ok(table_markup)
}

fn make_section(section: &Section, section_num: u32) -> Result<(String, Vec<String>)> {
    let mut section_markup: String = format!("= {}\n\n", section.title);
    let mut answers = Vec::new();
    let mut gnr = rng();

    match section.section_type.as_str() {
        "open" => {
            for (q_idx, question) in section.questions.iter().enumerate() {
                section_markup.push_str(&format!(
                    "#open_question(points: {}, lines: {}, qnum: \"{}.{}\")[{}]\n\n",
                    question.points,
                    question.lines.unwrap_or(5),
                    section_num,
                    q_idx + 1,
                    question.question
                ));
            }
        }
        "multiple_choice" => {
            for (q_idx, question) in section.questions.iter().enumerate() {
                let mut options = question.options.clone().unwrap_or_default();
                options.shuffle(&mut gnr);

                let mut opt_strings = Vec::new();

                for (opt_dx, option) in options.iter().enumerate() {
                    opt_strings.push(format!("[{}]", option.text));

                    if option.is_correct {
                        let label = (b'A' + opt_dx as u8) as char;
                        answers.push(label.to_string());
                    }
                }

                section_markup.push_str(&format!(
                    "#multiple_choice_question(points: {}, qnum: \"{}.{}\", options: ({}))[{}]\n\n",
                    question.points,
                    section_num,
                    q_idx + 1,
                    opt_strings.join(", "),
                    question.question
                ));
            }
        }
        _ => {}
    }

    Ok((section_markup, answers))
}
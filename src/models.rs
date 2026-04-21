use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Student {
    pub name: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Quiz {
    pub title: String,
    pub class: String,
    pub group: Option<String>,
    pub date: String,
    pub sections: Vec<Section>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Section {
    pub title: String,
    #[serde(rename = "type")]
    pub section_type: String,
    pub description: String,
    pub questions: Vec<Question>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Question {
    pub question: String,
    pub points: u32,
    pub lines: Option<u32>,
    pub options: Option<Vec<QuestOption>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QuestOption {
    pub text: String,
    pub is_correct: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct TableLocation {
    /// 0 for Results, 1 for Choices
    pub table_type: i32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
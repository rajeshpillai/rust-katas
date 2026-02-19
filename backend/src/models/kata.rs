use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Kata {
    pub id: String,
    pub phase: u32,
    pub phase_title: String,
    pub sequence: u32,
    pub title: String,
    pub hints: Vec<String>,
    pub description: String,
    pub broken_code: String,
    pub correct_code: String,
    pub explanation: String,
    pub compiler_error_interpretation: String,
}

#[derive(Debug, Serialize)]
pub struct KataListResponse {
    pub phases: Vec<PhaseGroup>,
}

#[derive(Debug, Serialize)]
pub struct PhaseGroup {
    pub phase: u32,
    pub title: String,
    pub katas: Vec<KataSummary>,
}

#[derive(Debug, Serialize)]
pub struct KataSummary {
    pub id: String,
    pub sequence: u32,
    pub title: String,
}

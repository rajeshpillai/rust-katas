use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;

use crate::models::kata::{Kata, KataListResponse, KataSummary, PhaseGroup};

pub async fn list_katas(State(katas): State<Arc<Vec<Kata>>>) -> Json<KataListResponse> {
    let mut phases: Vec<PhaseGroup> = Vec::new();

    for kata in katas.iter() {
        let group = phases.iter_mut().find(|g| g.phase == kata.phase);
        let summary = KataSummary {
            id: kata.id.clone(),
            sequence: kata.sequence,
            title: kata.title.clone(),
        };

        match group {
            Some(g) => g.katas.push(summary),
            None => phases.push(PhaseGroup {
                phase: kata.phase,
                title: kata.phase_title.clone(),
                katas: vec![summary],
            }),
        }
    }

    phases.sort_by_key(|g| g.phase);
    for group in &mut phases {
        group.katas.sort_by_key(|k| k.sequence);
    }

    Json(KataListResponse { phases })
}

pub async fn get_kata(
    State(katas): State<Arc<Vec<Kata>>>,
    Path(id): Path<String>,
) -> Result<Json<Kata>, StatusCode> {
    katas
        .iter()
        .find(|k| k.id == id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

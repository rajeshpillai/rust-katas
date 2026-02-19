use axum::Json;

use crate::models::execution::{ExecutionRequest, ExecutionResult};
use crate::services::sandbox;

pub async fn run_code(Json(req): Json<ExecutionRequest>) -> Json<ExecutionResult> {
    Json(sandbox::execute_rust_code(&req.code).await)
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ExecutionRequest {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub execution_time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ExecutionResult {
    pub fn error(msg: String) -> Self {
        Self {
            stdout: String::new(),
            stderr: String::new(),
            success: false,
            execution_time_ms: 0,
            error: Some(msg),
        }
    }
}

use crate::models::execution::ExecutionResult;
use std::time::Instant;
use tempfile::TempDir;
use tokio::time::{timeout, Duration};

const COMPILE_TIMEOUT: Duration = Duration::from_secs(10);
const RUN_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn execute_rust_code(code: &str) -> ExecutionResult {
    let start = Instant::now();

    // Create isolated temp directory (auto-cleans on drop)
    let tmp_dir = match TempDir::new() {
        Ok(d) => d,
        Err(e) => return ExecutionResult::error(format!("Failed to create temp dir: {}", e)),
    };

    let source_path = tmp_dir.path().join("main.rs");
    let binary_path = tmp_dir.path().join("main");

    // Write source file
    if let Err(e) = tokio::fs::write(&source_path, code).await {
        return ExecutionResult::error(format!("Failed to write source: {}", e));
    }

    // Compile with timeout
    let compile_result = timeout(
        COMPILE_TIMEOUT,
        tokio::process::Command::new("rustc")
            .arg("--edition")
            .arg("2021")
            .arg(&source_path)
            .arg("-o")
            .arg(&binary_path)
            .output(),
    )
    .await;

    let compile_output = match compile_result {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => return ExecutionResult::error(format!("Failed to run rustc: {}", e)),
        Err(_) => return ExecutionResult::error("Compilation timed out (10s limit)".into()),
    };

    if !compile_output.status.success() {
        return ExecutionResult {
            stdout: String::new(),
            stderr: String::from_utf8_lossy(&compile_output.stderr).to_string(),
            success: false,
            execution_time_ms: start.elapsed().as_millis() as u64,
            error: None,
        };
    }

    // Run the compiled binary with timeout
    let run_result = timeout(
        RUN_TIMEOUT,
        tokio::process::Command::new(&binary_path).output(),
    )
    .await;

    let run_output = match run_result {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => return ExecutionResult::error(format!("Failed to run binary: {}", e)),
        Err(_) => return ExecutionResult::error("Execution timed out (5s limit)".into()),
    };

    ExecutionResult {
        stdout: String::from_utf8_lossy(&run_output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&run_output.stderr).to_string(),
        success: run_output.status.success(),
        execution_time_ms: start.elapsed().as_millis() as u64,
        error: None,
    }
}

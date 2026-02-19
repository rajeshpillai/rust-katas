use crate::models::kata::Kata;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct KataFrontmatter {
    id: String,
    phase: u32,
    phase_title: String,
    sequence: u32,
    title: String,
    #[serde(default)]
    hints: Vec<String>,
}

pub fn load_all_katas(katas_dir: &Path) -> Result<Vec<Kata>, Box<dyn std::error::Error>> {
    let mut katas = Vec::new();

    if !katas_dir.exists() {
        tracing::warn!("Katas directory not found: {}", katas_dir.display());
        return Ok(katas);
    }

    let mut phase_dirs: Vec<_> = std::fs::read_dir(katas_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    phase_dirs.sort_by_key(|e| e.file_name());

    for phase_dir in phase_dirs {
        let mut files: Vec<_> = std::fs::read_dir(phase_dir.path())?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext == "md")
            })
            .collect();
        files.sort_by_key(|e| e.file_name());

        for file in files {
            match parse_kata_file(&file.path()) {
                Ok(kata) => {
                    tracing::info!("Loaded kata: {} (phase {})", kata.title, kata.phase);
                    katas.push(kata);
                }
                Err(e) => {
                    tracing::error!("Failed to parse {}: {}", file.path().display(), e);
                }
            }
        }
    }

    katas.sort_by(|a, b| a.phase.cmp(&b.phase).then(a.sequence.cmp(&b.sequence)));
    tracing::info!("Loaded {} katas total", katas.len());
    Ok(katas)
}

fn parse_kata_file(path: &Path) -> Result<Kata, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;

    // Split frontmatter from body
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Err(format!("Invalid frontmatter in {}", path.display()).into());
    }

    let frontmatter: KataFrontmatter = serde_yaml::from_str(parts[1].trim())?;
    let body = parts[2];

    // Extract sections by ## headings
    let description = extract_section(body, "Description");
    let broken_code = extract_code_block(body, "Broken Code");
    let correct_code = extract_code_block(body, "Correct Code");
    let explanation = extract_section(body, "Explanation");
    let compiler_error = extract_section(body, "Compiler Error Interpretation");

    Ok(Kata {
        id: frontmatter.id,
        phase: frontmatter.phase,
        phase_title: frontmatter.phase_title,
        sequence: frontmatter.sequence,
        title: frontmatter.title,
        hints: frontmatter.hints,
        description,
        broken_code,
        correct_code,
        explanation,
        compiler_error_interpretation: compiler_error,
    })
}

fn extract_section(body: &str, heading: &str) -> String {
    let marker = format!("## {}", heading);
    let Some(start) = body.find(&marker) else {
        return String::new();
    };

    let after_heading = &body[start + marker.len()..];
    // Find the next ## heading or end of string
    let end = after_heading
        .find("\n## ")
        .unwrap_or(after_heading.len());

    after_heading[..end].trim().to_string()
}

fn extract_code_block(body: &str, heading: &str) -> String {
    let section = extract_section(body, heading);
    // Find the first ```rust ... ``` block, or just ``` ... ```
    let Some(start) = section.find("```") else {
        return section;
    };

    let after_backticks = &section[start + 3..];
    // Skip the language tag line
    let code_start = after_backticks.find('\n').map_or(0, |i| i + 1);
    let code_body = &after_backticks[code_start..];

    let end = code_body.find("```").unwrap_or(code_body.len());
    code_body[..end].trim().to_string()
}

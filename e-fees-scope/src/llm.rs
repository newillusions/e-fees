//! Ollama LLM client for scope polishing.

use crate::error::ApiError;
use serde_json::{json, Value};

/// Send raw scope text to Ollama for professional polish.
///
/// The LLM refines language, specificity, and professionalism while
/// preserving structure, numbering, and deliverables exactly.
pub async fn polish_scope(
    http: &reqwest::Client,
    ollama_url: &str,
    model: &str,
    project_context: &Value,
    raw_scope: &str,
    similar_examples: &[String],
    stage_context: Option<&str>,
) -> Result<String, ApiError> {
    let examples_text = if similar_examples.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nReference examples from similar past proposals:\n{}",
            similar_examples.join("\n---\n")
        )
    };

    let stage_text = match stage_context {
        Some(ctx) if !ctx.is_empty() => format!("\n\nProject stages: {}", ctx),
        _ => String::new(),
    };

    let prompt = format!(
        "Given this project context:\n{}{}\n\n\
         Refine these scope of services clauses for professional tone and project specificity. \
         Maintain the exact structure, numbering, and deliverables. Do not add or remove items. \
         Only improve language, specificity, and professionalism.\n\n\
         Scope text:\n{}\n{}",
        serde_json::to_string_pretty(project_context).unwrap_or_default(),
        stage_text,
        raw_scope,
        examples_text
    );

    let body = json!({
        "model": model,
        "prompt": prompt,
        "system": "You are a senior lighting design consultant writing scope of services \
                    for fee proposals. Write in clear, professional English. \
                    Be specific to the project context provided.",
        "stream": false,
        "think": false,
        "options": {
            "temperature": 0.3
        }
    });

    let res = http
        .post(format!("{}/api/generate", ollama_url))
        .json(&body)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| ApiError::service_unavailable(format!("Ollama error: {}", e)))?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(ApiError::service_unavailable(format!(
            "Ollama returned {}: {}",
            status, text
        )));
    }

    let result: Value = res
        .json()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to parse Ollama response: {}", e)))?;

    result["response"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| ApiError::internal("Ollama response missing 'response' field"))
}

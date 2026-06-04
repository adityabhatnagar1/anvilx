// src/providers/ollama.rs
//
// Ollama local model provider — runs AI on YOUR machine, $0 cost.
//
// Ollama exposes an OpenAI-compatible API on localhost:11434.
// No API key needed. No data leaves your machine.
//
// INSTALL:
//   curl -fsSL https://ollama.com/install.sh | sh
//   ollama pull llama3.2      # 2GB, fast on CPU
//   ollama serve              # start the local server

use anyhow::{Context, Result};
use std::time::Instant;
use super::ProviderResponse;

/// Ollama's default local endpoint. Doesn't leave your machine.
const OLLAMA_BASE: &str = "http://localhost:11434";

/// Ask a locally running Ollama model.
///
/// # Arguments
/// * `prompt` - The user's question
/// * `model`  - Local model name, e.g. "llama3.2", "deepseek-r1:7b"
pub async fn ask(prompt: &str, model: &str) -> Result<ProviderResponse> {
    let start  = Instant::now();
    let client = reqwest::Client::new();

    // Ollama uses a slightly different endpoint (/api/chat) but same JSON shape as OpenAI
    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "user", "content": prompt }
        ],
        "stream": false  // We want the full response at once, not a stream
    });

    let response = client
        .post(format!("{}/api/chat", OLLAMA_BASE))
        .json(&body)
        .send()
        .await
        .context("Ollama not reachable. Is it running? Try: ollama serve")?;

    let status = response.status();
    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "could not read error body".to_string());
        anyhow::bail!("Ollama returned HTTP {}: {}", status, error_body);
    }

    let resp: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse Ollama response as JSON")?;


    
    let input_tokens = resp["prompt_eval_count"]
    .as_u64()
    .unwrap_or(0) as u32;

    let output_tokens = resp["eval_count"]
        .as_u64()
        .unwrap_or(0) as u32;

    // Ollama's response shape: { "message": { "content": "..." }, ... }
    let text = resp["message"]["content"]
        .as_str()
        .unwrap_or("(empty response)")
        .to_string();

    Ok(ProviderResponse {
        model: format!("ollama/{}", model),
        text,
        input_tokens,
        output_tokens,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

/// Ping Ollama to check if it's running.
/// Used by the router to fall back to a cloud model if Ollama is offline.
pub async fn is_available() -> bool {
    reqwest::Client::new()
        .get(format!("{}/", OLLAMA_BASE))
        .send()
        .await
        .map(|r| r.status().is_success() || r.status().as_u16() == 200)
        .unwrap_or(false)
}
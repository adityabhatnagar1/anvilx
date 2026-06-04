// src/providers/openai.rs
//
// OpenAI GPT provider adapter.
// Calls the /v1/chat/completions endpoint and returns a ProviderResponse.
//
// SECURITY NOTES:
//   - API key comes from env var, never hardcoded or logged
//   - We use rustls-tls (not native-tls) for TLS — pure Rust, no system OpenSSL risk
//   - Timeout is enforced by the caller (tokio::time::timeout in main.rs)
//   - We never log the full prompt in production (PII risk)

use anyhow::{Context, Result};
use std::time::Instant;
use super::ProviderResponse;

/// Ask OpenAI GPT a question. Returns a uniform ProviderResponse.
///
/// # Arguments
/// * `prompt`  - The user's question / instruction
/// * `api_key` - Your OpenAI API key (from OPENAI_API_KEY env var)
/// * `model`   - Which GPT model, e.g. "gpt-4o-mini" or "gpt-4o"
pub async fn ask(prompt: &str, api_key: &str, model: &str) -> Result<ProviderResponse> {
    // Record wall-clock time so we can report latency
    let start = Instant::now();

    // Build an HTTP client. reqwest::Client is cheap to create but
    // you'd normally share one across calls in production. Fine here.
    let client = reqwest::Client::new();

    // Build the JSON body that OpenAI's /v1/chat/completions expects.
    // We keep max_tokens at 1024 — enough for real answers, safe from runaway costs.
    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "user", "content": prompt }
        ],
        "max_tokens": 1024,
        "temperature": 0.7
    });

    // Make the HTTP request.
    // The ? operator propagates errors up to main — no panics here.
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        // Authorization header — Bearer token pattern
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .context("Failed to reach OpenAI API — check your internet connection")?;

    // Check the HTTP status before parsing the body.
    // A 401 means bad API key; 429 means rate-limited.
    let status = response.status();
    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "could not read error body".to_string());
        anyhow::bail!(
            "OpenAI returned HTTP {}: {}",
            status,
            error_body
        );
    }

    // Parse the JSON response
    let resp: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse OpenAI response as JSON")?;

    // Navigate the response tree to get the actual text.
    // choices[0].message.content is where OpenAI puts it.
    let text = resp["choices"][0]["message"]["content"]
        .as_str()
        .context("OpenAI response missing choices[0].message.content")?
        .to_string();

    // Token counts come from the "usage" object
    let usage = &resp["usage"];
    let input_tokens  = usage["prompt_tokens"].as_u64().unwrap_or(0) as u32;
    let output_tokens = usage["completion_tokens"].as_u64().unwrap_or(0) as u32;

    Ok(ProviderResponse {
        model:         model.to_string(),
        text,
        input_tokens,
        output_tokens,
        duration_ms:   start.elapsed().as_millis() as u64,
    })
}
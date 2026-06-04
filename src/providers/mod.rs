// src/providers/mod.rs
//
// This module is the "contract" all provider adapters must follow.
// Every provider (OpenAI, Anthropic, Ollama, Gemini) returns a
// ProviderResponse. The router and display layer never care which
// provider answered — they just work with this uniform shape.

//pub mod openai;
//pub mod anthropic;
pub mod ollama; // Added Week 8 — Ollama local models

// ── Shared response shape ─────────────────────────────────────────────────────

/// The uniform response every provider adapter returns.
/// Adding a new provider never changes downstream code — it just
/// returns this same struct.
#[derive(Debug, Clone)]
pub struct ProviderResponse {
    /// Which model answered, e.g. "gpt-4o-mini", "claude-haiku-4-5"
    pub model: String,
    /// The text response from the model
    pub text: String,
    /// Tokens in the prompt you sent
    pub input_tokens: u32,
    /// Tokens in the model's reply
    pub output_tokens: u32,
    /// How long the API call took (wall clock, milliseconds)
    pub duration_ms: u64,
}

impl ProviderResponse {
    /// Calculate the estimated USD cost for this response.
    ///
    /// Prices as of mid-2025. Verify at:
    ///   OpenAI    → platform.openai.com/pricing
    ///   Anthropic → anthropic.com/pricing
    ///
    /// Formula: (input_tokens / 1_000_000) * price_per_M_input
    ///        + (output_tokens / 1_000_000) * price_per_M_output
    pub fn cost_usd(&self) -> f64 {
        // Input/output costs per 1 million tokens
        let (cost_in, cost_out) = match self.model.as_str() {
            // ── FREE ─────────────────────────────────────────────────────────
            m if m.starts_with("ollama/") => (0.0, 0.0),

            // ── CHEAP tier (~$0.001 per typical call) ────────────────────────
            "gpt-4o-mini"              => (0.15,  0.60),   // $0.15 / $0.60 per 1M
            "claude-haiku-4-5"         => (0.80,  4.00),   // $0.80 / $4.00 per 1M
            "claude-haiku-4-5-20251001"=> (0.80,  4.00),
            "gemini-1.5-flash"         => (0.075, 0.30),   // $0.075 / $0.30 per 1M

            // ── MID tier (~$0.01 per typical call) ───────────────────────────
            "gpt-4o"                   => (2.50,  10.00),  // $2.50 / $10.00 per 1M
            "claude-sonnet-4-6"        => (3.00,  15.00),  // $3.00 / $15.00 per 1M

            // ── PREMIUM tier (~$0.05+ per typical call) ──────────────────────
            "claude-opus-4-6"          => (15.00, 75.00),  // $15 / $75 per 1M

            // Unknown model — assume free rather than mislead
            _ => {
    eprintln!("Warning: unknown model pricing for {}", self.model);
    (0.0, 0.0)
}};

        let input_cost  = (self.input_tokens  as f64 / 1_000_000.0) * cost_in;
        let output_cost = (self.output_tokens as f64 / 1_000_000.0) * cost_out;
        input_cost + output_cost
    }

    /// Total tokens used (input + output)
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }

    pub fn estimated_energy_cost_usd(&self) -> f64 {
        let watts = 45.0;
        let electricity_per_kwh = 0.10;

        let hours =
            self.duration_ms as f64 / 1000.0 / 3600.0;

        (watts / 1000.0) * hours * electricity_per_kwh
    }

    pub fn tokens_per_second(&self) -> f64 {
        if self.duration_ms == 0 {
            return 0.0;
        }

        self.output_tokens as f64
            / (self.duration_ms as f64 / 1000.0)
    }

    
}
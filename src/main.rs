mod providers;
mod output;

use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Ask {
        prompt: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    output::show_banner_once();

    let cli = Cli::parse();

    match cli.command {
        Commands::Ask { prompt } => {
            let spinner = output::Spinner::start("Thinking...");

            let response = providers::ollama::ask(&prompt, "qwen2.5:7b").await?;

            spinner.stop();

            println!(
                "\n⚒️ {} • {:.2}s • {}→{} • {} tok • {:.1} tok/s • ${:.8}\n",
                response.model.replace("ollama/", ""),
                response.duration_ms as f64 / 1000.0,
                response.input_tokens,
                response.output_tokens,
                response.total_tokens(),
                response.tokens_per_second(),
                response.estimated_energy_cost_usd()
            );

            output::print_response(&response.text);
        }
    }

    Ok(())
}
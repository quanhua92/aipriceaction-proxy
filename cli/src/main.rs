use aipriceaction::{
    analysis::ask_ai::handle_ask_ai_request,
    utils::init_logger,
};

// Internal imports for CLI functionality
use aipriceaction::state_machine::ClientDataStateMachine;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aipriceaction")]
#[command(about = "A CLI for Vietnamese stock market analysis with vectorized money flow calculations")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the state machine to fetch and process stock data
    Run {
        /// Number of ticks to run (0 for infinite)
        #[arg(short, long, default_value_t = 0)]
        ticks: usize,
    },
    /// Generate AI analysis prompt for ticker(s)
    Ask {
        /// Ticker symbol(s) to analyze (comma-separated)
        #[arg(short, long)]
        tickers: String,
        /// Template ID for the analysis
        #[arg(short = 'p', long)]
        template_id: String,
        /// Language for the analysis (en/vn)
        #[arg(short, long, default_value = "en")]
        language: String,
        /// Chart context days
        #[arg(long, default_value_t = 10)]
        chart_days: usize,
        /// Money flow context days
        #[arg(long, default_value_t = 10)]
        money_flow_days: usize,
        /// MA Score context days
        #[arg(long, default_value_t = 10)]
        ma_score_days: usize,
        /// MA period (10, 20, or 50)
        #[arg(long, default_value_t = 20)]
        ma_period: u32,
        /// Context date (YYYY-MM-DD) for historical analysis
        #[arg(long)]
        context_date: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    init_logger()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { ticks } => {
            let now = chrono::Utc::now();
            println!("[{}] 🚀 Starting AI Price Action CLI with vectorized money flow calculations",
                now.format("%Y-%m-%d %H:%M:%S UTC"));

            // Create and run state machine
            let mut state_machine = ClientDataStateMachine::new();

            if ticks == 0 {
                let start_time = chrono::Utc::now();
                println!("[{}] 📊 Running indefinitely... Press Ctrl+C to stop",
                    start_time.format("%Y-%m-%d %H:%M:%S UTC"));
                state_machine.start().await?;
            } else {
                let start_time = chrono::Utc::now();
                println!("[{}] 📊 Running for {} ticks... (limited tick mode not yet implemented, running indefinitely)",
                    start_time.format("%Y-%m-%d %H:%M:%S UTC"), ticks);
                state_machine.start().await?;
            }

            let complete_time = chrono::Utc::now();
            println!("[{}] ✅ State machine completed",
                complete_time.format("%Y-%m-%d %H:%M:%S UTC"));
        },
        Commands::Ask {
            tickers,
            template_id,
            language,
            chart_days,
            money_flow_days,
            ma_score_days,
            ma_period,
            context_date,
        } => {
            handle_ask_ai_request(
                tickers,
                template_id,
                language,
                chart_days,
                money_flow_days,
                ma_score_days,
                ma_period,
                context_date,
            ).await?;
        }
    }

    Ok(())
}
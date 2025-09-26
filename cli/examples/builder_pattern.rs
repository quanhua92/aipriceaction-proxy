//! Builder Pattern Example
//!
//! Demonstrates how to use the builder pattern for configuring complex analysis operations.

use aipriceaction::prelude::*;
use aipriceaction::api::{AnalysisBuilder, StockAnalyzer};
use aipriceaction::ask_ai::{Language, MAPeriod};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    aipriceaction::init_logger()?;

    println!("🏗️  Builder Pattern Example - Flexible Configuration");
    println!("{}", "=".repeat(60));

    // Example 1: Using the builder pattern for banking stocks analysis
    println!("\n🏦 Example 1: Banking Stocks with Builder Pattern");

    let analyzer = AnalysisBuilder::banking_stocks()
        .medium_term() // 3 months analysis
        .with_ma_period(MAPeriod::MA20)
        .with_language(Language::English)
        .build();

    println!("✅ Created analyzer for banking stocks with medium-term configuration");

    // Example 2: Custom configuration for securities stocks
    println!("\n📊 Example 2: Custom Securities Configuration");

    let securities_builder = AnalysisBuilder::securities_stocks()
        .short_term() // 1 month analysis
        .with_days_back(30)
        .with_ma_period(MAPeriod::MA10)
        .with_chart_context_days(15)
        .with_money_flow_context_days(10);

    // Build different configurations from the same builder
    let ma_config = securities_builder.clone().build_ma_score_config();
    let money_flow_config = securities_builder.clone().build_money_flow_config();

    println!("✅ Built MA Score config: MA{} period, {} days back", ma_config.default_ma_period, ma_config.days_back);
    println!("✅ Built Money Flow config: {} days back, vectorization enabled: {}",
             money_flow_config.days_back, money_flow_config.enable_vectorization);

    // Example 3: Real estate stocks with long-term analysis
    println!("\n🏢 Example 3: Real Estate Long-term Analysis");

    let real_estate_analyzer = AnalysisBuilder::real_estate_stocks()
        .long_term() // 1 year analysis
        .with_ma_period(MAPeriod::MA50)
        .with_language(Language::Vietnamese)
        .add_ticker("BCM".to_string()) // Add additional ticker
        .build();

    println!("✅ Created real estate analyzer with long-term configuration");

    // Example 4: AI Prompt configuration
    println!("\n🤖 Example 4: AI Prompt Configuration");

    let ai_config = AnalysisBuilder::new()
        .with_tickers(vec!["VCB".to_string(), "FPT".to_string()])
        .with_language(Language::Vietnamese)
        .with_chart_context_days(14)
        .with_ma_period(MAPeriod::MA20)
        .build_ai_prompt_config("should-hold-sell-buy-more".to_string());

    println!("✅ AI Config created:");
    println!("   Template: {}", ai_config.template_id);
    println!("   Language: {:?}", ai_config.language);
    println!("   Chart context days: {}", ai_config.chart_context_days);
    println!("   MA period: {:?}", ai_config.ma_period);

    // Example 5: Quick sector analysis
    println!("\n⚡ Example 5: Quick Sector Comparisons");

    let sectors = [
        ("Banking", AnalysisBuilder::banking_stocks()),
        ("Securities", AnalysisBuilder::securities_stocks()),
        ("Real Estate", AnalysisBuilder::real_estate_stocks()),
    ];

    for (sector_name, builder) in sectors {
        let tickers = builder.tickers();
        println!("📈 {} sector: {} stocks ({})", sector_name, tickers.len(), tickers.join(", "));
    }

    println!("\n🎉 Builder pattern examples completed!");
    println!("💡 The builder pattern provides a flexible way to configure analysis operations");
    println!("   with sensible defaults and easy customization options.");
    println!("{}", "=".repeat(60));

    Ok(())
}
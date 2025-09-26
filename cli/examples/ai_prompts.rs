//! AI Prompts Example
//!
//! Demonstrates how to generate AI analysis prompts for stock analysis using
//! the aipriceaction library's template system.

use aipriceaction::prelude::*;
use aipriceaction::api::{AnalysisBuilder, StockAnalyzer};
use aipriceaction::api::analyzer::AIPromptConfig;
use aipriceaction::ask_ai::{Language, MAPeriod};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    aipriceaction::init_logger()?;

    println!("🤖 AI Prompts Example - Generate Analysis Prompts");
    println!("{}", "=".repeat(60));

    let analyzer = StockAnalyzer::new();

    // Example 1: Basic AI prompt generation for Vietnamese banking stocks
    println!("\n📊 Example 1: Banking Stocks Analysis Prompt (English)");

    let banking_tickers = vec!["VCB".to_string(), "BID".to_string(), "CTG".to_string()];

    let config = AIPromptConfig {
        template_id: "should-hold-sell-buy-more".to_string(),
        language: Language::English,
        chart_context_days: 10,
        money_flow_context_days: 10,
        ma_score_context_days: 10,
        ma_period: MAPeriod::MA20,
        context_date: None,
    };

    match analyzer.generate_ai_prompt(banking_tickers, config).await {
        Ok(prompt) => {
            println!("✅ AI prompt generated successfully!");
            println!("📝 Prompt preview: {}", prompt);
        }
        Err(e) => {
            println!("❌ Failed to generate AI prompt: {}", e);
        }
    }

    // Example 2: Vietnamese language prompt for money flow analysis
    println!("\n💰 Example 2: Money Flow Analysis (Vietnamese)");

    let securities_tickers = vec!["SSI".to_string(), "VCI".to_string()];

    let vn_config = AnalysisBuilder::securities_stocks()
        .with_language(Language::Vietnamese)
        .with_ma_period(MAPeriod::MA10)
        .with_money_flow_context_days(14)
        .build_ai_prompt_config("money-flow-analysis".to_string());

    match analyzer.generate_ai_prompt(securities_tickers, vn_config).await {
        Ok(prompt) => {
            println!("✅ Vietnamese AI prompt generated!");
            println!("🇻🇳 Template: Money Flow Analysis in Vietnamese");
        }
        Err(e) => {
            println!("❌ Failed to generate Vietnamese prompt: {}", e);
        }
    }

    // Example 3: Multi-ticker comparison prompt
    println!("\n🔄 Example 3: Multi-ticker Comparison");

    let comparison_tickers = vec![
        "VHM".to_string(), // Real estate
        "VCB".to_string(), // Banking
        "FPT".to_string(), // Technology
    ];

    let comparison_config = AIPromptConfig {
        template_id: "market-leader-comparison".to_string(),
        language: Language::English,
        chart_context_days: 14,
        money_flow_context_days: 14,
        ma_score_context_days: 14,
        ma_period: MAPeriod::MA50, // Long-term analysis
        context_date: None,
    };

    match analyzer.generate_ai_prompt(comparison_tickers, comparison_config).await {
        Ok(prompt) => {
            println!("✅ Multi-ticker comparison prompt generated!");
            println!("📊 Analysis type: Market leader comparison across sectors");
        }
        Err(e) => {
            println!("❌ Failed to generate comparison prompt: {}", e);
        }
    }

    // Example 4: Historical analysis with specific date context
    println!("\n📅 Example 4: Historical Analysis with Date Context");

    let historical_config = AIPromptConfig {
        template_id: "reversal-setup-scanner".to_string(),
        language: Language::English,
        chart_context_days: 20,
        money_flow_context_days: 15,
        ma_score_context_days: 15,
        ma_period: MAPeriod::MA20,
        context_date: Some("2024-01-15".to_string()), // Historical analysis
    };

    let blue_chip_tickers = vec!["VIC".to_string(), "VHM".to_string(), "GAS".to_string()];

    match analyzer.generate_ai_prompt(blue_chip_tickers, historical_config).await {
        Ok(prompt) => {
            println!("✅ Historical analysis prompt generated!");
            println!("📈 Context date: 2024-01-15 (reversal setup analysis)");
        }
        Err(e) => {
            println!("❌ Failed to generate historical prompt: {}", e);
        }
    }

    // Example 5: Sector rotation analysis
    println!("\n🔄 Example 5: Sector Rotation Strategy");

    let sector_rotation_config = AnalysisBuilder::new()
        .with_tickers(vec![
            "VCB".to_string(), // Banking leader
            "SSI".to_string(), // Securities leader
            "VHM".to_string(), // Real estate leader
        ])
        .with_language(Language::Vietnamese)
        .with_ma_period(MAPeriod::MA20)
        .build_ai_prompt_config("sector-rotation-analysis".to_string());

    let rotation_tickers = vec!["VCB".to_string(), "SSI".to_string(), "VHM".to_string()];

    match analyzer.generate_ai_prompt(rotation_tickers, sector_rotation_config).await {
        Ok(prompt) => {
            println!("✅ Sector rotation analysis prompt generated!");
            println!("🎯 Strategy: Cross-sector momentum analysis");
        }
        Err(e) => {
            println!("❌ Failed to generate sector rotation prompt: {}", e);
        }
    }

    println!("\n🎉 AI prompts examples completed!");
    println!("💡 Available template types:");
    println!("   • should-hold-sell-buy-more - Investment decision analysis");
    println!("   • money-flow-analysis - Smart money vs retail analysis");
    println!("   • market-leader-comparison - Multi-ticker comparison");
    println!("   • sector-rotation-analysis - Cross-sector momentum");
    println!("   • reversal-setup-scanner - Technical reversal patterns");
    println!("{}", "=".repeat(60));

    Ok(())
}
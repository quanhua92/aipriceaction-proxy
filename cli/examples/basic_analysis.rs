//! Basic Analysis Example
//!
//! Demonstrates how to use the aipriceaction library to perform basic money flow
//! and MA score analysis on Vietnamese stocks.

use aipriceaction::prelude::*;
use aipriceaction::api::StockAnalyzer;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    aipriceaction::init_logger()?;

    println!("🚀 Basic Analysis Example - Vietnamese Stock Market");
    println!("{}", "=".repeat(60));

    // Create a stock analyzer
    let analyzer = StockAnalyzer::new();

    // Example 1: Analyze money flow for major Vietnamese banking stocks
    println!("\n📊 Example 1: Money Flow Analysis");
    println!("Analyzing Vietnamese banking stocks: VCB, BID, CTG, TCB");

    let banking_tickers = vec![
        "VCB".to_string(),
        "BID".to_string(),
        "CTG".to_string(),
        "TCB".to_string(),
    ];

    let date_range = DateRangeConfig::default_3m(); // Last 3 months

    match analyzer.analyze_money_flow(banking_tickers.clone(), date_range.clone()).await {
        Ok(result) => {
            println!("✅ Money flow analysis completed successfully!");
            println!("   Results: {} date ranges processed", result.results.len());
            println!("   Performance: {:.2}ms calculation time", result.performance_metrics.vectorized_time);
            println!("   Total calculations: {}", result.performance_metrics.calculation_count);
        }
        Err(e) => {
            println!("❌ Money flow analysis failed: {}", e);
        }
    }

    // Example 2: Analyze MA scores for the same stocks
    println!("\n📈 Example 2: MA Score Analysis");

    let ma_config = aipriceaction::models::ma_score::MAScoreProcessConfig {
        date_range_config: date_range,
        days_back: 90,
        current_date: None,
        default_ma_period: 20, // MA20
    };

    match analyzer.analyze_ma_score(banking_tickers, ma_config).await {
        Ok(result) => {
            println!("✅ MA score analysis completed successfully!");
            println!("   Results: {} date ranges processed", result.results.len());
            println!("   MA Period: MA{}", result.performance_metrics.ma_period);
            println!("   Performance: {:.2}ms calculation time", result.performance_metrics.calculation_time);
        }
        Err(e) => {
            println!("❌ MA score analysis failed: {}", e);
        }
    }

    // Example 3: Cache information
    println!("\n💾 Example 3: Cache Information");
    let cache = analyzer.cache();
    println!("Cache initialized: {}", cache.is_initialized());
    println!("Available ticker count: {}", cache.get_ticker_count());

    println!("\n🎉 Basic analysis example completed!");
    println!("{}", "=".repeat(60));

    Ok(())
}
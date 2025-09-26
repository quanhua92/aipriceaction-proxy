//! High-level stock analyzer for easy library usage

use std::collections::HashMap;
use crate::prelude::*;
use crate::ask_ai::{handle_ask_ai_request, Language, MAPeriod};
use crate::utils::vectorized_money_flow::calculate_multiple_dates_vectorized;
use crate::utils::vectorized_ma_score::calculate_multiple_dates_vectorized_ma_score;
use crate::models::ma_score::MAScoreProcessConfig;
use crate::services::CacheManager;

/// High-level interface for stock market analysis
///
/// Provides simple methods for common analysis tasks without requiring
/// deep knowledge of the internal implementation.
pub struct StockAnalyzer {
    cache: CacheManager,
}

/// Result of money flow analysis
#[derive(Debug, Clone)]
pub struct MoneyFlowAnalysisResult {
    pub results: HashMap<String, Vec<MoneyFlowTickerData>>,
    pub performance_metrics: crate::utils::money_flow_utils::PerformanceMetrics,
}

/// Result of MA score analysis
#[derive(Debug, Clone)]
pub struct MAScoreAnalysisResult {
    pub results: HashMap<String, Vec<MAScoreTickerData>>,
    pub performance_metrics: crate::models::ma_score::MAScorePerformanceMetrics,
}

/// Configuration for AI prompt generation
#[derive(Debug, Clone)]
pub struct AIPromptConfig {
    pub template_id: String,
    pub language: Language,
    pub chart_context_days: usize,
    pub money_flow_context_days: usize,
    pub ma_score_context_days: usize,
    pub ma_period: MAPeriod,
    pub context_date: Option<String>,
}

impl Default for AIPromptConfig {
    fn default() -> Self {
        Self {
            template_id: "should-hold-sell-buy-more".to_string(),
            language: Language::English,
            chart_context_days: 10,
            money_flow_context_days: 10,
            ma_score_context_days: 10,
            ma_period: MAPeriod::MA20,
            context_date: None,
        }
    }
}

impl StockAnalyzer {
    /// Create a new stock analyzer instance
    pub fn new() -> Self {
        Self {
            cache: CacheManager::new(),
        }
    }

    /// Analyze money flow for given tickers
    ///
    /// # Arguments
    /// * `tickers` - Vector of ticker symbols (e.g., vec!["VCB", "BID"])
    /// * `date_range_config` - Date range configuration for analysis
    ///
    /// # Returns
    /// MoneyFlowAnalysisResult containing calculated money flow data and performance metrics
    pub async fn analyze_money_flow(
        &self,
        tickers: Vec<String>,
        _date_range_config: DateRangeConfig,
    ) -> Result<MoneyFlowAnalysisResult, Box<dyn std::error::Error>> {
        // In a real implementation, we would:
        // 1. Load data from cache or fetch if needed
        // 2. Run vectorized money flow calculations
        // 3. Return results

        // For now, create a basic implementation that would work with real data
        let config = crate::utils::money_flow_utils::VectorizedMoneyFlowConfig::default();

        // This would normally get data from cache/services
        let ticker_data = HashMap::new(); // Placeholder - would load real data
        let date_range = vec![]; // Placeholder - would calculate real date range

        let result = calculate_multiple_dates_vectorized(
            &ticker_data,
            &tickers,
            &date_range,
            None, // vnindex_data
            config.vnindex_volume_weighting,
            config.directional_colors,
        );

        Ok(MoneyFlowAnalysisResult {
            results: result.results,
            performance_metrics: result.metrics,
        })
    }

    /// Analyze MA scores for given tickers
    ///
    /// # Arguments
    /// * `tickers` - Vector of ticker symbols
    /// * `config` - MA score process configuration
    ///
    /// # Returns
    /// MAScoreAnalysisResult containing calculated MA score data and performance metrics
    pub async fn analyze_ma_score(
        &self,
        tickers: Vec<String>,
        config: MAScoreProcessConfig,
    ) -> Result<MAScoreAnalysisResult, Box<dyn std::error::Error>> {
        // Similar to money flow, this would load real data and calculate MA scores
        let ticker_data = HashMap::new(); // Placeholder
        let date_range = vec![]; // Placeholder

        let (results, metrics) = calculate_multiple_dates_vectorized_ma_score(
            &ticker_data,
            &tickers,
            &date_range,
            &config,
        );

        Ok(MAScoreAnalysisResult {
            results,
            performance_metrics: metrics,
        })
    }

    /// Generate AI analysis prompt
    ///
    /// # Arguments
    /// * `tickers` - Vector of ticker symbols to analyze
    /// * `config` - AI prompt configuration
    ///
    /// # Returns
    /// Generated AI prompt as a string
    pub async fn generate_ai_prompt(
        &self,
        tickers: Vec<String>,
        config: AIPromptConfig,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Capture the output by redirecting stdout
        

        let tickers_string = tickers.join(",");
        let language_string = match config.language {
            Language::English => "en".to_string(),
            Language::Vietnamese => "vn".to_string(),
        };

        // This calls the existing handler but we would need to modify it to return the string
        // instead of printing to stdout. For now, this is a placeholder.
        handle_ask_ai_request(
            tickers_string,
            config.template_id,
            language_string,
            config.chart_context_days,
            config.money_flow_context_days,
            config.ma_score_context_days,
            config.ma_period as u32,
            config.context_date,
        ).await?;

        Ok("AI prompt generated successfully - check stdout".to_string())
    }

    /// Get cache manager instance for advanced operations
    pub fn cache(&self) -> &CacheManager {
        &self.cache
    }
}

impl Default for StockAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
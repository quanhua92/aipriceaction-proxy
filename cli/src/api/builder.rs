//! Builder pattern for configuring stock analysis operations

use crate::prelude::*;
use crate::ask_ai::{Language, MAPeriod};
use crate::api::analyzer::{StockAnalyzer, AIPromptConfig};
use crate::models::ma_score::MAScoreProcessConfig;
use crate::utils::money_flow_utils::VectorizedMoneyFlowConfig;

/// Builder for configuring stock analysis operations
///
/// Provides a fluent interface for setting up analysis parameters.
///
/// # Example
/// ```rust
/// use aipriceaction::api::AnalysisBuilder;
/// use aipriceaction::prelude::*;
///
/// let analyzer = AnalysisBuilder::new()
///     .with_tickers(vec!["VCB".to_string(), "BID".to_string()])
///     .with_date_range(DateRangeConfig::default_3m())
///     .with_days_back(60)
///     .build();
/// ```
pub struct AnalysisBuilder {
    tickers: Vec<String>,
    date_range_config: Option<DateRangeConfig>,
    days_back: usize,
    current_date: Option<String>,
    ma_period: MAPeriod,
    language: Language,
    chart_context_days: usize,
    money_flow_context_days: usize,
    ma_score_context_days: usize,
}

impl AnalysisBuilder {
    /// Create a new analysis builder with default settings
    pub fn new() -> Self {
        Self {
            tickers: Vec::new(),
            date_range_config: None,
            days_back: 60,
            current_date: None,
            ma_period: MAPeriod::MA20,
            language: Language::English,
            chart_context_days: 10,
            money_flow_context_days: 10,
            ma_score_context_days: 10,
        }
    }

    /// Set the tickers to analyze
    pub fn with_tickers(mut self, tickers: Vec<String>) -> Self {
        self.tickers = tickers;
        self
    }

    /// Add a single ticker to the analysis
    pub fn add_ticker(mut self, ticker: String) -> Self {
        self.tickers.push(ticker);
        self
    }

    /// Set the date range configuration
    pub fn with_date_range(mut self, config: DateRangeConfig) -> Self {
        self.date_range_config = Some(config);
        self
    }

    /// Set how many days back to analyze
    pub fn with_days_back(mut self, days: usize) -> Self {
        self.days_back = days;
        self
    }

    /// Set the current date for analysis (for historical analysis)
    pub fn with_current_date(mut self, date: String) -> Self {
        self.current_date = Some(date);
        self
    }

    /// Set the MA period for analysis
    pub fn with_ma_period(mut self, period: MAPeriod) -> Self {
        self.ma_period = period;
        self
    }

    /// Set the language for AI prompts
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// Set chart context days for AI analysis
    pub fn with_chart_context_days(mut self, days: usize) -> Self {
        self.chart_context_days = days;
        self
    }

    /// Set money flow context days for AI analysis
    pub fn with_money_flow_context_days(mut self, days: usize) -> Self {
        self.money_flow_context_days = days;
        self
    }

    /// Set MA score context days for AI analysis
    pub fn with_ma_score_context_days(mut self, days: usize) -> Self {
        self.ma_score_context_days = days;
        self
    }

    /// Build a StockAnalyzer with the configured settings
    pub fn build(self) -> StockAnalyzer {
        StockAnalyzer::new()
    }

    /// Build money flow process configuration
    pub fn build_money_flow_config(self) -> VectorizedMoneyFlowConfig {
        VectorizedMoneyFlowConfig {
            days_back: self.days_back,
            current_date: self.current_date,
            vnindex_volume_weighting: true,
            directional_colors: false,
            enable_vectorization: true,
        }
    }

    /// Build MA score process configuration
    pub fn build_ma_score_config(self) -> MAScoreProcessConfig {
        MAScoreProcessConfig {
            date_range_config: self.date_range_config.unwrap_or_else(DateRangeConfig::default_3m),
            days_back: self.days_back,
            current_date: self.current_date,
            default_ma_period: self.ma_period as i32,
        }
    }

    /// Build AI prompt configuration
    pub fn build_ai_prompt_config(self, template_id: String) -> AIPromptConfig {
        AIPromptConfig {
            template_id,
            language: self.language,
            chart_context_days: self.chart_context_days,
            money_flow_context_days: self.money_flow_context_days,
            ma_score_context_days: self.ma_score_context_days,
            ma_period: self.ma_period,
            context_date: self.current_date,
        }
    }

    /// Get the configured tickers
    pub fn tickers(&self) -> &[String] {
        &self.tickers
    }
}

impl Default for AnalysisBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick builder functions for common configurations
impl AnalysisBuilder {
    /// Create builder for Vietnamese banking stocks
    pub fn banking_stocks() -> Self {
        Self::new().with_tickers(vec![
            "VCB".to_string(),
            "BID".to_string(),
            "CTG".to_string(),
            "TCB".to_string(),
            "MBB".to_string(),
        ])
    }

    /// Create builder for Vietnamese securities stocks
    pub fn securities_stocks() -> Self {
        Self::new().with_tickers(vec![
            "SSI".to_string(),
            "VCI".to_string(),
            "VCS".to_string(),
            "SHS".to_string(),
            "MBS".to_string(),
        ])
    }

    /// Create builder for Vietnamese real estate stocks
    pub fn real_estate_stocks() -> Self {
        Self::new().with_tickers(vec![
            "VHM".to_string(),
            "VIC".to_string(),
            "VRE".to_string(),
            "NVL".to_string(),
            "KDH".to_string(),
        ])
    }

    /// Create builder configured for short-term analysis (1 month)
    pub fn short_term(self) -> Self {
        self.with_date_range(DateRangeConfig::default_1m())
            .with_days_back(30)
    }

    /// Create builder configured for medium-term analysis (3 months)
    pub fn medium_term(self) -> Self {
        self.with_date_range(DateRangeConfig::default_3m())
            .with_days_back(90)
    }

    /// Create builder configured for long-term analysis (1 year)
    pub fn long_term(self) -> Self {
        self.with_date_range(DateRangeConfig::default_1y())
            .with_days_back(365)
    }
}
use serde::{Deserialize, Serialize};
use crate::models::StockDataPoint;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskAITemplate {
    pub id: String,
    pub title: String,
    pub prompt: String,
}

#[derive(Debug, Clone)]
pub struct TickerContextData {
    pub ticker: String,
    pub chart_data: Vec<StockDataPoint>,
    pub vpa_content: Option<String>,
    pub ticker_ai_data: Option<TickerAIData>,
}

#[derive(Debug, Clone)]
pub struct TickerAIData {
    pub company_name: Option<String>,
    pub market_cap: Option<f64>,
    pub pe_ratio: Option<f64>,
    pub pb_ratio: Option<f64>,
    pub roe: Option<f64>,
    pub roa: Option<f64>,
    pub debt_to_equity: Option<f64>,
    pub current_ratio: Option<f64>,
    pub revenue_growth: Option<f64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ContextConfig {
    pub chart_context_days: usize,
    pub vpa_context_days: usize,
    pub money_flow_context_days: usize,
    pub ma_score_context_days: usize,
    pub include_basic_info: bool,
    pub include_financial_ratios: bool,
    pub include_description: bool,
    pub ma_period: MAPeriod,
    pub context_date: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum MAPeriod {
    MA10 = 10,
    MA20 = 20,
    MA50 = 50,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            chart_context_days: 10,
            vpa_context_days: 5,
            money_flow_context_days: 10,
            ma_score_context_days: 10,
            include_basic_info: true,
            include_financial_ratios: true,
            include_description: true,
            ma_period: MAPeriod::MA20,
            context_date: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Language {
    English,
    Vietnamese,
}

#[derive(Debug, Clone)]
pub struct AskAIRequest {
    pub tickers: Vec<String>,
    pub template_id: String,
    pub language: Language,
    pub config: ContextConfig,
}
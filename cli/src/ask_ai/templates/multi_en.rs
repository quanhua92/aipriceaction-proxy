use crate::ask_ai::types::AskAITemplate;

pub fn get_multi_ticker_templates_en() -> Vec<AskAITemplate> {
    vec![
        AskAITemplate {
            id: "portfolio-optimization".to_string(),
            title: "🎯 Portfolio Optimization Strategy".to_string(),
            prompt: "Optimize my portfolio with these selected tickers: (1) Correlation analysis and diversification benefits, (2) Optimal position sizing for each ticker, (3) Risk-adjusted return optimization, (4) Sector allocation balance, (5) Entry/exit timing coordination, (6) Hedging strategies between positions. Provide specific allocation percentages and rebalancing rules.".to_string(),
        },
        AskAITemplate {
            id: "comparative-analysis".to_string(),
            title: "⚖️ Comparative Analysis - Which to Buy?".to_string(),
            prompt: "Compare these tickers across all dimensions: (1) Technical strength and momentum, (2) Money flow and institutional preference, (3) Fundamental valuation and growth, (4) Risk-reward profiles, (5) Sector positioning and rotation, (6) Catalyst timing and potential. Rank them by investment attractiveness with specific reasons.".to_string(),
        },
        AskAITemplate {
            id: "sector-rotation-play".to_string(),
            title: "🔄 Sector Rotation Strategy".to_string(),
            prompt: "Design a sector rotation strategy using these tickers: (1) Current sector cycle position analysis, (2) Leading vs lagging sector identification, (3) Rotation timing signals and triggers, (4) Cross-sector correlation patterns, (5) Economic cycle positioning, (6) Optimal rotation sequence. Time the sector switches for maximum alpha.".to_string(),
        },
        AskAITemplate {
            id: "pairs-trading".to_string(),
            title: "↔️ Pairs Trading Opportunities".to_string(),
            prompt: "Identify pairs trading opportunities between these tickers: (1) Historical correlation analysis, (2) Mean reversion patterns, (3) Spread analysis and fair value, (4) Momentum divergence signals, (5) Risk management for pair trades, (6) Optimal entry/exit timing. Find profitable relative value opportunities.".to_string(),
        },
        AskAITemplate {
            id: "risk-diversification".to_string(),
            title: "🛡️ Risk Diversification Analysis".to_string(),
            prompt: "Analyze risk diversification across these tickers: (1) Correlation matrix and clustering, (2) Sector and style diversification, (3) Volatility contribution analysis, (4) Tail risk assessment, (5) Concentration risk evaluation, (6) Hedge ratio optimization. Build a truly diversified portfolio.".to_string(),
        },
        AskAITemplate {
            id: "momentum-basket".to_string(),
            title: "🚀 Momentum Basket Strategy".to_string(),
            prompt: "Create a momentum basket strategy: (1) Momentum ranking and scoring, (2) Rotation rules within the basket, (3) Adding/removing criteria, (4) Position sizing by momentum strength, (5) Risk management for momentum strategies, (6) Momentum decay detection. Ride the strongest trends together.".to_string(),
        },
    ]
}
use crate::ask_ai::types::AskAITemplate;

pub fn get_money_flow_templates_en() -> Vec<AskAITemplate> {
    vec![
        AskAITemplate {
            id: "market-money-flow-analysis".to_string(),
            title: "🌊 Complete Market Money Flow Analysis".to_string(),
            prompt: "Analyze the complete market money flow patterns: (1) Sector rotation based on money flow rankings, (2) Smart money vs retail sentiment divergence, (3) Institutional accumulation/distribution patterns, (4) Cross-sector money flow correlations, (5) Market leadership changes and implications, (6) Money flow divergence with price action. Identify where smart money is positioning for maximum profits.".to_string(),
        },
        AskAITemplate {
            id: "sector-money-flow".to_string(),
            title: "🏭 Sector Money Flow & Rotation Strategy".to_string(),
            prompt: "Design a sector rotation strategy based on money flow: (1) Current sector money flow rankings and trends, (2) Leading vs lagging sectors identification, (3) Money flow momentum and acceleration, (4) Sector correlation with market leaders, (5) Economic cycle positioning through money flow, (6) Optimal sector switching signals. Time sector rotations using smart money movements.".to_string(),
        },
        AskAITemplate {
            id: "smart-money-tracking".to_string(),
            title: "🧠 Smart Money Tracking & Following".to_string(),
            prompt: "Track and follow smart money movements: (1) Identify stocks with strongest institutional buying, (2) Money flow vs price divergence analysis, (3) Volume profile and smart money entry points, (4) Accumulation phase identification, (5) Distribution warning signals, (6) Smart money exit strategies. Follow the institutions for superior returns.".to_string(),
        },
        AskAITemplate {
            id: "money-flow-divergence".to_string(),
            title: "📈 Money Flow Divergence Analysis".to_string(),
            prompt: "Analyze money flow divergences for opportunities: (1) Price vs money flow divergence identification, (2) Bullish and bearish divergence patterns, (3) Hidden divergences and continuation signals, (4) Divergence strength and reliability, (5) Time horizon for divergence resolution, (6) Trading strategies for each divergence type. Exploit market inefficiencies.".to_string(),
        },
        AskAITemplate {
            id: "market-leaders-analysis".to_string(),
            title: "👑 Market Leaders Money Flow Analysis".to_string(),
            prompt: "Analyze market leaders through money flow lens: (1) Current market leadership based on money flow, (2) Leadership rotation patterns and timing, (3) Money flow quality vs quantity analysis, (4) Sustainable vs temporary leadership, (5) Correlation between leaders and market direction, (6) Next potential market leaders identification. Identify and ride the strongest trends.".to_string(),
        },
        AskAITemplate {
            id: "money-flow-momentum".to_string(),
            title: "⚡ Money Flow Momentum Strategy".to_string(),
            prompt: "Develop a money flow momentum strategy: (1) Money flow acceleration and deceleration signals, (2) Momentum ranking and scoring system, (3) Entry and exit rules based on flow momentum, (4) Risk management for momentum strategies, (5) Momentum divergence warning signals, (6) Portfolio construction using flow momentum. Build a systematic momentum approach.".to_string(),
        },
        AskAITemplate {
            id: "institutional-vs-retail".to_string(),
            title: "🏛️ Institutional vs Retail Money Flow".to_string(),
            prompt: "Analyze institutional vs retail money flow patterns: (1) Smart money vs retail sentiment divergence, (2) Institutional accumulation during retail panic, (3) Distribution during retail euphoria, (4) Volume analysis for institutional vs retail activity, (5) Contrarian opportunities from flow divergence, (6) Market timing using institutional flows. Position with the smart money.".to_string(),
        },
    ]
}
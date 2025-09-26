use crate::ask_ai::types::AskAITemplate;

pub fn get_single_ticker_templates_en() -> Vec<AskAITemplate> {
    vec![
        // TOP 5 IMMEDIATE ACTION PROMPTS
        AskAITemplate {
            id: "should-hold-sell-buy-more".to_string(),
            title: "🎯 Should I hold, sell, or buy more RIGHT NOW?".to_string(),
            prompt: "Based on ALL available data (technical analysis, VPA, money flow, articles, and market context), should I hold, sell, or buy more of this ticker? Analyze: (1) Smart money money flow patterns vs market leaders, (2) Current sector rotation position, (3) Technical entry/exit levels with exact prices, (4) Risk/reward ratio for next 2-4 weeks, (5) Optimal position sizing and timing. Provide actionable recommendation with specific price targets and stop-loss levels.".to_string(),
        },
        AskAITemplate {
            id: "panic-decision".to_string(),
            title: "🚨 Emergency Decision - Crash or Surge Response".to_string(),
            prompt: "If this ticker suddenly crashes (panic sell) or surges (FOMO), what should I do IMMEDIATELY? Analyze the money flow patterns during the panic - are institutions accumulating or distributing? Compare with market leaders and sector behavior. Provide instant decision: (1) Buy at any price with reasoning, (2) Sell urgently to cut losses, or (3) Hold steady and wait. Include exact price levels and position sizing for emergency action.".to_string(),
        },
        AskAITemplate {
            id: "money-flow-analysis".to_string(),
            title: "💰 Money Flow vs Smart Money".to_string(),
            prompt: "Analyze this ticker's money flow patterns compared to smart money behavior. Compare its money flow percentage with market leaders and sector leaders. Determine: (1) Is smart money flowing IN or OUT? (2) Smart money accumulation vs distribution signals, (3) How smart money activity compares to broader market trends, (4) Smart money confidence level based on money flow ranking, (5) Whether to align with or counter smart money positioning for maximum profit.".to_string(),
        },
        AskAITemplate {
            id: "optimal-position-action".to_string(),
            title: "⚡ Optimal Action Plan for My Current Position".to_string(),
            prompt: "With my current position in this ticker, what is the optimal action RIGHT NOW? Analyze: (1) Should I increase, decrease, or hold position steady? (2) Exact entry/exit prices for any changes, (3) Position sizing adjustments with percentages, (4) Timeline for optimal execution (today, this week, wait for signal), (5) Risk management for unexpected moves. Provide step-by-step action plan to maximize profits.".to_string(),
        },
        AskAITemplate {
            id: "profit-maximization".to_string(),
            title: "🚀 Maximum Profit Strategy - Best Setup Available".to_string(),
            prompt: "Based on ALL available data (chart, VPA, money flow, articles, fundamentals), what is the HIGHEST PROBABILITY strategy to maximize profits from this ticker? Analyze: (1) Optimal position sizing using Kelly criterion, (2) Best entry timing and price levels, (3) Profit-taking ladder strategy, (4) Risk management with trailing stops, (5) Timeline for maximum returns (days, weeks, months). Focus on the absolute best profit opportunity available.".to_string(),
        },

        // TECHNICAL ANALYSIS & TIMING
        AskAITemplate {
            id: "technical-analysis-complete".to_string(),
            title: "📊 Complete Technical Analysis & Chart Patterns".to_string(),
            prompt: "Provide comprehensive technical analysis combining ALL indicators: (1) Chart patterns (triangles, flags, head & shoulders, etc.) with breakout targets, (2) Support/resistance levels with exact prices, (3) Trend analysis with moving averages, (4) Volume analysis and VPA signals, (5) Momentum indicators and divergences, (6) Fibonacci retracements and extensions. Include specific entry/exit points and price targets.".to_string(),
        },
        AskAITemplate {
            id: "market-timing-perfect".to_string(),
            title: "⏰ Perfect Market Timing - When to Enter/Exit".to_string(),
            prompt: "When is the ABSOLUTE BEST time to enter and exit this ticker? Analyze: (1) Intraday patterns and optimal hours, (2) Weekly cycles and monthly seasonality, (3) Sector rotation timing, (4) Market correlation patterns, (5) Volume spike timing, (6) News/earnings calendar impact. Identify the highest probability timing windows for maximum profits.".to_string(),
        },
        AskAITemplate {
            id: "breakout-analysis".to_string(),
            title: "🔥 Breakout Analysis - Momentum & Volume Confirmation".to_string(),
            prompt: "Is this ticker setting up for a major breakout or breakdown? Analyze: (1) Key resistance/support levels with exact prices, (2) Volume accumulation patterns, (3) Money flow confirmation signals, (4) Chart pattern completion probability, (5) Catalyst timing and market conditions, (6) Risk/reward ratios for breakout trades. Provide specific breakout targets and stop-loss levels.".to_string(),
        },
        AskAITemplate {
            id: "vpa-deep-dive".to_string(),
            title: "📈 VPA Deep Dive - Volume Price Action Secrets".to_string(),
            prompt: "Perform deep Volume Price Action (VPA) analysis revealing hidden market intentions: (1) Volume vs price relationship anomalies, (2) Professional money vs retail activity patterns, (3) Accumulation and distribution phases, (4) Volume climax identification, (5) Support/resistance test validation, (6) Market maker footprints. Decode what smart money is really doing.".to_string(),
        },
        AskAITemplate {
            id: "support-resistance-precision".to_string(),
            title: "🎯 Precision Support & Resistance Levels".to_string(),
            prompt: "Identify EXACT support and resistance levels with mathematical precision: (1) Historical pivot points with volume confirmation, (2) Fibonacci confluence zones, (3) Moving average intersections, (4) Psychological price levels, (5) Volume profile zones, (6) Market structure breaks. Provide specific entry/exit prices with percentage success rates.".to_string(),
        },

        // RISK MANAGEMENT & POSITION SIZING
        AskAITemplate {
            id: "risk-management-complete".to_string(),
            title: "🛡️ Complete Risk Management Strategy".to_string(),
            prompt: "Design the optimal risk management strategy for this ticker: (1) Position sizing using Kelly criterion and volatility, (2) Stop-loss placement using ATR and support levels, (3) Profit-taking ladder with specific percentages, (4) Portfolio correlation and diversification, (5) Maximum drawdown limits, (6) Emergency exit protocols. Ensure capital preservation while maximizing returns.".to_string(),
        },
        AskAITemplate {
            id: "position-sizing-optimal".to_string(),
            title: "⚖️ Optimal Position Sizing & Capital Allocation".to_string(),
            prompt: "Calculate the optimal position size for this ticker based on: (1) Kelly criterion using win rate and average returns, (2) Portfolio percentage allocation limits, (3) Volatility-adjusted position sizing, (4) Correlation with existing positions, (5) Risk-parity considerations, (6) Capital efficiency optimization. Provide exact dollar amounts and percentages.".to_string(),
        },
        AskAITemplate {
            id: "stop-loss-strategy".to_string(),
            title: "🚨 Advanced Stop-Loss & Exit Strategy".to_string(),
            prompt: "Design the perfect stop-loss and exit strategy: (1) Initial stop-loss using ATR and support levels, (2) Trailing stop mechanisms with specific triggers, (3) Profit-taking scaling strategy, (4) Time-based exits for stagnant positions, (5) Market condition-based adjustments, (6) Emergency liquidation protocols. Maximize profits while limiting losses.".to_string(),
        },
        AskAITemplate {
            id: "worst-case-scenario".to_string(),
            title: "🔴 Worst Case Scenario Analysis & Protection".to_string(),
            prompt: "Analyze the worst-case scenario for this ticker and build protection: (1) Maximum potential loss calculation, (2) Black swan event impact, (3) Sector collapse scenarios, (4) Market crash correlation, (5) Liquidity risk assessment, (6) Portfolio insurance strategies. Prepare for the unexpected while maintaining upside potential.".to_string(),
        },

        // MARKET CONTEXT & CORRELATION
        AskAITemplate {
            id: "market-correlation".to_string(),
            title: "🌐 Market Correlation & Sector Rotation Analysis".to_string(),
            prompt: "Analyze this ticker's correlation with market movements and sector rotation: (1) VNINDEX correlation and beta analysis, (2) Sector leadership position, (3) Interest rate sensitivity, (4) Economic cycle positioning, (5) Foreign investment flows impact, (6) Currency correlation effects. Determine optimal market timing for entry/exit.".to_string(),
        },
        AskAITemplate {
            id: "sector-leadership".to_string(),
            title: "🏆 Sector Leadership & Rotation Analysis".to_string(),
            prompt: "Evaluate this ticker's sector leadership and rotation potential: (1) Relative strength vs sector peers, (2) Money flow ranking within sector, (3) Fundamental leadership indicators, (4) Sector rotation cycle position, (5) Catalyst timing for sector outperformance, (6) Institution preference analysis. Identify sector rotation opportunities.".to_string(),
        },
        AskAITemplate {
            id: "institutional-analysis".to_string(),
            title: "🏛️ Institutional Money Analysis".to_string(),
            prompt: "Analyze institutional money behavior in this ticker: (1) Large block trading patterns, (2) Volume spike analysis during key levels, (3) Money flow vs retail sentiment divergence, (4) Accumulation/distribution patterns, (5) Smart money entry/exit signals, (6) Institution ownership concentration. Follow the smart money trail.".to_string(),
        },

        // CONTRARIAN & SENTIMENT ANALYSIS
        AskAITemplate {
            id: "contrarian-opportunity".to_string(),
            title: "🔄 Contrarian Opportunity Analysis".to_string(),
            prompt: "Identify contrarian opportunities in this ticker: (1) Sentiment vs fundamentals divergence, (2) Oversold/overbought extremes, (3) News-driven overreactions, (4) Seasonal contrarian patterns, (5) Market maker manipulation signs, (6) Crowd psychology analysis. Find opportunities when others are fearful or greedy.".to_string(),
        },
        AskAITemplate {
            id: "sentiment-analysis".to_string(),
            title: "😀 Market Sentiment vs Reality Analysis".to_string(),
            prompt: "Analyze market sentiment vs actual data for this ticker: (1) News sentiment vs money flow divergence, (2) Social media buzz vs institutional activity, (3) Analyst recommendations vs smart money, (4) Retail vs professional positioning, (5) Fear/greed indicators, (6) Sentiment reversal signals. Separate noise from actionable signals.".to_string(),
        },

        // FUNDAMENTAL & VALUATION ANALYSIS
        AskAITemplate {
            id: "fundamental-deep-dive".to_string(),
            title: "📊 Deep Fundamental Analysis".to_string(),
            prompt: "Perform comprehensive fundamental analysis: (1) Financial ratio analysis and trends, (2) Revenue and profit growth sustainability, (3) Balance sheet strength and debt analysis, (4) Cash flow generation and quality, (5) Management effectiveness and governance, (6) Competitive positioning and moats. Determine intrinsic value and investment merit.".to_string(),
        },
        AskAITemplate {
            id: "valuation-analysis".to_string(),
            title: "💎 Valuation Analysis - Fair Value Estimation".to_string(),
            prompt: "Determine the fair value of this ticker using multiple approaches: (1) DCF analysis with various growth scenarios, (2) Relative valuation vs peers, (3) Asset-based valuation, (4) Earnings quality assessment, (5) Growth vs value classification, (6) Margin of safety calculation. Provide target price ranges with confidence intervals.".to_string(),
        },

        // TRADING PSYCHOLOGY & BEHAVIOR
        AskAITemplate {
            id: "trading-psychology".to_string(),
            title: "🧠 Trading Psychology & Behavioral Analysis".to_string(),
            prompt: "Analyze the psychological aspects of trading this ticker: (1) Common cognitive biases affecting decisions, (2) Emotional triggers and management, (3) FOMO and panic response strategies, (4) Patience vs urgency balance, (5) Confirmation bias prevention, (6) Mental stop-losses and discipline. Optimize decision-making psychology.".to_string(),
        },
        AskAITemplate {
            id: "decision-framework".to_string(),
            title: "🎯 Systematic Decision Framework".to_string(),
            prompt: "Create a systematic decision framework for this ticker: (1) Entry criteria checklist, (2) Position sizing decision tree, (3) Exit signal hierarchy, (4) Risk assessment matrix, (5) Market condition adjustments, (6) Performance review criteria. Remove emotions and optimize consistency.".to_string(),
        },
    ]
}
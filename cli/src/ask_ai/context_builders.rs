use std::collections::HashMap;
use crate::models::StockDataPoint;
use crate::utils::money_flow_utils::MoneyFlowTickerData;
use super::formatters::{format_chart_context, format_vpa_context, format_ma_score_context, format_ticker_ai_context, format_money_flow_context};
use super::types::{TickerContextData, ContextConfig};
use super::utils::find_ticker_sector;

/// Build complete context for single ticker analysis
pub fn build_single_ticker_context(
    ticker_data: &TickerContextData,
    config: &ContextConfig,
) -> String {
    let mut contexts = Vec::new();

    // Company context
    if let Some(ticker_ai_data) = &ticker_data.ticker_ai_data {
        if let Some(ticker_context) = format_ticker_ai_context(
            &ticker_data.ticker,
            Some(ticker_ai_data),
            config.include_basic_info,
            config.include_financial_ratios,
            config.include_description,
        ) {
            contexts.push(format!("# Company Context\n{}", ticker_context));
        }
    }

    // Chart context
    let chart_context = format_chart_context(
        &ticker_data.ticker,
        ticker_data.chart_data.clone(),
        config.chart_context_days,
        config.context_date.as_deref(),
    );
    if !chart_context.is_empty() {
        contexts.push(format!("# Chart Context\n{}", chart_context));
    }

    // VPA context
    let vpa_context = format_vpa_context(
        &ticker_data.ticker,
        ticker_data.vpa_content.as_deref(),
        config.vpa_context_days,
        config.context_date.as_deref(),
    );
    if !vpa_context.is_empty() && !vpa_context.contains("No VPA data available") {
        contexts.push(format!("# Volume Price Action Context\n{}", vpa_context));
    }

    contexts.join("\n\n")
}

/// Build complete context for multiple tickers analysis
pub fn build_multiple_tickers_context(
    tickers_data: &[TickerContextData],
    config: &ContextConfig,
) -> String {
    let contexts: Vec<String> = tickers_data
        .iter()
        .map(|ticker_data| {
            let mut ticker_contexts = Vec::new();

            // Company context
            if let Some(ticker_ai_data) = &ticker_data.ticker_ai_data {
                if let Some(ticker_context) = format_ticker_ai_context(
                    &ticker_data.ticker,
                    Some(ticker_ai_data),
                    config.include_basic_info,
                    config.include_financial_ratios,
                    config.include_description,
                ) {
                    ticker_contexts.push(format!("# Company Context\n{}", ticker_context));
                }
            }

            // Chart context
            let chart_context = format_chart_context(
                &ticker_data.ticker,
                ticker_data.chart_data.clone(),
                config.chart_context_days,
                config.context_date.as_deref(),
            );
            if !chart_context.is_empty() {
                ticker_contexts.push(format!("# Chart Context\n{}", chart_context));
            }

            // VPA context
            let vpa_context = format_vpa_context(
                &ticker_data.ticker,
                ticker_data.vpa_content.as_deref(),
                config.vpa_context_days,
                config.context_date.as_deref(),
            );
            if !vpa_context.is_empty() && !vpa_context.contains("No VPA data available") {
                ticker_contexts.push(format!("# Volume Price Action Context\n{}", vpa_context));
            }

            if ticker_contexts.is_empty() {
                String::new()
            } else {
                format!("## {}\n{}", ticker_data.ticker, ticker_contexts.join("\n\n"))
            }
        })
        .filter(|context| !context.is_empty())
        .collect();

    contexts.join("\n\n---\n\n")
}

/// Build context for interested tickers with chart, money flow, and MA Score data
pub fn build_interested_tickers_context(
    interested_tickers: &[String],
    market_data: &HashMap<String, Vec<StockDataPoint>>,
    money_flow_result: Option<&HashMap<String, MoneyFlowTickerData>>,
    ma_score_data: Option<&HashMap<String, Vec<crate::models::ma_score::MAScoreTickerData>>>,
    ticker_groups: Option<&crate::models::TickerGroups>,
    config: &ContextConfig,
) -> String {
    if interested_tickers.is_empty() {
        return String::new();
    }

    let ticker_contexts: Vec<String> = interested_tickers
        .iter()
        .map(|ticker| {
            let mut sections = Vec::new();

            // Get chart context
            if let Some(chart_data) = market_data.get(ticker) {
                let chart_context = format_chart_context(
                    ticker,
                    chart_data.clone(),
                    config.chart_context_days,
                    config.context_date.as_deref(),
                );
                if !chart_context.is_empty() {
                    sections.push(chart_context);
                }
            }

            // Get money flow context
            if let Some(money_flow_data) = money_flow_result.and_then(|mf| mf.get(ticker)) {
                let money_flow_context = format_money_flow_context(
                    ticker,
                    money_flow_data,
                    config.money_flow_context_days,
                    config.context_date.as_deref(),
                );
                sections.push(money_flow_context);
            }

            // Get MA Score context
            if let Some(ma_data) = ma_score_data {
                let ma_score_context = format_ma_score_context(
                    ticker,
                    Some(ma_data),
                    config.ma_score_context_days,
                    config.ma_period,
                    config.context_date.as_deref(),
                );
                if !ma_score_context.contains("No MA Score data available") {
                    sections.push(ma_score_context);
                }
            }

            // Get sector information
            let sector_info = if let Some(groups) = ticker_groups {
                if let Some(sector) = find_ticker_sector(groups, ticker) {
                    format!(" (Sector: {})", sector)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            if sections.is_empty() {
                String::new()
            } else {
                format!("# {} - Detailed Analysis{}\n\n{}\n\n---", ticker, sector_info, sections.join("\n"))
            }
        })
        .filter(|context| !context.is_empty())
        .collect();

    // Build sector breakdown summary
    let mut sector_summary = String::new();
    if let Some(ticker_groups) = ticker_groups {
        let mut sector_groups: HashMap<String, Vec<String>> = HashMap::new();

        for ticker in interested_tickers {
            if let Some(sector_name) = find_ticker_sector(ticker_groups, ticker) {
                sector_groups.entry(sector_name).or_default().push(ticker.clone());
            } else {
                sector_groups.entry("Unknown".to_string()).or_default().push(ticker.clone());
            }
        }

        if sector_groups.len() > 1 {
            let sector_lines: Vec<String> = sector_groups
                .iter()
                .map(|(sector, tickers)| format!("- {}: {}", sector, tickers.join(", ")))
                .collect();

            sector_summary = format!(
                "# Sector Breakdown\n{}\n\n---\n\n",
                sector_lines.join("\n")
            );
        }
    }

    format!("{}{}", sector_summary, ticker_contexts.join("\n"))
}

/// Build VNINDEX market context
pub fn build_vnindex_market_context(
    vnindex_data: &[StockDataPoint],
    config: &ContextConfig,
) -> String {
    if vnindex_data.is_empty() {
        return String::new();
    }

    let vnindex_context = format_chart_context(
        "VNINDEX",
        vnindex_data.to_vec(),
        config.chart_context_days,
        config.context_date.as_deref(),
    );

    if vnindex_context.is_empty() {
        String::new()
    } else {
        format!("# Market Context\n{}", vnindex_context)
    }
}

/// Build market leaders context from money flow data
pub fn build_market_leaders_context(
    money_flow_result: &HashMap<String, MoneyFlowTickerData>,
    limit: usize,
    _config: &ContextConfig,
) -> String {
    if money_flow_result.is_empty() {
        return String::new();
    }

    // Get top performers by trend score
    let mut sorted_tickers: Vec<_> = money_flow_result
        .iter()
        .filter(|(ticker, _)| *ticker != "VNINDEX")
        .collect();

    sorted_tickers.sort_by(|a, b| b.1.trend_score.partial_cmp(&a.1.trend_score).unwrap_or(std::cmp::Ordering::Equal));

    let top_performers: Vec<String> = sorted_tickers
        .iter()
        .take(limit)
        .map(|(ticker, data)| {
            format!(
                "{}: TrendScore={:.2}, MoneyFlow={:.2}%, MarketCap={}",
                ticker,
                data.trend_score,
                data.signed_percentage_data.values().next().copied().unwrap_or(0.0),
                super::utils::format_number_with_separator(data.market_cap)
            )
        })
        .collect();

    if top_performers.is_empty() {
        String::new()
    } else {
        format!(
            "# Top {} Market Leaders by Money Flow\n{}",
            limit,
            top_performers.join("\n")
        )
    }
}

/// Build sector leaders context
pub fn build_sector_leaders_context(
    money_flow_result: &HashMap<String, MoneyFlowTickerData>,
    ticker_groups: &crate::models::TickerGroups,
    _config: &ContextConfig,
) -> String {
    if money_flow_result.is_empty() {
        return String::new();
    }

    // Group tickers by sector
    let mut sector_groups: HashMap<String, Vec<(&String, &MoneyFlowTickerData)>> = HashMap::new();

    for (ticker, data) in money_flow_result {
        if ticker == "VNINDEX" {
            continue;
        }

        let sector = find_ticker_sector(ticker_groups, ticker).unwrap_or("Unknown".to_string());
        sector_groups.entry(sector).or_default().push((ticker, data));
    }

    // Get top performer from each sector
    let mut sector_leaders = Vec::new();
    for (sector, mut tickers) in sector_groups {
        tickers.sort_by(|a, b| b.1.trend_score.partial_cmp(&a.1.trend_score).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((ticker, data)) = tickers.first() {
            sector_leaders.push(format!(
                "{} ({}): TrendScore={:.2}, MoneyFlow={:.2}%",
                ticker,
                sector,
                data.trend_score,
                data.signed_percentage_data.values().next().copied().unwrap_or(0.0)
            ));
        }
    }

    if sector_leaders.is_empty() {
        String::new()
    } else {
        format!(
            "# Sector Leaders by Money Flow\n{}",
            sector_leaders.join("\n")
        )
    }
}
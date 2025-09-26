use std::collections::HashMap;
use crate::models::StockDataPoint;
use crate::utils::money_flow_utils::MoneyFlowTickerData;
use super::context_builders::{build_vnindex_market_context, build_market_leaders_context, build_sector_leaders_context};
use super::types::ContextConfig;
use super::utils::find_ticker_sector;

/// Build market-wide money flow context for sector rotation analysis
pub fn build_market_money_flow_context(
    _all_ticker_data: &HashMap<String, Vec<StockDataPoint>>,
    ticker_groups: &crate::models::TickerGroups,
    vnindex_data: &[StockDataPoint],
    money_flow_result: Option<&HashMap<String, MoneyFlowTickerData>>,
    config: &ContextConfig,
) -> String {
    let mut context_lines = Vec::new();

    // 1. VNINDEX Chart Context
    let vnindex_context = build_vnindex_market_context(vnindex_data, config);
    if !vnindex_context.is_empty() {
        context_lines.push(vnindex_context);
    }

    // 2. Money flow analysis if available
    if let Some(money_flow_data) = money_flow_result {
        // Top market leaders
        let market_leaders = build_market_leaders_context(money_flow_data, 10, config);
        if !market_leaders.is_empty() {
            context_lines.push(market_leaders);
        }

        // Sector leaders
        let sector_leaders = build_sector_leaders_context(money_flow_data, ticker_groups, config);
        if !sector_leaders.is_empty() {
            context_lines.push(sector_leaders);
        }

        // High level money flow summary by sector
        let sector_flow_summary = build_sector_flow_summary(money_flow_data, ticker_groups, config);
        if !sector_flow_summary.is_empty() {
            context_lines.push(sector_flow_summary);
        }
    }

    context_lines.join("\n\n")
}

/// Build sector flow summary showing money flow patterns across sectors
fn build_sector_flow_summary(
    money_flow_result: &HashMap<String, MoneyFlowTickerData>,
    ticker_groups: &crate::models::TickerGroups,
    _config: &ContextConfig,
) -> String {
    if money_flow_result.is_empty() {
        return String::new();
    }

    // Group tickers by sector and calculate averages
    let mut sector_flows: HashMap<String, Vec<f64>> = HashMap::new();
    let mut sector_trend_scores: HashMap<String, Vec<f64>> = HashMap::new();

    for (ticker, data) in money_flow_result {
        if ticker == "VNINDEX" {
            continue;
        }

        let sector = find_ticker_sector(ticker_groups, ticker).unwrap_or("Unknown".to_string());

        // Get recent money flow percentage
        if let Some(recent_flow) = data.signed_percentage_data.values().next() {
            sector_flows.entry(sector.clone()).or_default().push(*recent_flow);
        }

        sector_trend_scores.entry(sector).or_default().push(data.trend_score);
    }

    // Calculate sector averages
    let mut sector_summaries = Vec::new();
    for sector in sector_flows.keys() {
        let flows = &sector_flows[sector];
        let trend_scores = &sector_trend_scores[sector];

        if !flows.is_empty() && !trend_scores.is_empty() {
            let avg_flow = flows.iter().sum::<f64>() / flows.len() as f64;
            let avg_trend_score = trend_scores.iter().sum::<f64>() / trend_scores.len() as f64;
            let ticker_count = flows.len();

            sector_summaries.push((
                sector.clone(),
                avg_flow,
                avg_trend_score,
                ticker_count,
            ));
        }
    }

    // Sort by average money flow percentage
    sector_summaries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    if sector_summaries.is_empty() {
        return String::new();
    }

    // Take top and bottom 5 sectors
    let top_sectors: Vec<String> = sector_summaries
        .iter()
        .take(5)
        .map(|(sector, avg_flow, avg_trend, count)| {
            format!(
                "- {} ({} stocks): AvgFlow={:.2}%, AvgTrendScore={:.2}",
                sector, count, avg_flow, avg_trend
            )
        })
        .collect();

    let bottom_sectors: Vec<String> = sector_summaries
        .iter()
        .rev()
        .take(5)
        .map(|(sector, avg_flow, avg_trend, count)| {
            format!(
                "- {} ({} stocks): AvgFlow={:.2}%, AvgTrendScore={:.2}",
                sector, count, avg_flow, avg_trend
            )
        })
        .collect();

    format!(
        "# Sector Money Flow Analysis\n## Top Performing Sectors\n{}\n\n## Underperforming Sectors\n{}",
        top_sectors.join("\n"),
        bottom_sectors.join("\n")
    )
}

/// Build money flow timeline context showing flow patterns over time
pub fn build_money_flow_timeline_context(
    money_flow_result: &HashMap<String, MoneyFlowTickerData>,
    interested_tickers: &[String],
    config: &ContextConfig,
) -> String {
    if money_flow_result.is_empty() || interested_tickers.is_empty() {
        return String::new();
    }

    let mut timeline_data: HashMap<String, Vec<(String, f64)>> = HashMap::new();

    // Collect timeline data for interested tickers
    for ticker in interested_tickers {
        if let Some(ticker_data) = money_flow_result.get(ticker) {
            for (date, flow) in &ticker_data.signed_percentage_data {
                timeline_data.entry(date.clone()).or_default().push((ticker.clone(), *flow));
            }
        }
    }

    // Sort dates chronologically
    let mut dates: Vec<String> = timeline_data.keys().cloned().collect();
    dates.sort();

    // Apply context date filter and get recent dates
    let filtered_dates = if let Some(context_date) = &config.context_date {
        dates.into_iter().filter(|date| date <= context_date).collect()
    } else {
        dates
    };

    let recent_dates: Vec<String> = filtered_dates
        .iter()
        .rev()
        .take(config.money_flow_context_days)
        .rev()
        .cloned()
        .collect();

    if recent_dates.is_empty() {
        return String::new();
    }

    let timeline_lines: Vec<String> = recent_dates
        .iter()
        .filter_map(|date| {
            let day_data = timeline_data.get(date)?;
            if day_data.is_empty() {
                return None;
            }

            let ticker_flows: Vec<String> = day_data
                .iter()
                .map(|(ticker, flow)| {
                    format!("{}:{:.2}%", ticker, flow)
                })
                .collect();

            Some(format!("{}: {}", date, ticker_flows.join(", ")))
        })
        .collect();

    if timeline_lines.is_empty() {
        String::new()
    } else {
        format!(
            "# Money Flow Timeline (Last {} Days)\n{}",
            config.money_flow_context_days,
            timeline_lines.join("\n")
        )
    }
}